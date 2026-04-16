//! ProcessExecutor — spawns agents as direct child processes under a
//! real pseudo-terminal (PTY).
//!
//! No tmux dependency. The runtime owns the PID and detects exit via
//! `portable_pty`'s `Child::try_wait()`. Output (stdout + stderr — the PTY
//! merges them, including the streamed-JSON event log from claude) is
//! read from the PTY master and tee'd into `.artifacts/agent-<slug>.log`
//! per step so steps don't clobber each other's logs.
//!
//! Why a real PTY?
//!   - agents like Claude check `isatty()` and adjust output formatting;
//!     a real TTY avoids the "plain-text fallback" mode.
//!   - the master reader is line-buffered — future work can attach a
//!     stream-json parser to the same source without file-tail polling.
//!   - Windows support comes for free (ConPTY under the hood).
//!
//! This is the PREFERRED executor for pipeline jobs. The tmux-window path
//! (`TmuxWindowExecutor`) is retained for interactive sessions where the
//! user wants to attach and watch/chat with the agent. Pipelines are
//! hands-off by definition — they should not require a TUI to coordinate.
//!
//! Coordination with the agent (send messages, broadcast events) continues
//! to use the mesh channel via `server:ship`, not the process stdin.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use runtime::services::dispatch_ports::{ExecutorHandle, ExitSignal, JobContext, JobExecutor};

/// Job executor that spawns agents as direct child processes under a PTY.
pub(crate) struct ProcessExecutor;

/// State held in the ExecutorHandle for process-based execution.
///
/// `child` is wrapped in `Arc<Mutex<_>>` so the monitor thread can poll
/// `try_wait()` while `stop()` (and the rare `is_alive()` probe) share
/// the same handle. `_master` keeps the PTY master alive for the lifetime
/// of the handle — the reader thread has its own cloned reader; dropping
/// the master early would close the slave side and signal HUP to the
/// child.
struct ProcessState {
    pid: Option<u32>,
    child: Arc<Mutex<Option<Box<dyn Child + Send + Sync>>>>,
    _master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
}

#[async_trait]
impl JobExecutor for ProcessExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let log_dir = ctx.work_dir.join(".artifacts");
        std::fs::create_dir_all(&log_dir)?;
        // Per-step log so step-2+ doesn't overwrite step-1's output.
        let log_path = log_dir.join(format!("agent-{}.log", ctx.slug));
        let log_file = std::fs::File::create(&log_path)?;

        // Non-interactive launch. The `-p` prompt is the instruction the
        // agent receives on invocation — it points at the composed spec so
        // the agent reads current-step role + predecessor summaries + job
        // body + runtime contract.
        //
        // Claude CLI's `stream-json` output format requires `--verbose` (it
        // refuses otherwise). We're not currently parsing stream-json events,
        // so use the default human-readable format. Switching to stream-json
        // later means adding `--verbose` and a stdout parser; the PTY master
        // reader is the natural place to hang that parser off of.
        let prompt = "Read .artifacts/job-spec.md and execute it autonomously.";
        let mut args: Vec<String> = vec![
            "--dangerously-skip-permissions".into(),
            "--dangerously-load-development-channels".into(),
            "server:ship".into(),
            "--print".into(),
            "-p".into(),
            prompt.into(),
        ];
        if let Some(ref m) = ctx.launch.model {
            args.push("--model".into());
            args.push(m.clone());
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| anyhow::anyhow!("openpty failed: {e}"))?;

        let mut cmd = CommandBuilder::new("claude");
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.cwd(&ctx.work_dir);
        cmd.env("SHIP_MESH_ID", &ctx.slug);
        cmd.env("SHIP_AGENT_ID", &ctx.agent);
        cmd.env("SHIP_SESSION_ID", &ctx.slug);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| anyhow::anyhow!("failed to spawn claude: {e}"))?;
        // Drop the slave so the master-side reader hits EOF on child exit.
        drop(pair.slave);

        let pid = child.process_id();
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| anyhow::anyhow!("pty clone_reader failed: {e}"))?;

        // Reader thread: tee PTY master output into the per-slug log file.
        // Future: fan this reader out to a structured-event parser.
        let log_path_reader = log_path.clone();
        std::thread::spawn(move || {
            tee_reader_to_file(reader, log_file, &log_path_reader);
        });

        let child = Arc::new(Mutex::new(Some(child)));
        let master = Arc::new(Mutex::new(pair.master));

        tracing::info!(
            slug = ctx.slug,
            pid,
            log = %log_path.display(),
            "process-executor: agent spawned under PTY"
        );

        // Monitor the process in a background thread — poll try_wait() so
        // we share the child with stop()/is_alive() via the mutex.
        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let log_path_clone = log_path.clone();
        let exit_tx = ctx.exit_tx.clone();
        let child_mon = child.clone();
        std::thread::spawn(move || {
            let exit_code = loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let mut guard = child_mon.lock().unwrap();
                let Some(c) = guard.as_mut() else { break None };
                match c.try_wait() {
                    Ok(Some(status)) => break Some(status.exit_code() as i32),
                    Ok(None) => continue,
                    Err(_) => break None,
                }
            };
            tracing::info!(slug, pid, exit_code, "process-executor: agent exited");
            drop(log_path_clone);
            if let Some(tx) = exit_tx {
                let _ = tx.send(ExitSignal { slug, job_id, exit_code });
            }
        });

        Ok(ExecutorHandle {
            pid,
            inner: Box::new(ProcessState { pid, child, _master: master }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let state = match handle.inner.downcast_ref::<ProcessState>() {
            Some(s) => s,
            None => return false,
        };
        let mut guard = state.child.lock().unwrap();
        match guard.as_mut() {
            None => false,
            Some(c) => matches!(c.try_wait(), Ok(None)),
        }
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let state = match handle.inner.downcast_ref::<ProcessState>() {
            Some(s) => s,
            None => return Ok(()),
        };

        // Graceful-first shutdown: SIGTERM, then poll for exit up to 30s,
        // then SIGKILL. The reliability contract in
        // tests_graceful_shutdown.rs pins this sequence.
        let pid = state.pid;
        #[cfg(unix)]
        if let Some(pid) = pid {
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        }
        tracing::debug!(pid, "process-executor: sent SIGTERM, waiting up to 30s");

        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let exited = {
                let mut guard = state.child.lock().unwrap();
                match guard.as_mut() {
                    None => true,
                    Some(c) => matches!(c.try_wait(), Ok(Some(_)) | Err(_)),
                }
            };
            if exited {
                tracing::debug!(pid, "process-executor: process exited after SIGTERM");
                return Ok(());
            }
        }

        // Still alive after 30s — force-kill via portable_pty (SIGKILL on
        // Unix, TerminateProcess on Windows).
        tracing::warn!(pid, "process-executor: process did not exit after 30s, sending SIGKILL");
        let mut guard = state.child.lock().unwrap();
        if let Some(c) = guard.as_mut() {
            let _ = c.kill();
        }
        Ok(())
    }

    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> Result<()> {
        // Non-interactive process — messages not supported.
        // Use mesh events for agent communication instead.
        Err(anyhow::anyhow!("ProcessExecutor does not support interactive messages"))
    }
}

fn tee_reader_to_file(
    mut reader: Box<dyn Read + Send>,
    mut sink: std::fs::File,
    log_path: &std::path::Path,
) {
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = sink.write_all(&buf[..n]) {
                    tracing::warn!(log = %log_path.display(), "pty log write failed: {e}");
                    break;
                }
                let _ = sink.flush();
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// Exercises the PTY plumbing end-to-end: openpty → spawn `echo` →
    /// reader-thread tee → try_wait() flips to Some after exit → log
    /// file contains the child's stdout. We can't easily stub the
    /// `claude` binary that `spawn()` hardcodes, but every step of the
    /// PTY path is the same for any command.
    #[tokio::test]
    #[cfg_attr(not(unix), ignore)]
    async fn echo_under_pty_writes_log_and_exits() {
        let tmp = tempfile::tempdir().unwrap();
        let log_path = tmp.path().join("echo.log");
        let log_file = std::fs::File::create(&log_path).unwrap();

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .unwrap();
        let mut cmd = CommandBuilder::new("echo");
        cmd.arg("hello");
        cmd.cwd(tmp.path());
        let mut child = pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().unwrap();
        let log_path_r = log_path.clone();
        std::thread::spawn(move || tee_reader_to_file(reader, log_file, &log_path_r));

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match child.try_wait().unwrap() {
                Some(_) => break,
                None if Instant::now() >= deadline => panic!("echo did not exit in 5s"),
                None => std::thread::sleep(Duration::from_millis(50)),
            }
        }

        // Give the reader thread a moment to drain after EOF.
        std::thread::sleep(Duration::from_millis(200));
        let contents = std::fs::read_to_string(&log_path).unwrap();
        assert!(
            contents.contains("hello"),
            "expected log to contain 'hello', got {contents:?}"
        );

        // Keep the master alive until after we've read — mirrors
        // ProcessState._master semantics.
        drop(pair.master);
    }
}
