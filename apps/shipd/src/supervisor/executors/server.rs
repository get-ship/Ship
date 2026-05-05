//! ServerExecutor — delegates job execution to a provider's long-lived
//! server-mode session (codex app-server over JSON-RPC, opencode serve
//! over HTTP+SSE).
//!
//! Unlike `ProcessExecutor` and `TmuxWindowExecutor`, which spawn a
//! fresh provider CLI per job, `ServerExecutor` reuses a persistent
//! connection held by a `ServerSession`. The connection survives across
//! jobs; each dispatch translates to a "turn" on the session.
//!
//! This slice is scaffolding: the per-provider `ServerSession` impls
//! land in follow-on slices. The executor is wired to any
//! `Arc<dyn ServerSession>`, so binding the real impl later is a
//! one-line swap in the session registry.
//!
//! Event mapping: `TurnCompleted` -> `ExitSignal{exit_code: 0}`;
//! `Error` -> `ExitSignal{exit_code: 1}`. Other `SessionEvent` variants
//! are logged at debug for now — richer `process.*` / `executor.*`
//! envelope mapping is future work (separate refinement slice), not
//! this scaffolding one.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ship_substrate::events::KernelRouter;
use ship_primitives::services::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, JobContext, JobExecutor,
};
use ship_primitives::services::server_session::{ServerSession, SessionEvent};
use tokio::sync::Mutex as TokioMutex;

/// Default prompt used when `.artifacts/job-spec.md` is missing — mirrors
/// the non-interactive autostart fallback in `ProcessExecutor`.
const FALLBACK_PROMPT: &str = "Read .artifacts/job-spec.md and execute it autonomously.";

/// Job executor that delegates to a provider's server-mode session.
///
/// Selected by `RoutingExecutor` when the provider advertises
/// `caps().server_mode` AND a real session is bound in the registry.
pub(crate) struct ServerExecutor {
    _kernel: Option<Arc<TokioMutex<KernelRouter>>>,
    session: Arc<dyn ServerSession>,
}

impl ServerExecutor {
    pub(crate) fn new(
        kernel: Option<Arc<TokioMutex<KernelRouter>>>,
        session: Arc<dyn ServerSession>,
    ) -> Self {
        Self { _kernel: kernel, session }
    }
}

/// Opaque state stored on the `ExecutorHandle` so `stop`/`is_alive` can
/// find the active turn and its owning session. We clone the `Arc` at
/// spawn-time so the handle is independent of the registry.
pub(super) struct ServerState {
    session: Arc<dyn ServerSession>,
    turn_id: String,
    slug: String,
}

impl ServerState {
    /// Session `Arc` carried by this handle — used by `RoutingExecutor`
    /// to rebuild a transient `ServerExecutor` for handle-dispatched
    /// lifecycle calls (`stop`, `is_alive`, `send`).
    pub(super) fn session_arc(&self) -> Arc<dyn ServerSession> {
        Arc::clone(&self.session)
    }
}

#[async_trait]
impl JobExecutor for ServerExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let prompt = build_prompt(ctx);
        let turn_id = self
            .session
            .start_turn(&prompt)
            .await
            .map_err(|e| anyhow!("server-executor: start_turn failed: {e}"))?;
        let events = self
            .session
            .subscribe(&turn_id)
            .await
            .map_err(|e| anyhow!("server-executor: subscribe failed: {e}"))?;

        let exit_tx = ctx.exit_tx.clone();
        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let provider = self.session.provider().to_string();
        let turn_id_mon = turn_id.clone();
        tokio::spawn(async move {
            translate_events(events, exit_tx, slug, job_id, provider, turn_id_mon).await;
        });

        tracing::info!(
            slug = ctx.slug,
            provider = self.session.provider(),
            turn_id,
            "server-executor: turn started"
        );

        Ok(ExecutorHandle {
            pid: None,
            inner: Box::new(ServerState {
                session: Arc::clone(&self.session),
                turn_id,
                slug: ctx.slug.clone(),
            }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        // The session monitor fires `ExitSignal` on TurnCompleted / Error;
        // liveness here just confirms the handle targets a session we know
        // about. A richer implementation could query the session for the
        // turn's state — per-provider impls can override this.
        handle.inner.downcast_ref::<ServerState>().is_some()
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let Some(state) = handle.inner.downcast_ref::<ServerState>() else {
            return Ok(());
        };
        tracing::info!(
            slug = state.slug,
            turn_id = state.turn_id,
            "server-executor: cancelling turn"
        );
        state.session.cancel(&state.turn_id).await
    }

    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> Result<()> {
        // Server-mode sessions submit new turns for follow-up input;
        // there is no "inject into stdin" equivalent. Mesh events remain
        // the agent-to-agent comm channel.
        Err(anyhow!("ServerExecutor does not support interactive messages"))
    }
}

/// Read `.artifacts/job-spec.md` from the work dir; fall back to a
/// generic instruction if absent. Mirrors `ProcessExecutor`'s default
/// autostart prompt so the two paths submit the same work.
fn build_prompt(ctx: &JobContext) -> String {
    let spec = ctx.work_dir.join(&ctx.spec_path);
    std::fs::read_to_string(&spec).unwrap_or_else(|_| FALLBACK_PROMPT.to_string())
}

/// Drain a `SessionEvent` stream and translate terminal events into an
/// `ExitSignal`. Non-terminal events are logged at debug. Mapping to
/// ship's `process.*` / `executor.*` envelopes is deliberately deferred
/// — this slice pins the plumbing; a refinement slice wires the event
/// vocabulary.
async fn translate_events(
    mut events: tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
    exit_tx: Option<tokio::sync::mpsc::UnboundedSender<ExitSignal>>,
    slug: String,
    job_id: String,
    provider: String,
    turn_id: String,
) {
    while let Some(event) = events.recv().await {
        match event {
            SessionEvent::TurnCompleted { stop_reason } => {
                tracing::info!(slug, provider, turn_id, stop_reason,
                    "server-executor: turn completed");
                if let Some(tx) = &exit_tx {
                    let _ = tx.send(ExitSignal {
                        slug: slug.clone(),
                        job_id: job_id.clone(),
                        exit_code: Some(0),
                        cause: ExitCause::AgentExited,
                    });
                }
                return;
            }
            SessionEvent::Error { message } => {
                tracing::warn!(slug, provider, turn_id, %message,
                    "server-executor: session error");
                if let Some(tx) = &exit_tx {
                    let _ = tx.send(ExitSignal {
                        slug: slug.clone(),
                        job_id: job_id.clone(),
                        exit_code: Some(1),
                        cause: ExitCause::AgentExited,
                    });
                }
                return;
            }
            SessionEvent::TurnStarted { turn_id: t } => {
                tracing::debug!(slug, provider, turn = t, "server-executor: turn started event");
            }
            SessionEvent::AgentMessage { text } => {
                tracing::debug!(slug, provider, text_len = text.len(),
                    "server-executor: agent message");
            }
            SessionEvent::ToolUse { tool, input_preview } => {
                tracing::debug!(slug, provider, tool, ?input_preview,
                    "server-executor: tool use");
            }
            SessionEvent::CommandStarted { cmd } => {
                tracing::debug!(slug, provider, cmd, "server-executor: command started");
            }
            SessionEvent::CommandOutput { stream, line } => {
                tracing::debug!(slug, provider, stream, line_len = line.len(),
                    "server-executor: command output");
            }
        }
    }

    // Channel closed without a terminal event — treat as session
    // destroyed so the liveness sweep can react.
    tracing::warn!(slug, provider, turn_id,
        "server-executor: event stream ended without terminal event");
    if let Some(tx) = &exit_tx {
        let _ = tx.send(ExitSignal {
            slug,
            job_id,
            exit_code: None,
            cause: ExitCause::SessionDestroyed,
        });
    }
}

