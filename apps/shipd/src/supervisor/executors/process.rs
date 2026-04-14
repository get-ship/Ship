//! ProcessExecutor — spawns agents as direct child processes.
//!
//! No tmux dependency. The runtime owns the PID and detects exit. Output
//! (stdout + stderr, including the streamed-JSON event log from claude) is
//! written to `.artifacts/agent-<slug>.log` per step so steps don't clobber
//! each other's logs.
//!
//! This is the PREFERRED executor for pipeline jobs. The tmux path is
//! retained for interactive sessions where the user wants to attach and
//! watch/chat with the agent. Pipelines are hands-off by definition — they
//! should not require a TUI to coordinate.
//!
//! What we gain over tmux:
//!   - No channels-prompt dismissal (CLI flags bypass the interactive menu)
//!   - No pane scraping / ready-pattern detection / key-name fiddling
//!   - Deterministic exit codes map cleanly to verdict evidence
//!   - Each step is its own process — no residual state from the prior step
//!
//! Coordination with the agent (send messages, broadcast events) continues
//! to use the mesh channel via `server:ship`, not the process stdin.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{ExecutorHandle, ExitSignal, JobContext, JobExecutor};

/// Job executor that spawns agents as direct child processes.
pub(crate) struct ProcessExecutor;

/// State held in the ExecutorHandle for process-based execution.
struct ProcessState {
    pid: u32,
}

#[async_trait]
impl JobExecutor for ProcessExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let log_dir = ctx.work_dir.join(".artifacts");
        std::fs::create_dir_all(&log_dir)?;
        // Per-step log so step-2+ doesn't overwrite step-1's output.
        let log_path = log_dir.join(format!("agent-{}.log", ctx.slug));
        let log_file = std::fs::File::create(&log_path)?;
        let err_file = log_file.try_clone()?;

        // Non-interactive launch. The `-p` prompt is the instruction the
        // agent receives on invocation — it points at the composed spec so
        // the agent reads current-step role + predecessor summaries + job
        // body + runtime contract.
        //
        // Claude CLI's `stream-json` output format requires `--verbose` (it
        // refuses otherwise, which previously caused the process to exit
        // immediately and the pipeline to "complete" in 19s doing nothing).
        // We're not currently parsing stream-json events, so use the default
        // human-readable format. Switching to stream-json later means adding
        // `--verbose` and a stdout parser.
        let prompt = "Read .artifacts/job-spec.md and execute it autonomously.";
        let mut args = vec![
            "--dangerously-skip-permissions",
            "--dangerously-load-development-channels", "server:ship",
            "--print",
            "-p", prompt,
        ];
        if let Some(ref m) = ctx.launch.model {
            args.push("--model");
            args.push(m);
        }
        let child = std::process::Command::new("claude")
            .args(&args)
            .env("SHIP_MESH_ID", &ctx.slug)
            .env("SHIP_AGENT_ID", &ctx.agent)
            .env("SHIP_SESSION_ID", &ctx.slug)
            .current_dir(&ctx.work_dir)
            .stdout(log_file)
            .stderr(err_file)
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn claude: {e}"))?;

        let pid = child.id();
        tracing::info!(
            slug = ctx.slug,
            pid,
            log = %log_path.display(),
            "process-executor: agent spawned"
        );

        // Monitor the process in a background thread — when it exits, signal via exit_tx.
        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let log_path_clone = log_path.clone();
        let exit_tx = ctx.exit_tx.clone();
        std::thread::spawn(move || {
            let mut child = child;
            let exit_code = child.wait().ok().and_then(|s| s.code());
            tracing::info!(slug, pid, exit_code, "process-executor: agent exited");
            drop(log_path_clone);
            if let Some(tx) = exit_tx {
                let _ = tx.send(ExitSignal { slug, job_id, exit_code });
            }
        });

        Ok(ExecutorHandle {
            pid: Some(pid),
            inner: Box::new(ProcessState { pid }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let state = match handle.inner.downcast_ref::<ProcessState>() {
            Some(s) => s,
            None => return false,
        };
        // Check if process exists (signal 0 = existence check)
        unsafe { libc::kill(state.pid as i32, 0) == 0 }
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let state = match handle.inner.downcast_ref::<ProcessState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let pid = state.pid as i32;
        unsafe { libc::kill(pid, libc::SIGTERM); }
        tracing::debug!(pid, "process-executor: sent SIGTERM, waiting up to 30s");

        // Wait up to 30s for the process to exit.
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if unsafe { libc::kill(pid, 0) } != 0 {
                tracing::debug!(pid, "process-executor: process exited after SIGTERM");
                return Ok(());
            }
        }

        // Still alive after 30s — force-kill.
        tracing::warn!(pid, "process-executor: process did not exit after 30s, sending SIGKILL");
        unsafe { libc::kill(pid, libc::SIGKILL); }
        Ok(())
    }

    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> Result<()> {
        // Non-interactive process — messages not supported.
        // Use mesh events for agent communication instead.
        Err(anyhow::anyhow!("ProcessExecutor does not support interactive messages"))
    }
}
