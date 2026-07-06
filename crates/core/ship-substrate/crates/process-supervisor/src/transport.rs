//! Transport — how a child process is held.
//!
//! Each transport owns the platform mechanics (PTY mastery, pipe
//! parsing, tmux window management). The orchestrator (`spawn_process`)
//! never inspects transport internals; it just calls `spawn` and
//! holds the resulting [`ChildHandle`].
//!
//! ## Layering
//!
//! [`ChildHandle`] is the public actor-control surface (write_input,
//! subscribe_output, stop, pid, resize, ...). It does NOT carry the
//! transport's internals directly anymore — instead it delegates to
//! a [`ChildBackend`] trait object. That seam is what lets a
//! sandbox-backed actor have the same public surface even though
//! its bytes flow over a ConnectRPC wire instead of a local PTY.
//!
//! The two existing backends ([`PtyChildBackend`],
//! [`HeadlessChildBackend`]) keep the legacy behavior verbatim;
//! they're the "in-process" path. A sidecar-backed backend lands in
//! Chunk C2 alongside the SidecarRuntime.

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use tokio::sync::broadcast;
use super::profile::ProcessProfile;

const OUTPUT_CAP: usize = 1024;

/// Compute the PATH the supervisor passes to spawned children. Prepends
/// the dir containing the running daemon binary so a stashed `ship` next
/// to `shipd` (per `scripts/dev/_lib.sh::stash_binary`) wins over any
/// stale `~/.local/bin/ship` on the parent's PATH.
fn pinned_path_env() -> Option<String> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let existing = std::env::var("PATH").unwrap_or_default();
    let sep = if cfg!(windows) { ";" } else { ":" };
    Some(format!("{}{}{}", exe_dir.display(), sep, existing))
}

/// Marker trait for transports — gates the spawn primitive. PTY for
/// long-lived interactive children, Headless for one-shot per-turn /
/// pipeline runs. Future variants (tmux, server-session) bolt on here.
pub trait Transport: Send + Sync {
    fn spawn(&self, actor_id: &str, profile: &ProcessProfile) -> Result<ChildHandle>;
}

/// Type-erased child process — both portable_pty::Child and
/// std::process::Child fit. The supervisor only needs `kill` and a
/// non-blocking exit-code probe.
pub trait ChildProcess: Send + Sync {
    fn kill(&mut self) -> Result<()>;
    /// Returns `Some(exit_code)` if the child exited, `None` if still alive.
    fn try_wait(&mut self) -> Result<Option<i32>>;
}

impl ChildProcess for Box<dyn portable_pty::Child + Send + Sync> {
    fn kill(&mut self) -> Result<()> {
        self.as_mut().kill().map_err(|e| anyhow!("pty kill: {e}"))?;
        Ok(())
    }
    fn try_wait(&mut self) -> Result<Option<i32>> {
        Ok(self
            .as_mut()
            .try_wait()
            .map_err(|e| anyhow!("pty try_wait: {e}"))?
            .map(|s| s.exit_code() as i32))
    }
}

impl ChildProcess for std::process::Child {
    fn kill(&mut self) -> Result<()> {
        std::process::Child::kill(self).map_err(|e| anyhow!("kill: {e}"))?;
        Ok(())
    }
    fn try_wait(&mut self) -> Result<Option<i32>> {
        Ok(std::process::Child::try_wait(self)?
            .map(|s| s.code().unwrap_or(-1)))
    }
}

/// The transport-specific machinery behind a [`ChildHandle`]. Each
/// transport (PTY, Headless, future Sidecar) implements this trait
/// for its own backing — kernel PTY, plain pipes, or RPC over the
/// boundary protocol.
///
/// [`ChildHandle`] owns the broadcast Sender; the backend is given a
/// Sender at construction time and arranges for output bytes to land
/// on it. Subscribers always go through `ChildHandle::subscribe_output`.
pub trait ChildBackend: Send + Sync {
    /// Write to the actor's stdin / PTY master. Returns
    /// `ErrorKind::BrokenPipe`-shaped error if writer is closed.
    fn write_input(&self, bytes: &[u8]) -> Result<()>;
    /// Signal the actor to stop. PTY/headless: kill the child. Sidecar:
    /// `Processes.Stop` RPC. No-op-friendly; safe to call multiple times.
    fn stop(&self) -> Result<()>;
    /// Last known process id. None when the actor is sidecar-backed
    /// and the PID is opaque to the host.
    fn pid(&self) -> Option<u32>;
    /// Non-blocking exit probe. `Some(code)` if the actor has exited.
    fn try_exit(&self) -> Option<i32>;
    /// Resize the actor's PTY. No-op for backends without a PTY.
    fn resize(&self, cols: u16, rows: u16) -> Result<()>;
    /// Liveness probe. Default checks `/proc/{pid}`; backends with a
    /// richer signal (sidecar's `Processes.Get`) override.
    fn is_alive(&self) -> bool {
        self.pid()
            .and_then(|p| std::fs::metadata(format!("/proc/{p}")).ok())
            .is_some()
    }
}

#[derive(Default)]
pub struct PtyTransport;

#[derive(Default)]
pub struct HeadlessTransport;

/// Live actor control handle. Owns the broadcast sender for output
/// fan-out, the log path, and the transport-specific backend that
/// services write_input / stop / pid / resize.
///
/// Drop semantics are deliberate: dropping does NOT kill the child.
/// Call `stop()` for that. Matches the `lifecycle.rs::PtySession`
/// contract so migration is a drop-in replacement.
pub struct ChildHandle {
    actor_id: String,
    backend: Box<dyn ChildBackend>,
    output_tx: broadcast::Sender<Vec<u8>>,
    log_path: PathBuf,
    spawned_at: chrono::DateTime<Utc>,
}

impl ChildHandle {
    pub fn new(
        actor_id: impl Into<String>,
        backend: Box<dyn ChildBackend>,
        output_tx: broadcast::Sender<Vec<u8>>,
        log_path: PathBuf,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            backend,
            output_tx,
            log_path,
            spawned_at: Utc::now(),
        }
    }

    pub fn write_input(&self, bytes: &[u8]) -> Result<()> {
        self.backend.write_input(bytes).with_context(|| {
            format!("actor '{}': write_input", self.actor_id)
        })
    }

    pub fn subscribe_output(&self) -> broadcast::Receiver<Vec<u8>> {
        self.output_tx.subscribe()
    }

    pub fn stop(&self) -> Result<()> {
        self.backend.stop()
    }

    pub fn pid(&self) -> Option<u32> {
        self.backend.pid()
    }

    pub fn is_alive(&self) -> bool {
        self.backend.is_alive()
    }

    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    pub fn spawned_at(&self) -> chrono::DateTime<Utc> {
        self.spawned_at
    }

    /// Non-blocking exit probe. `Some(code)` if the child has exited,
    /// `None` if still running. Mostly useful immediately after the
    /// output broadcast closes — the reader exits on EOF, then the
    /// child status becomes available within a beat.
    pub fn try_exit(&self) -> Option<i32> {
        self.backend.try_exit()
    }

    /// Resize the PTY, if this child is held over a PTY transport.
    /// No-op for Headless transport (no master to resize). Returns Ok
    /// in both cases; the no-op is silent because xterm.js sends a
    /// resize on every layout change and PTY-less attach is a valid
    /// readonly state (mesh-view debug attach to a stopped pipeline,
    /// for example).
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.backend.resize(cols, rows)
    }
}

/// Backend for [`PtyTransport`] — kernel PTY master + spawned child.
pub struct PtyChildBackend {
    pid: Mutex<Option<u32>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    /// PTY master kept alive for the lifetime of the child — dropping
    /// closes the slave's stdio.
    master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    child: Mutex<Option<Box<dyn ChildProcess>>>,
}

impl ChildBackend for PtyChildBackend {
    fn write_input(&self, bytes: &[u8]) -> Result<()> {
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let writer = guard
            .as_mut()
            .ok_or_else(|| anyhow!("writer closed"))?;
        writer.write_all(bytes)?;
        writer.flush()?;
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        let mut guard = self.child.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
        Ok(())
    }

    fn pid(&self) -> Option<u32> {
        *self.pid.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn try_exit(&self) -> Option<i32> {
        let mut guard = self.child.lock().unwrap_or_else(|e| e.into_inner());
        guard.as_mut().and_then(|c| c.try_wait().ok().flatten())
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let guard = self.master.lock().unwrap_or_else(|e| e.into_inner());
        let Some(master) = guard.as_ref() else { return Ok(()) };
        master
            .resize(PtySize { cols, rows, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| anyhow!("pty resize: {e}"))?;
        Ok(())
    }
}

/// Backend for [`HeadlessTransport`] — piped stdin/stdout/stderr, no
/// PTY. `resize` is a no-op; `pid` uses the host process id.
pub struct HeadlessChildBackend {
    pid: Mutex<Option<u32>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    child: Mutex<Option<Box<dyn ChildProcess>>>,
}

impl ChildBackend for HeadlessChildBackend {
    fn write_input(&self, bytes: &[u8]) -> Result<()> {
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let writer = guard
            .as_mut()
            .ok_or_else(|| anyhow!("writer closed"))?;
        writer.write_all(bytes)?;
        writer.flush()?;
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        let mut guard = self.child.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
        Ok(())
    }

    fn pid(&self) -> Option<u32> {
        *self.pid.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn try_exit(&self) -> Option<i32> {
        let mut guard = self.child.lock().unwrap_or_else(|e| e.into_inner());
        guard.as_mut().and_then(|c| c.try_wait().ok().flatten())
    }

    fn resize(&self, _cols: u16, _rows: u16) -> Result<()> {
        Ok(())
    }
}

impl Transport for PtyTransport {
    fn spawn(&self, actor_id: &str, profile: &ProcessProfile) -> Result<ChildHandle> {
        let log_dir = match &profile.log_dir {
            Some(p) => p.clone(),
            None => super::default_log_dir()?,
        };
        std::fs::create_dir_all(&log_dir)
            .with_context(|| format!("create log dir {}", log_dir.display()))?;
        let log_path = log_dir.join(format!("{actor_id}.log"));
        let log_file = File::create(&log_path)
            .with_context(|| format!("create log {}", log_path.display()))?;

        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize { rows: 24, cols: 120, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| anyhow!("openpty: {e}"))?;

        let mut cmd = CommandBuilder::new(&profile.bin);
        for arg in &profile.args {
            cmd.arg(arg);
        }
        cmd.cwd(&profile.work_dir);
        for (k, v) in &profile.env {
            cmd.env(k, v);
        }
        cmd.env("SHIP_ACTOR_ID", actor_id);
        if let Some(a) = profile.agent_id.as_deref() {
            cmd.env("SHIP_AGENT_ID", a);
        }
        if let Some(path) = pinned_path_env() {
            cmd.env("PATH", path);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .with_context(|| format!("spawn {}", profile.bin))?;
        drop(pair.slave);
        let pid = child.process_id();

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| anyhow!("clone_reader: {e}"))?;
        let writer: Box<dyn Write + Send> = pair
            .master
            .take_writer()
            .map_err(|e| anyhow!("take_writer: {e}"))?;

        let (output_tx, _) = broadcast::channel(OUTPUT_CAP);

        let child_box: Box<dyn ChildProcess> = Box::new(child);
        let backend = PtyChildBackend {
            pid: Mutex::new(pid),
            writer: Mutex::new(Some(writer)),
            master: Mutex::new(Some(pair.master)),
            child: Mutex::new(Some(child_box)),
        };
        let handle = ChildHandle::new(actor_id, Box::new(backend), output_tx.clone(), log_path);

        // Reader thread: tees PTY output to log file and broadcast
        // channel. Exits when the child closes its end.
        let actor = actor_id.to_string();
        std::thread::spawn(move || {
            reader_loop(reader, log_file, output_tx, actor);
        });

        Ok(handle)
    }
}

impl Transport for HeadlessTransport {
    fn spawn(&self, actor_id: &str, profile: &ProcessProfile) -> Result<ChildHandle> {
        use std::process::{Command, Stdio};

        let log_dir = match &profile.log_dir {
            Some(p) => p.clone(),
            None => super::default_log_dir()?,
        };
        std::fs::create_dir_all(&log_dir)
            .with_context(|| format!("create log dir {}", log_dir.display()))?;
        let log_path = log_dir.join(format!("{actor_id}.log"));
        let log_file = File::create(&log_path)
            .with_context(|| format!("create log {}", log_path.display()))?;

        let mut cmd = Command::new(&profile.bin);
        for arg in &profile.args {
            cmd.arg(arg);
        }
        cmd.current_dir(&profile.work_dir);
        for (k, v) in &profile.env {
            cmd.env(k, v);
        }
        cmd.env("SHIP_ACTOR_ID", actor_id);
        if let Some(a) = profile.agent_id.as_deref() {
            cmd.env("SHIP_AGENT_ID", a);
        }
        if let Some(path) = pinned_path_env() {
            cmd.env("PATH", path);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Gap 5 (headless): place the child in its own process
        // group so `kill(-pgid, SIGTERM)` reaches every descendant
        // (claude's tool subagents, bash subshells). The PTY
        // transport already calls `setsid(2)` inside portable_pty
        // (the slave is its own session + pgid leader), so this
        // is the headless-only companion. `pre_exec` runs in the
        // forked child between fork and exec; failures abort the
        // spawn. Unix-only; Windows already creates a job object
        // per process so process-tree reaping is the OS's job.
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("spawn {}", profile.bin))?;
        let pid = child.id();
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("no stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow!("no stderr"))?;
        let (output_tx, _) = broadcast::channel(OUTPUT_CAP);

        let backend = HeadlessChildBackend {
            pid: Mutex::new(Some(pid)),
            writer: Mutex::new(None),
            child: Mutex::new(Some(Box::new(child))),
        };
        let handle = ChildHandle::new(actor_id, Box::new(backend), output_tx.clone(), log_path);

        // stdout reader: bytes → log + broadcast.
        let stdout_actor = actor_id.to_string();
        let stdout_tx = output_tx.clone();
        let stdout_log = log_file.try_clone().context("clone log file")?;
        std::thread::spawn(move || {
            reader_loop(Box::new(stdout), stdout_log, stdout_tx, stdout_actor);
        });
        // stderr reader: same broadcast (merged stream).
        let stderr_actor = actor_id.to_string();
        std::thread::spawn(move || {
            reader_loop(Box::new(stderr), log_file, output_tx, stderr_actor);
        });

        Ok(handle)
    }
}

fn reader_loop(
    mut reader: Box<dyn Read + Send>,
    mut log_file: File,
    output_tx: broadcast::Sender<Vec<u8>>,
    actor_id: String,
) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = &buf[..n];
                let _ = log_file.write_all(chunk);
                let _ = log_file.flush();
                let _ = output_tx.send(chunk.to_vec());
            }
            Err(e) => {
                tracing::debug!(actor = %actor_id, "pty reader exit: {e}");
                break;
            }
        }
    }
}
