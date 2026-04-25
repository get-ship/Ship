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

/// Marker trait for transports — gates the spawn primitive. Today PTY
/// is the only impl; headless / tmux / server arrive as the
/// migration progresses.
pub trait Transport: Send + Sync {
    fn spawn(&self, actor_id: &str, profile: &ActorProfile) -> Result<ChildHandle>;
}

#[derive(Default)]
pub struct PtyTransport;

/// Live child handle. Holds the writer + master so input flows back
/// through the same fd; spawns a reader thread that tees output to
/// the log file and broadcast channel. Drop semantics are deliberate:
/// dropping does NOT kill the child. Call `stop()` for that.
pub struct ChildHandle {
    actor_id: String,
    pid: Mutex<Option<u32>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    /// Master kept alive for the lifetime of the child — dropping the
    /// master closes the slave's stdio.
    _master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    child: Mutex<Option<Box<dyn portable_pty::Child + Send + Sync>>>,
    output_tx: broadcast::Sender<Vec<u8>>,
    log_path: PathBuf,
    spawned_at: chrono::DateTime<Utc>,
}

impl ChildHandle {
    pub fn write_input(&self, bytes: &[u8]) -> Result<()> {
        let mut guard = self.writer.lock().unwrap();
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
        let mut guard = self.child.lock().unwrap();
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
        Ok(())
    }

    pub fn pid(&self) -> Option<u32> {
        *self.pid.lock().unwrap()
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

        let mut child = pair
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

        let handle = ChildHandle {
            actor_id: actor_id.to_string(),
            pid: Mutex::new(pid),
            writer: Mutex::new(Some(writer)),
            _master: Mutex::new(Some(pair.master)),
            child: Mutex::new(Some(child)),
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
