//! TmuxExecutor — spawns agents in named tmux sessions.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::KernelRouter;
use ship_platform::services::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, JobContext, JobExecutor,
};
use ship_platform::services::orchestrator::emit::emit_executor_session_destroyed;
use ship_platform::services::provider_adapter::adapter_for_or_default;
use tokio::sync::Mutex;

use super::super::helpers::{ensure_tmux_session, kill_tmux_session};
use super::super::terminal_launcher;

/// Job executor that spawns agents in tmux sessions.
///
/// `kernel` is optional so unit tests without a runtime kernel can
/// construct this executor directly. Emission is skipped when `None`.
pub(crate) struct TmuxExecutor {
    kernel: Option<Arc<Mutex<KernelRouter>>>,
}

impl TmuxExecutor {
    pub(crate) fn new(kernel: Option<Arc<Mutex<KernelRouter>>>) -> Self {
        Self { kernel }
    }
}

/// State held in the ExecutorHandle for tmux-based execution.
struct TmuxState {
    slug: String,
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

        let adapter = adapter_for_or_default(ctx.launch.provider.as_deref())
            .map_err(|e| anyhow::anyhow!("tmux executor: {e}"))?;
        let cmd = adapter.build_interactive(&ctx.slug, &ctx.launch);
        let shell_cmd = cmd.to_shell_string();

        // Clear residual output from any prior agent before the new one
        // launches. On step advance the pane still shows "bypass permissions
        // on" from the previous claude's prompt — which matches the ready
        // pattern and makes drive_agent_to_ready fire autostart into the
        // shell before the new claude even starts. `clear` + clear-history
        // gives a clean slate.
        if reuse_session {
            let _ = crate::spawn_bounded::Cmd::new("tmux")
                .args(["send-keys", "-t", &tmux_session, "clear", "C-m"])
                .status();
            let _ = crate::spawn_bounded::Cmd::new("tmux")
                .args(["clear-history", "-t", &tmux_session])
                .status();
        }

        let result = crate::spawn_bounded::Cmd::new("tmux")
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
            drive_agent_to_ready(&session, &ready_pat, &autostart);
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
                slug: ctx.slug.clone(),
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
        crate::spawn_bounded::Cmd::new("tmux")
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
        // Unknown-provider falls through to the force-kill path at the end.
        let exit_command = adapter_for_or_default(state.provider.as_deref())
            .map(|a| a.exit_command().to_string())
            .unwrap_or_default();
        let _ = crate::spawn_bounded::Cmd::new("tmux")
            .args(["send-keys", "-t", &state.session, &exit_command, "Enter"])
            .status();
        tracing::debug!(
            session = %state.session,
            "tmux-executor: sent exit_command, waiting up to 30s"
        );

        // Wait up to 30s for the session to disappear.
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let alive = crate::spawn_bounded::Cmd::new("tmux")
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
        let result = kill_tmux_session(&state.session);

        // Destruction is the intent regardless of whether kill_tmux_session
        // returned Ok (session killed) or Err (already gone). Either way the
        // operator-facing fact is that the session is no longer live.
        if let Some(kernel) = self.kernel.as_ref() {
            emit_executor_session_destroyed(
                kernel,
                &state.slug,
                "tmux",
                &state.session,
                DestructionCause::External,
            )
            .await;
        }
        result
    }

    async fn resume_handle(
        &self,
        slug: &str,
        provider: Option<&str>,
        _current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        let session = format!("job-{slug}");
        if !tmux_session_exists(&session) {
            return None;
        }
        Some(ExecutorHandle {
            pid: None,
            inner: Box::new(TmuxState {
                slug: slug.to_string(),
                session,
                provider: provider.map(str::to_string),
            }),
        })
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let state = match handle.inner.downcast_ref::<TmuxState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let status = crate::spawn_bounded::Cmd::new("tmux")
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
    crate::spawn_bounded::Cmd::new("tmux")
        .args(["has-session", "-t", session])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Drive the agent from "just launched" to "ready and autostarted" by polling
/// the tmux pane and reacting to every state we see along the way.
///
/// Replaces the previous two-stage `dismiss_channels_prompt` → `wait_for_ready`
/// that raced and timed out independently. Those were brittle:
///
///   - claude can take 20+ seconds to render the channels menu on a second
///     launch in an existing shell, past the old 15s dismiss window
///   - `❯` appears both in the menu and at the real prompt, so a naive
///     `wait_for_ready("❯")` false-matched the menu
///   - dismiss sent `tmux send-keys Enter` which claude's menu widget ignores;
///     only `C-m` (literal carriage return) confirms the selection
///
/// Single loop, one deadline, re-evaluate every second:
///   - if the channels menu is on screen → send `C-m`, continue polling
///   - if `ready_pat` shows AND the menu text is gone → send autostart, done
///   - timeout → log and bail, monitor_tmux_session still takes over
///
/// We poll pane state every 1s for up to `READY_DEADLINE`. The agent typically
/// transitions within 5-15s on fresh launch and 15-30s on reuse.
fn drive_agent_to_ready(session: &str, ready_pat: &str, autostart: &str) {
    const READY_DEADLINE: u64 = 60;
    const MENU_TEXT: &str = "I am using this for local development";

    for tick in 0..READY_DEADLINE {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let Ok(out) = crate::spawn_bounded::Cmd::new("tmux")
            .args(["capture-pane", "-t", session, "-p"])
            .output()
        else {
            continue;
        };
        let text = String::from_utf8_lossy(&out.stdout);

        // Channels menu on screen → dismiss, keep polling for the real prompt.
        if text.contains(MENU_TEXT) {
            tracing::info!(session, tick, "agent: dismissing channels confirmation");
            let _ = crate::spawn_bounded::Cmd::new("tmux")
                .args(["send-keys", "-t", session, "C-m"])
                .status();
            // Give claude a beat to redraw before the next poll matches the
            // ready pattern — otherwise we might send autostart into a
            // transitioning UI.
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        // Real prompt visible AND menu gone → send autostart and we're done.
        if text.contains(ready_pat) {
            tracing::info!(session, tick, "agent: ready, sending autostart");
            let _ = crate::spawn_bounded::Cmd::new("tmux")
                .args(["send-keys", "-t", session, autostart, "Enter"])
                .status();
            return;
        }
    }

    tracing::warn!(
        session,
        "agent: did not reach ready state within {READY_DEADLINE}s — \
         autostart was not sent, pipeline may stall"
    );
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
        let alive = crate::spawn_bounded::Cmd::new("tmux")
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
                cause: ExitCause::SessionDestroyed,
            });
            return;
        }
    }
}
