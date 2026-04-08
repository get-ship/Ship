//! ProcessExecutor — spawns agents as direct child processes.
//!
//! No tmux dependency. The runtime owns the PID and detects exit.
//! Output is logged to `.ship-session/agent.log` in the worktree.
//! The agent runs non-interactively via `--print -p "go"`.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{ExecutorHandle, ExitSignal, JobContext, JobExecutor};
use std::path::PathBuf;

/// Job executor that spawns agents as direct child processes.
pub(crate) struct ProcessExecutor;

/// State held in the ExecutorHandle for process-based execution.
struct ProcessState {
    pid: u32,
    log_path: PathBuf,
}

#[async_trait]
impl JobExecutor for ProcessExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let log_dir = ctx.work_dir.join(".ship-session");
        std::fs::create_dir_all(&log_dir)?;
        let log_path = log_dir.join("agent.log");
        let log_file = std::fs::File::create(&log_path)?;
        let err_file = log_file.try_clone()?;

        let mut args = vec![
            "--dangerously-skip-permissions",
            "--dangerously-load-development-channels", "server:ship",
            "--print",
            "-p", "go",
        ];
        if let Some(ref m) = ctx.launch.model {
            args.push("--model");
            args.push(m);
        }
        let child = std::process::Command::new("claude")
            .args(&args)
            .env("SHIP_MESH_ID", &ctx.slug)
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
            inner: Box::new(ProcessState { pid, log_path }),
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
        unsafe { libc::kill(state.pid as i32, libc::SIGTERM); }
        Ok(())
    }

    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> Result<()> {
        // Non-interactive process — messages not supported.
        // Use mesh events for agent communication instead.
        Err(anyhow::anyhow!("ProcessExecutor does not support interactive messages"))
    }
}
