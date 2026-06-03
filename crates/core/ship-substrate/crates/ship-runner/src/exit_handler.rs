//! Exit-signal routing — receives a single `ExitSignal` from an executor's
//! monitor thread and emits the corresponding terminal event for whichever
//! lifecycle vocabulary the caller selected. Pure event plumbing; no OS calls.
//! Mechanism only: it routes through the substrate `KernelRouter` via the L0
//! `emit` helpers and depends only on the domain-free runner ports.
//!
//! # One mechanism, two vocabularies
//!
//! The exit-classification skeleton — *receive the signal, unregister the
//! handle, branch on [`ExitCause`], then on the exit code* — is identical for
//! every dispatch shape. What differs is the terminal **event vocabulary**: a
//! pipeline run emits `pipeline.run.{completed,failed}` keyed on its
//! `pipeline_run_id`; a bare agent session emits
//! `agent.session.{completed,failed}` keyed on its `session_id`. Event
//! vocabulary is therefore a *parameter* ([`ExitVocabulary`]), not a separate
//! handler per layer.
//!
//! For multi-step pipeline jobs the step monitor takes over the completion
//! pathway, so callers spawn this only for non-pipeline (single-agent) jobs
//! and for bare agent sessions.

use std::sync::Arc;

use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionCompletedPayload,
    AgentSessionFailedPayload,
};
use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::pipeline::event_types as pipeline_event_types;
use ship_substrate::events::{EventEnvelope, KernelRouter};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;

use ship_substrate::services::dispatch_ports::{ExitCause, ExitSignal, ProcessRegistryPort};

use super::emit::{
    emit_executor_session_destroyed, emit_executor_window_destroyed, emit_process_exited,
    persist_and_route,
};

/// Which terminal-event vocabulary the exit handler emits.
///
/// One [`start`] implementation serves every dispatch shape; the caller picks
/// the vocabulary that matches what it dispatched. The variant also carries the
/// entity id the terminal events are keyed on — `pipeline_run_id` for pipeline
/// runs, `session_id` for bare agent sessions — because the two id spaces are
/// distinct even though both arrive on the same [`ExitSignal`].
#[derive(Debug, Clone)]
pub enum ExitVocabulary {
    /// Emit `pipeline.run.{completed,failed}` keyed on the run id, plus the
    /// `process.exited` / `executor.*_destroyed` substrate trail. Used by the
    /// non-pipeline (single-agent) pipeline-run dispatch path. The run id is
    /// read from the signal (`ExitSignal::pipeline_run_id`).
    PipelineRun,
    /// Emit `agent.session.{completed,failed}` keyed on `session_id`. Used by
    /// the bare agent-session dispatch path. The session id is carried here
    /// because [`ExitSignal::pipeline_run_id`] is reused by the session route
    /// as the routing ulid, but the terminal events must key on the canonical
    /// `session_id` the projection indexes on.
    AgentSession { session_id: String },
}

/// Spawn a task that waits for an exit signal from a process monitor thread.
///
/// For non-pipeline jobs (single agent, no steps) and bare agent sessions this
/// emits the terminal completed/failed pair for the selected [`ExitVocabulary`].
/// Multi-step pipeline jobs route through the step monitor instead.
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

/// The agent process exited — classify on the captured exit code.
async fn handle_agent_exited(
    kernel: &Arc<Mutex<KernelRouter>>,
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
) {
    // The pipeline vocabulary records the raw `process.exited` substrate trail
    // before the lifecycle verdict; the agent-session vocabulary does not (its
    // observability comes solely from the agent.session.* events).
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
            build_failed(vocab, signal, &failed_message(vocab, code), Some(code), "exit_nonzero")
        }
        None => {
            // Process gone, no exit code captured — treat as failure. The
            // pipeline path historically emitted completed here; that was the
            // bug the slice that introduced this handler fixed.
            build_failed(vocab, signal, no_code_message(vocab), None, "exit_no_code")
        }
    };
    emit_terminal(kernel, &signal.slug, terminal).await;
}

/// The executor substrate went away without us observing the agent exit.
async fn handle_session_destroyed(
    kernel: &Arc<Mutex<KernelRouter>>,
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
) {
    // The pipeline vocabulary records the executor-teardown trail; the
    // agent-session vocabulary reports the cause via the failed event alone.
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
        "session_destroyed",
    );
    emit_terminal(kernel, &signal.slug, terminal).await;
}

/// The completed-event text differs only by vocabulary; the failed-event text
/// is built per-cause by the helpers below.
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

/// A fully-resolved terminal event: the substrate vocabulary's event type, the
/// entity it keys on, the payload, and the owning service actor. Building this
/// is the *whole* of the per-vocabulary divergence, so it is a pure function —
/// the tests assert it for BOTH vocabularies without a live kernel.
struct TerminalEvent {
    event_type: &'static str,
    entity_id: String,
    payload: serde_json::Value,
    actor_id: &'static str,
}

/// Build the completed terminal for the selected vocabulary.
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
            let payload = AgentSessionCompletedPayload {
                session_id: session_id.clone(),
                slug: signal.slug.clone(),
                exit_code: signal.exit_code,
            };
            TerminalEvent {
                event_type: agent_session_event_types::AGENT_SESSION_COMPLETED,
                entity_id: session_id.clone(),
                payload: serde_json::to_value(&payload).unwrap_or_default(),
                actor_id: "service.agent-session-dispatch",
            }
        }
    }
}

/// Build the failed terminal for the selected vocabulary. `stage` is the coarse
/// agent-session classifier; the pipeline vocabulary ignores it.
fn build_failed(
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
    error: &str,
    exit_code: Option<i32>,
    stage: &str,
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
            let payload = AgentSessionFailedPayload {
                session_id: session_id.clone(),
                slug: signal.slug.clone(),
                error: error.to_string(),
                exit_code,
                stage: stage.to_string(),
            };
            TerminalEvent {
                event_type: agent_session_event_types::AGENT_SESSION_FAILED,
                entity_id: session_id.clone(),
                payload: serde_json::to_value(&payload).unwrap_or_default(),
                actor_id: "service.agent-session-dispatch",
            }
        }
    }
}

/// Build the terminal envelope and route it. Thin wrapper over the pure
/// [`TerminalEvent`] builder — the routing is the only impure part.
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
    persist_and_route(kernel, envelope, slug, None).await;
}

#[cfg(test)]
#[path = "exit_handler_tests.rs"]
mod tests;
