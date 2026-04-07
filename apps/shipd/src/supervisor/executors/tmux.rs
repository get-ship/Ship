//! TmuxExecutor — spawns agents in named tmux sessions.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{ExecutorHandle, JobContext, JobExecutor};

use super::super::helpers::{ensure_tmux_session, kill_tmux_session};
use super::super::terminal_launcher;

/// Job executor that spawns agents in tmux sessions.
pub(crate) struct TmuxExecutor;

#[async_trait]
impl JobExecutor for TmuxExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let tmux_session = format!("job-{}", ctx.slug);
        ensure_tmux_session(&tmux_session, &ctx.work_dir)?;
        spawn_agent_with_mesh_id(&tmux_session, &ctx.slug, ctx.model.as_deref());

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

/// Send agent CLI command into the tmux session with SHIP_MESH_ID set.
fn spawn_agent_with_mesh_id(tmux_session: &str, slug: &str, model: Option<&str>) {
    let model_flag = match model {
        Some(m) => format!(" --model {m}"),
        None => String::new(),
    };
    let cmd = format!(
        "SHIP_MESH_ID={slug} claude --dangerously-skip-permissions \
         --dangerously-load-development-channels server:ship{model_flag}"
    );
    let result = std::process::Command::new("tmux")
        .args(["send-keys", "-t", tmux_session, &cmd, "Enter"])
        .status();
    if let Err(e) = result {
        tracing::warn!(session = tmux_session, "tmux send-keys failed: {e}");
        return;
    }

    // Wait for claude to be ready, then auto-accept prompts and send "go".
    // Polls the tmux pane for the input prompt (❯) instead of blind delays.
    let session = tmux_session.to_string();
    std::thread::spawn(move || {
        // Phase 1: Wait for experimental channels prompt (up to 30s)
        if wait_for_prompt(&session, 30) {
            // Accept the experimental channels prompt
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", &session, "Enter"])
                .status();
        }
        // Phase 2: Wait for the main input prompt (up to 30s)
        if wait_for_prompt(&session, 30) {
            // Send "go" to trigger job autostart
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", &session, "go", "Enter"])
                .status();
        }
    });
}

/// Poll tmux pane until the Claude input prompt (❯) appears.
/// Returns true if found, false on timeout.
fn wait_for_prompt(session: &str, timeout_secs: u64) -> bool {
    for _ in 0..timeout_secs {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let output = std::process::Command::new("tmux")
            .args(["capture-pane", "-t", session, "-p"])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            // Claude shows ❯ when ready for input
            if text.contains('❯') {
                return true;
            }
        }
    }
    tracing::warn!(session, "agent: timed out waiting for prompt");
    false
}
