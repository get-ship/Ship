//! TmuxExecutor — spawns agents in named tmux sessions.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{ExecutorHandle, JobContext, JobExecutor};
use runtime::services::provider_adapter::adapter_for;

use super::super::helpers::{ensure_tmux_session, kill_tmux_session};
use super::super::terminal_launcher;

/// Job executor that spawns agents in tmux sessions.
pub(crate) struct TmuxExecutor;

/// State held in the ExecutorHandle for tmux-based execution.
struct TmuxState {
    session: String,
    provider: Option<String>,
}

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

        // Wait for the agent to be ready, then send autostart input.
        let session = tmux_session.clone();
        let ready_pat = adapter.ready_pattern().to_string();
        let autostart = adapter.autostart_input().to_string();
        std::thread::spawn(move || {
            if wait_for_ready(&session, &ready_pat, 30) {
                let _ = std::process::Command::new("tmux")
                    .args(["send-keys", "-t", &session, &autostart, "Enter"])
                    .status();
            }
        });

        let (strategy, launched) = terminal_launcher::launch(&tmux_session);
        tracing::info!(
            slug = ctx.slug, strategy, launched,
            "job-dispatch: terminal launch attempted"
        );

        Ok(ExecutorHandle {
            pid: None,
            inner: Box::new(TmuxState {
                session: tmux_session,
                provider: ctx.launch.provider.clone(),
            }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let state = match handle.inner.downcast_ref::<TmuxState>() {
            Some(s) => s,
            None => return false,
        };
        std::process::Command::new("tmux")
            .args(["has-session", "-t", &state.session])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let state = match handle.inner.downcast_ref::<TmuxState>() {
            Some(s) => s,
            None => return Ok(()),
        };

        // Send exit command for graceful shutdown before force-killing.
        let adapter = adapter_for(state.provider.as_deref());
        let exit_command = adapter.exit_command().to_string();
        let _ = std::process::Command::new("tmux")
            .args(["send-keys", "-t", &state.session, &exit_command, "Enter"])
            .status();
        tracing::debug!(
            session = %state.session,
            "tmux-executor: sent exit_command, waiting up to 30s"
        );

        // Wait up to 30s for the session to disappear.
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let alive = std::process::Command::new("tmux")
                .args(["has-session", "-t", &state.session])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !alive {
                tracing::debug!(session = %state.session, "tmux-executor: session exited cleanly");
                return Ok(());
            }
        }

        // Still alive after 30s — force-kill.
        tracing::warn!(
            session = %state.session,
            "tmux-executor: session did not exit after 30s, force-killing"
        );
        kill_tmux_session(&state.session)
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let state = match handle.inner.downcast_ref::<TmuxState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let status = std::process::Command::new("tmux")
            .args(["send-keys", "-t", &state.session, message, "Enter"])
            .status()
            .map_err(|e| anyhow::anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!("tmux send-keys returned non-zero"));
        }
        Ok(())
    }
}

/// Poll tmux pane until the ready pattern appears.
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
