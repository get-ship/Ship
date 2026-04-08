//! TmuxExecutor — spawns agents in named tmux sessions.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{ExecutorHandle, ExitSignal, JobContext, JobExecutor};
use runtime::services::provider_adapter::adapter_for;

use super::super::helpers::{ensure_tmux_session, kill_tmux_session};
use super::super::terminal_launcher;

/// Job executor that spawns agents in tmux sessions.
pub(crate) struct TmuxExecutor;

#[async_trait]
impl JobExecutor for TmuxExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let tmux_session = format!("job-{}", ctx.slug);
        ensure_tmux_session(&tmux_session, &ctx.work_dir)?;

        let adapter = adapter_for(ctx.launch.provider.as_deref());
        let cmd = adapter.build_interactive(&ctx.slug, &ctx.launch);
        let shell_cmd = cmd.to_shell_string();

        let result = std::process::Command::new("tmux")
            .args(["send-keys", "-t", &tmux_session, &shell_cmd, "Enter"])
            .status();
        if let Err(e) = result {
            tracing::warn!(session = tmux_session, "tmux send-keys failed: {e}");
        }

        // Wait for the agent to be ready, send autostart, then monitor liveness.
        let session = tmux_session.clone();
        let ready_pat = adapter.ready_pattern().to_string();
        let autostart = adapter.autostart_input().to_string();
        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let exit_tx = ctx.exit_tx.clone();
        std::thread::spawn(move || {
            if wait_for_ready(&session, &ready_pat, 30) {
                let _ = std::process::Command::new("tmux")
                    .args(["send-keys", "-t", &session, &autostart, "Enter"])
                    .status();
            }
            // After readiness (or timeout), monitor for session death.
            if let Some(tx) = exit_tx {
                monitor_tmux_session(&session, &slug, &job_id, tx);
            }
        });

        let (strategy, launched) = terminal_launcher::launch(&tmux_session);
        tracing::info!(
            slug = ctx.slug, strategy, launched,
            "job-dispatch: terminal launch attempted"
        );

        Ok(ExecutorHandle {
            pid: None,
            inner: Box::new(tmux_session),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return false,
        };
        std::process::Command::new("tmux")
            .args(["has-session", "-t", session])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return Ok(()),
        };
        kill_tmux_session(session)
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let status = std::process::Command::new("tmux")
            .args(["send-keys", "-t", session, message, "Enter"])
            .status()
            .map_err(|e| anyhow::anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!("tmux send-keys returned non-zero"));
        }
        Ok(())
    }
}

/// Poll tmux pane until the ready pattern appears or timeout expires.
fn wait_for_ready(session: &str, pattern: &str, timeout_secs: u64) -> bool {
    for _ in 0..timeout_secs {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let output = std::process::Command::new("tmux")
            .args(["capture-pane", "-t", session, "-p"])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.contains(pattern) {
                return true;
            }
        }
    }
    tracing::warn!(session, "agent: timed out waiting for ready pattern");
    false
}

/// Poll the tmux session every 5s; send ExitSignal when it dies.
///
/// Starts after `wait_for_ready` returns so we don't fire before the agent
/// even launches. The thread is self-terminating — once the session dies the
/// signal is sent and the thread exits.
fn monitor_tmux_session(
    session: &str,
    slug: &str,
    job_id: &str,
    exit_tx: tokio::sync::mpsc::UnboundedSender<ExitSignal>,
) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let alive = std::process::Command::new("tmux")
            .args(["has-session", "-t", session])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !alive {
            tracing::info!(session, slug, "tmux-executor: session dead, signalling exit");
            let _ = exit_tx.send(ExitSignal {
                slug: slug.to_string(),
                job_id: job_id.to_string(),
                exit_code: None, // tmux does not expose the agent's exit code
            });
            return;
        }
    }
}
