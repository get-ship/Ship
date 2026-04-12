//! TmuxExecutor — spawns agents in named tmux sessions.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{ExecutorHandle, ExitSignal, JobContext, JobExecutor};
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

        // On step advance within the same pipeline, the tmux session from
        // the previous step is already running and the user already has a
        // terminal tab attached to it. The previous claude has exited (via
        // graceful_agent_exit before we got here) leaving a shell prompt.
        // We still need to launch the new agent inside the session, but
        // we MUST NOT open a new terminal tab — that's what was creating
        // duplicates.
        let reuse_session = tmux_session_exists(&tmux_session);

        if !reuse_session {
            ensure_tmux_session(&tmux_session, &ctx.work_dir)?;
        }

        let adapter = adapter_for(ctx.launch.provider.as_deref());
        let cmd = adapter.build_interactive(&ctx.slug, &ctx.launch);
        let shell_cmd = cmd.to_shell_string();

        let result = std::process::Command::new("tmux")
            .args(["send-keys", "-t", &tmux_session, &shell_cmd, "Enter"])
            .status();
        if let Err(e) = result {
            tracing::warn!(session = tmux_session, "tmux send-keys failed: {e}");
        }

        let session = tmux_session.clone();
        let ready_pat = adapter.ready_pattern().to_string();
        let autostart = adapter.autostart_input().to_string();
        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let exit_tx = ctx.exit_tx.clone();
        std::thread::spawn(move || {
            dismiss_channels_prompt(&session, 15);
            if wait_for_ready(&session, &ready_pat, 30) {
                let _ = std::process::Command::new("tmux")
                    .args(["send-keys", "-t", &session, &autostart, "Enter"])
                    .status();
            }
            if let Some(tx) = exit_tx {
                monitor_tmux_session(&session, &slug, &job_id, tx);
            }
        });

        // Terminal tab only on fresh dispatch. On step advance the user
        // already has the tab open; a second `terminal_launcher::launch`
        // opens a duplicate that attaches to the same session.
        if !reuse_session {
            let (strategy, launched) = terminal_launcher::launch(&tmux_session);
            tracing::info!(
                slug = ctx.slug, strategy, launched,
                "job-dispatch: terminal launch attempted"
            );
        } else {
            tracing::info!(
                slug = ctx.slug, session = tmux_session,
                "job-dispatch: reusing existing tmux session, no new terminal tab"
            );
        }

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

/// True when a tmux session with this name already exists.
fn tmux_session_exists(session: &str) -> bool {
    std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Dismiss Claude's development channels confirmation prompt if present.
/// Polls the tmux pane for the confirmation text and presses Enter to accept.
/// Mirrors dispatch.sh's auto_kickstart_tmux behavior.
///
/// The tmux `send-keys` syntax requires the literal key name `Enter` as its
/// own argument — `send-keys "" Enter` sends an empty string *then* Enter,
/// which sometimes confuses the prompt. Use just `Enter` so it's a plain
/// keypress. After accepting, sleep briefly so claude has time to redraw
/// before `wait_for_ready` starts matching.
fn dismiss_channels_prompt(session: &str, timeout_secs: u64) {
    for _ in 0..timeout_secs {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let output = std::process::Command::new("tmux")
            .args(["capture-pane", "-t", session, "-p"])
            .output();
        let Ok(out) = output else { continue };
        let text = String::from_utf8_lossy(&out.stdout);
        if text.contains("I am using this for local development") {
            tracing::info!(session, "dismissing channels confirmation prompt");
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", session, "Enter"])
                .status();
            // Let claude transition from the confirmation screen into the
            // real prompt before wait_for_ready starts matching ❯.
            std::thread::sleep(std::time::Duration::from_secs(1));
            return;
        }
        // Prompt pre-accepted — nothing to do.
        if text.contains("bypass permissions") {
            return;
        }
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
