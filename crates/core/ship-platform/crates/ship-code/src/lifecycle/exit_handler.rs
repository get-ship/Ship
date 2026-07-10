//! Exit-signal routing — receives a single `ExitSignal` from an executor's
//! monitor thread and emits the corresponding terminal event.


use std::sync::Arc;

use agent_def::canonical_session::{
    SessionCompleted, SessionFailed, event_types as canonical_session_event_types,
};
use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::pipeline::event_types as pipeline_event_types;
use ship_substrate::events::{EventEnvelope, KernelRouter};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::dispatch_ports::{ExitCause, ExitSignal, ProcessRegistryPort};

use ship_substrate::events::emit::{
    emit_executor_session_destroyed, emit_executor_window_destroyed, emit_process_exited,
    persist_and_route,
};

#[derive(Debug, Clone)]
pub enum ExitVocabulary {
    PipelineRun,
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
    if matches!(vocab, ExitVocabulary::PipelineRun) {
        let exit_signal = signal
            .exit_code
            .and_then(|c| if c != 0 { Some("non-zero") } else { None });
        emit_process_exited(
            kernel,
            &signal.slug,
            None,
            signal.exit_code,
            exit_signal,
            0,
            "unknown",
        )
        .await;
    }

    let terminal = match signal.exit_code {
        Some(0) => build_completed(vocab, signal),
        Some(code) => {
            build_failed(vocab, signal, &failed_message(vocab, code), Some(code))
        }
        None => {
            build_failed(vocab, signal, no_code_message(vocab), None)
        }
    };
    emit_terminal(kernel, &signal.slug, terminal).await;
}

async fn handle_session_destroyed(
    kernel: &Arc<Mutex<KernelRouter>>,
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
) {
    if matches!(vocab, ExitVocabulary::PipelineRun) {
        emit_executor_window_destroyed(
            kernel,
            &signal.slug,
            "tmux_window",
            &signal.slug,
            &signal.slug,
            None,
            DestructionCause::External,
        )
        .await;
        emit_executor_session_destroyed(
            kernel,
            &signal.slug,
            "tmux_window",
            &signal.slug,
            DestructionCause::External,
        )
        .await;
    }

    let terminal = build_failed(
        vocab,
        signal,
        session_destroyed_message(vocab),
        None,
    );
    emit_terminal(kernel, &signal.slug, terminal).await;
}

fn failed_message(vocab: &ExitVocabulary, code: i32) -> String {
    match vocab {
        ExitVocabulary::PipelineRun => format!("process exited with code {code}"),
        ExitVocabulary::AgentSession { .. } => format!("harness exited with code {code}"),
    }
}

fn no_code_message(vocab: &ExitVocabulary) -> &'static str {
    match vocab {
        ExitVocabulary::PipelineRun => "agent process exited with no captured exit code",
        ExitVocabulary::AgentSession { .. } => "harness exited with no captured exit code",
    }
}

fn session_destroyed_message(vocab: &ExitVocabulary) -> &'static str {
    match vocab {
        ExitVocabulary::PipelineRun => {
            "executor session destroyed externally (operator kill or substrate failure)"
        }
        ExitVocabulary::AgentSession { .. } => "executor session destroyed externally",
    }
}

struct TerminalEvent {
    event_type: &'static str,
    entity_id: String,
    payload: serde_json::Value,
    actor_id: &'static str,
}

fn build_completed(vocab: &ExitVocabulary, signal: &ExitSignal) -> TerminalEvent {
    match vocab {
        ExitVocabulary::PipelineRun => TerminalEvent {
            event_type: pipeline_event_types::PIPELINE_RUN_COMPLETED,
            entity_id: signal.pipeline_run_id.clone(),
            payload: serde_json::json!({
                "pipeline_run_id": signal.pipeline_run_id, "slug": signal.slug,
            }),
            actor_id: "service.exit-handler",
        },
        ExitVocabulary::AgentSession { session_id } => {
            // Canonical: exit_code=0 → session.completed; no exit_code field in payload.
            let payload = SessionCompleted { session_id: session_id.clone() };
            TerminalEvent {
                event_type: canonical_session_event_types::SESSION_COMPLETED,
                entity_id: session_id.clone(),
                payload: serde_json::to_value(&payload).unwrap_or_default(),
                actor_id: "service.agent-session-dispatch",
            }
        }
    }
}

fn build_failed(
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
    error: &str,
    exit_code: Option<i32>,
) -> TerminalEvent {
    match vocab {
        ExitVocabulary::PipelineRun => TerminalEvent {
            event_type: pipeline_event_types::PIPELINE_RUN_FAILED,
            entity_id: signal.pipeline_run_id.clone(),
            payload: serde_json::json!({
                "pipeline_run_id": signal.pipeline_run_id,
                "error": error,
            }),
            actor_id: "service.exit-handler",
        },
        ExitVocabulary::AgentSession { session_id } => {
            // Canonical: error + stage collapse into a single reason string;
            // exit_code is not carried in the canonical payload.
            let reason = if let Some(code) = exit_code {
                format!("{error} (exit {code})")
            } else {
                error.to_string()
            };
            let payload = SessionFailed { session_id: session_id.clone(), reason };
            TerminalEvent {
                event_type: canonical_session_event_types::SESSION_FAILED,
                entity_id: session_id.clone(),
                payload: serde_json::to_value(&payload).unwrap_or_default(),
                actor_id: "service.agent-session-dispatch",
            }
        }
    }
}

async fn emit_terminal(
    kernel: &Arc<Mutex<KernelRouter>>,
    slug: &str,
    terminal: TerminalEvent,
) {
    let envelope = match EventEnvelope::new(terminal.event_type, &terminal.entity_id, &terminal.payload) {
        Ok(e) => e.with_actor_id(terminal.actor_id),
        Err(e) => {
            tracing::warn!(slug, event_type = terminal.event_type, "exit_handler: failed to build envelope: {e}");
            return;
        }
    };
    persist_and_route(kernel, envelope, slug).await;
}

#[cfg(test)]
#[path = "exit_handler_tests.rs"]
mod tests;
