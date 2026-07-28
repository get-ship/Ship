//! Exit-signal routing — receives a single `ExitSignal` from an executor's
//! monitor thread and reports the session terminal outcome to the host sink.
//!
//! Does **not** touch `KernelRouter`. The host maps
//! [`LifecycleSink::session_completed`] / [`LifecycleSink::session_failed`]
//! onto canonical session events.

use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::dispatch_ports::{ExitCause, ExitSignal, ProcessRegistryPort};
use crate::lifecycle::sink::LifecycleSink;

#[derive(Debug, Clone)]
pub enum ExitVocabulary {
    /// Bare-session dispatch. The session id is the routing key on the host.
    AgentSession { session_id: String },
}

pub fn start(
    sink: Arc<dyn LifecycleSink>,
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
                    handle_agent_exited(&sink, &vocab, &signal).await;
                }
                ExitCause::SessionDestroyed => {
                    handle_session_destroyed(&sink, &vocab, &signal).await;
                }
            }
        }
    });
}

async fn handle_agent_exited(
    sink: &Arc<dyn LifecycleSink>,
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
) {
    let ExitVocabulary::AgentSession { session_id } = vocab;
    let exit_code = signal.exit_code;
    if matches!(exit_code, Some(0)) {
        sink.session_completed(session_id, &signal.slug).await;
    } else {
        let reason = format!(
            "harness exited with code {}",
            exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "?".to_string())
        );
        sink.session_failed(session_id, &signal.slug, reason).await;
    }
}

async fn handle_session_destroyed(
    sink: &Arc<dyn LifecycleSink>,
    vocab: &ExitVocabulary,
    signal: &ExitSignal,
) {
    let ExitVocabulary::AgentSession { session_id } = vocab;
    sink.session_failed(
        session_id,
        &signal.slug,
        "executor session destroyed externally".to_string(),
    )
    .await;
}
