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
use runtime::events::process::{event_types as proc_events, ProcessKilledPayload};
use runtime::events::{EventEnvelope, KernelRouter};
use runtime::services::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, JobContext, JobExecutor,
};
use runtime::services::orchestrator::emit::persist_and_route;
use tokio::sync::Mutex as TokioMutex;

/// Job executor that spawns agents as direct child processes under a PTY.
///
/// `kernel` is optional so unit tests that don't wire a runtime kernel
/// can construct the executor bare. Emission is skipped when `None`.
pub(crate) struct ProcessExecutor {
    kernel: Option<Arc<TokioMutex<KernelRouter>>>,
}

impl ProcessExecutor {
    pub(crate) fn new(kernel: Option<Arc<TokioMutex<KernelRouter>>>) -> Self {
        Self { kernel }
    }
}

/// State held in the ExecutorHandle for process-based execution.
///
/// `child` is wrapped in `Arc<Mutex<_>>` so the monitor thread can poll
/// `try_wait()` while `stop()` (and the rare `is_alive()` probe) share
/// the same handle. `_master` keeps the PTY master alive for the lifetime
/// of the handle — the reader thread has its own cloned reader; dropping
/// the master early would close the slave side and signal HUP to the
/// child.
struct ProcessState {
    slug: String,
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
                let _ = tx.send(ExitSignal {
                    slug,
                    job_id,
                    exit_code,
                    cause: ExitCause::AgentExited,
                });
            }
        });

        Ok(ExecutorHandle {
            pid,
            inner: Box::new(ProcessState {
                slug: ctx.slug.clone(),
                pid,
                child,
                _master: master,
            }),
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

        emit_process_killed(
            self.kernel.as_ref(),
            &state.slug,
            pid,
            "SIGTERM",
            "graceful_shutdown",
        )
        .await;

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
        {
            let mut guard = state.child.lock().unwrap();
            if let Some(c) = guard.as_mut() {
                let _ = c.kill();
            }
        }

        emit_process_killed(
            self.kernel.as_ref(),
            &state.slug,
            pid,
            "SIGKILL",
            "force_after_grace_period",
        )
        .await;
        Ok(())
    }

    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> Result<()> {
        // Non-interactive process — messages not supported.
        // Use mesh events for agent communication instead.
        Err(anyhow::anyhow!("ProcessExecutor does not support interactive messages"))
    }
}

/// Build a `process.killed` envelope and persist+route it via the kernel.
/// Mirrors the inline pattern used in `handlers.rs::cancel_job` so both
/// call sites project into the same event stream.
async fn emit_process_killed(
    kernel: Option<&Arc<TokioMutex<KernelRouter>>>,
    slug: &str,
    pid: Option<u32>,
    signal: &str,
    reason: &str,
) {
    let Some(kernel) = kernel else { return };
    let payload = ProcessKilledPayload {
        slug: slug.to_string(),
        pid,
        signal: signal.to_string(),
        reason: reason.to_string(),
        actor: "supervisor".to_string(),
    };
    match EventEnvelope::new(proc_events::PROCESS_KILLED, slug, &payload) {
        Ok(env) => {
            let env = env.with_actor_id("service.process-executor");
            persist_and_route(kernel, env, slug, None).await;
        }
        Err(e) => {
            tracing::warn!(slug, "process-executor: failed to build process.killed envelope: {e}");
        }
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

    // ── emission tests ─────────────────────────────────────────────────────
    //
    // These tests drive `stop()` end-to-end by constructing a real PTY child
    // + ProcessState and invoking the JobExecutor trait. They assert the
    // event store actually contains a `process.killed` envelope afterward,
    // not just that the call site fires — matches the spec's acceptance
    // criterion for event-store reads.

    use runtime::events::kernel_router::KernelRouter;
    use runtime::events::process::event_types as proc_events;
    use runtime::db::events::query_events_by_entity_and_type_at;
    use std::sync::Arc;

    /// Spin up a PTY child running `sleep 60` and wrap it in a ProcessState
    /// ready for `stop()`. Returns the state plus the tempdir that anchors
    /// the log path for lifetime control.
    fn spawn_sleep_state(slug: &str) -> (tempfile::TempDir, ProcessState) {
        let tmp = tempfile::tempdir().unwrap();
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .unwrap();
        let mut cmd = CommandBuilder::new("sh");
        cmd.arg("-c");
        cmd.arg("sleep 60");
        cmd.cwd(tmp.path());
        let child = pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);
        let pid = child.process_id();
        let state = ProcessState {
            slug: slug.to_string(),
            pid,
            child: Arc::new(std::sync::Mutex::new(Some(child))),
            _master: Arc::new(std::sync::Mutex::new(pair.master)),
        };
        (tmp, state)
    }

    fn setup_kernel() -> (tempfile::TempDir, Arc<TokioMutex<KernelRouter>>, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join(".ship");
        let router = KernelRouter::new(base.clone()).unwrap();
        let db = base.join("platform.db");
        (dir, Arc::new(TokioMutex::new(router)), db)
    }

    /// `stop()` must project a `process.killed` envelope with signal=SIGTERM
    /// and reason=graceful_shutdown — the audit record that the graceful
    /// path fired.
    #[tokio::test]
    #[cfg_attr(not(unix), ignore)]
    async fn process_executor_emits_process_killed_on_sigterm_path() {
        let (_dir, kernel, db) = setup_kernel();
        let (_tmp, state) = spawn_sleep_state("proc-sigterm");

        let executor = ProcessExecutor::new(Some(Arc::clone(&kernel)));
        let handle = ExecutorHandle { pid: state.pid, inner: Box::new(state) };

        executor.stop(&handle).await.expect("stop should succeed");

        let events =
            query_events_by_entity_and_type_at(&db, "proc-sigterm", proc_events::PROCESS_KILLED)
                .expect("query must succeed");
        let payload: ProcessKilledPayload = events
            .iter()
            .find_map(|e| {
                let p: ProcessKilledPayload = serde_json::from_str(&e.payload_json).ok()?;
                (p.signal == "SIGTERM").then_some(p)
            })
            .expect("process.killed SIGTERM event must be persisted");
        assert_eq!(payload.reason, "graceful_shutdown");
        assert_eq!(payload.actor, "supervisor");
        assert_eq!(payload.slug, "proc-sigterm");
    }

    /// Source-level check: the SIGKILL fallback site emits `process.killed`
    /// with signal=SIGKILL and reason=force_after_grace_period. A true
    /// end-to-end SIGKILL test requires a process that ignores SIGTERM and
    /// a 30-second wait — too flaky and slow for the lib test suite. The
    /// SIGTERM test above verifies the emit-helper plumbing; this pins the
    /// distinct call-site arguments by source inspection.
    #[test]
    fn process_executor_source_emits_process_killed_on_sigkill_fallback() {
        let src = include_str!("process.rs");
        assert!(
            src.contains("\"SIGKILL\","),
            "stop() must emit process.killed with signal=SIGKILL after the fallback"
        );
        assert!(
            src.contains("\"force_after_grace_period\""),
            "SIGKILL emit must carry reason=force_after_grace_period"
        );
    }
}
