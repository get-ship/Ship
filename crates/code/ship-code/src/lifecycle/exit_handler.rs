//! Exit-signal routing — receives a single `ExitSignal` from an executor's
//! monitor thread and emits the canonical session-terminal event.
//!
//! Engine cut 2026-07-10: the `ExitVocabulary::PipelineRun` arm is gone
//! (it emitted `pipeline.run.completed` / `pipeline.run.failed`, both
//! deregistered). Only the bare-session `AgentSession` path survives;
//! the routed terminal event is `session.completed` / `session.failed`.

use std::sync::Arc;

use agent_def::canonical_session::{
    SessionCompleted, SessionFailed, event_types as canonical_session_event_types,
};
use ship_substrate::events::{EventEnvelope, KernelRouter};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::dispatch_ports::{ExitCause, ExitSignal, ProcessRegistryPort};

use ship_substrate::events::emit::persist_and_route;

#[derive(Debug, Clone)]
pub enum ExitVocabulary {
    /// Bare-session dispatch (the only path post-cut). The session id
    /// is the routing key — the terminal event's `entity_id` matches the
    /// session that produced it.
    AgentSession { session_id: String },
}

pub fn start(
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<dyn ProcessRegistryPort>,
    vocab: ExitVocabulary,
    mut exit_rx: UnboundedReceiver<ExitSignal>,
) {
    tokio::spawn(async move {
        if let Some(signal) = exit_rx.recv().await {
            tracing::info!(
                slug = signal.slug,
                exit_code = ?signal.exit_code,
                cause = ?signal.cause,
                ?vocab,
                "exit_handler: signal received",
            );
            registry.unregister(&signal.slug).await;

            match signal.cause {
                ExitCause::AgentExited => {
                    handle_agent_exited(&kernel, &vocab, &signal).await;
                }
                ExitCause::SessionDestroyed => {
                    handle_session_destroyed(&kernel, &vocab, &signal).await;
                }
            }
        }
    });
}

async fn handle_agent_exited(
    kernel: &Arc<Mutex<KernelRouter>>,
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
) {
    let ExitVocabulary::AgentSession { session_id } = vocab;
    let exit_code = signal.exit_code;
    if matches!(exit_code, Some(0)) {
        let payload = SessionCompleted { session_id: session_id.clone() };
        if let Ok(env) = EventEnvelope::new(
            canonical_session_event_types::SESSION_COMPLETED,
            session_id,
            &payload,
        ) {
            persist_and_route(kernel, env, &signal.slug).await;
        }
    } else {
        let reason = format!(
            "harness exited with code {}",
            exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "?".to_string())
        );
        let payload = SessionFailed {
            session_id: session_id.clone(),
            reason,
        };
        if let Ok(env) = EventEnvelope::new(
            canonical_session_event_types::SESSION_FAILED,
            session_id,
            &payload,
        ) {
            persist_and_route(kernel, env, &signal.slug).await;
        }
    }
}

async fn handle_session_destroyed(
    kernel: &Arc<Mutex<KernelRouter>>,
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
) {
    let ExitVocabulary::AgentSession { session_id } = vocab;
    let reason = "executor session destroyed externally".to_string();
    let payload = SessionFailed {
        session_id: session_id.clone(),
        reason,
    };
    if let Ok(env) = EventEnvelope::new(
        canonical_session_event_types::SESSION_FAILED,
        session_id,
        &payload,
    ) {
        persist_and_route(kernel, env, &signal.slug).await;
    }
}
