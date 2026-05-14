//! Transport — how a child process is held.
//!
//! Each transport owns the platform mechanics (PTY mastery, pipe
//! parsing, tmux window management). The orchestrator (`spawn_actor`)
//! never inspects transport internals; it just calls `spawn` and
//! holds the resulting `ChildHandle`.

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::broadcast;

use super::profile::ActorProfile;

const OUTPUT_CAP: usize = 1024;

/// Marker trait for transports — gates the spawn primitive. PTY for
/// long-lived interactive children, Headless for one-shot per-turn /
/// pipeline runs. Future variants (tmux, server-session) bolt on here.
pub trait Transport: Send + Sync {
    fn spawn(&self, actor_id: &str, profile: &ActorProfile) -> Result<ChildHandle>;
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

#[derive(Default)]
pub struct PtyTransport;

#[derive(Default)]
pub struct HeadlessTransport;

/// Live child handle. Holds writer + child so input flows back to
/// stdin (PTY master or stdin pipe); a reader thread tees output to
/// the log file and broadcast channel. Drop semantics are deliberate:
/// dropping does NOT kill the child. Call `stop()` for that.
pub struct ChildHandle {
    actor_id: String,
    pid: Mutex<Option<u32>>,
    /// Stdin writer. `Some` for PTY (master fd) and Headless with
    /// piped stdin; `None` for Headless without stdin (the common
    /// per-turn case where the prompt is in argv).
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    /// PTY master kept alive for the lifetime of a PTY child —
    /// dropping the master closes the slave's stdio. `None` for
    /// Headless.
    _master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    child: Mutex<Option<Box<dyn ChildProcess>>>,
    output_tx: broadcast::Sender<Vec<u8>>,
    log_path: PathBuf,
    spawned_at: chrono::DateTime<Utc>,
}

impl ChildHandle {
    pub fn write_input(&self, bytes: &[u8]) -> Result<()> {
        let mut guard = self.writer.lock().expect("ChildHandle writer mutex poisoned");
        let writer = guard
            .as_mut()
            .ok_or_else(|| anyhow!("actor '{}': writer closed", self.actor_id))?;
        writer.write_all(bytes)?;
        writer.flush()?;
        Ok(())
    }

    pub fn subscribe_output(&self) -> broadcast::Receiver<Vec<u8>> {
        self.output_tx.subscribe()
    }

    pub fn stop(&self) -> Result<()> {
        let mut guard = self.child.lock().expect("ChildHandle child mutex poisoned");
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
        Ok(())
    }

    pub fn pid(&self) -> Option<u32> {
        *self.pid.lock().expect("ChildHandle pid mutex poisoned")
    }

    pub fn is_alive(&self) -> bool {
        self.pid()
            .and_then(|p| std::fs::metadata(format!("/proc/{p}")).ok())
            .is_some()
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
        let mut guard = self.child.lock().expect("ChildHandle child mutex poisoned");
        guard.as_mut().and_then(|c| c.try_wait().ok().flatten())
    }

    /// Resize the PTY, if this child is held over a PTY transport.
    /// No-op for Headless transport (no master to resize). Returns Ok
    /// in both cases; the no-op is silent because xterm.js sends a
    /// resize on every layout change and PTY-less attach is a valid
    /// readonly state (mesh-view debug attach to a stopped pipeline,
    /// for example).
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let guard = self._master.lock().expect("ChildHandle _master mutex poisoned");
        let Some(master) = guard.as_ref() else { return Ok(()) };
        master
            .resize(PtySize { cols, rows, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| anyhow!("pty resize: {e}"))?;
        Ok(())
    }
}

impl Transport for PtyTransport {
    fn spawn(&self, actor_id: &str, profile: &ActorProfile) -> Result<ChildHandle> {
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
        let handle = ChildHandle {
            actor_id: actor_id.to_string(),
            pid: Mutex::new(pid),
            writer: Mutex::new(Some(writer)),
            _master: Mutex::new(Some(pair.master)),
            child: Mutex::new(Some(child_box)),
            output_tx: output_tx.clone(),
            log_path: log_path.clone(),
            spawned_at: Utc::now(),
        };

        // Reader thread: tees PTY output to log file and broadcast
        // channel. Exits when the child closes its end. Reaps the
        // ChildHandle's stored child via `try_wait` indirectly — the
        // master drop in handle's mutex closes the fd.
        let actor = actor_id.to_string();
        std::thread::spawn(move || {
            reader_loop(reader, log_file, output_tx, actor);
        });

        Ok(handle)
    }
}

impl Transport for HeadlessTransport {
    fn spawn(&self, actor_id: &str, profile: &ActorProfile) -> Result<ChildHandle> {
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
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .with_context(|| format!("spawn {}", profile.bin))?;
        let pid = child.id();
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("no stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow!("no stderr"))?;

        let (output_tx, _) = broadcast::channel(OUTPUT_CAP);

        let handle = ChildHandle {
            actor_id: actor_id.to_string(),
            pid: Mutex::new(Some(pid)),
            writer: Mutex::new(None),
            _master: Mutex::new(None),
            child: Mutex::new(Some(Box::new(child))),
            output_tx: output_tx.clone(),
            log_path: log_path.clone(),
            spawned_at: Utc::now(),
        };

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
