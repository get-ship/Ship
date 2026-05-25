//! Exit-signal routing — receives a single `ExitSignal` from an executor's
//! monitor thread and emits the corresponding `job.completed` / `job.failed`
//! event. Pure event plumbing; no OS calls. Lives in platform so any
//! orchestrator consumer (today: the daemon) routes through one path.
//!
//! For multi-step pipeline jobs the step monitor takes over the completion
//! pathway, so callers spawn this only for non-pipeline (single-agent) jobs.

use std::sync::Arc;

use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::{EventEnvelope, KernelRouter};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::services::dispatch_ports::{ExitCause, ExitSignal, ProcessRegistryPort};

use super::emit::{emit_executor_session_destroyed, emit_process_exited, persist_and_route};

/// Spawn a task that waits for an exit signal from a process monitor thread.
///
/// For non-pipeline jobs (single agent, no steps), this emits `job.completed`
/// or `job.failed` directly. Multi-step pipeline jobs route through the
/// step monitor instead — see [`super::step_monitor::start_step_monitor`].
pub fn start(
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<dyn ProcessRegistryPort>,
    mut exit_rx: UnboundedReceiver<ExitSignal>,
) {
    tokio::spawn(async move {
        if let Some(signal) = exit_rx.recv().await {
            tracing::info!(
                slug = signal.slug,
                exit_code = ?signal.exit_code,
                cause = ?signal.cause,
                "exit_handler: signal received",
            );
            registry.unregister(&signal.slug).await;

            match signal.cause {
                ExitCause::AgentExited => {
                    let exit_signal = signal
                        .exit_code
                        .and_then(|c| if c != 0 { Some("non-zero") } else { None });
                    emit_process_exited(
                        &kernel,
                        &signal.slug,
                        None,
                        signal.exit_code,
                        exit_signal,
                        0,
                        "unknown",
                    )
                    .await;

                    match signal.exit_code {
                        Some(0) => {
                            let payload = serde_json::json!({
                                "pipeline_run_id": signal.pipeline_run_id, "slug": signal.slug,
                            });
                            emit_job_event(
                                &kernel,
                                event_types::PIPELINE_RUN_COMPLETED,
                                &signal.pipeline_run_id,
                                &signal.slug,
                                payload,
                            )
                            .await;
                        }
                        Some(code) => {
                            let payload = serde_json::json!({
                                "pipeline_run_id": signal.pipeline_run_id,
                                "error": format!("process exited with code {code}"),
                            });
                            emit_job_event(
                                &kernel,
                                event_types::PIPELINE_RUN_FAILED,
                                &signal.pipeline_run_id,
                                &signal.slug,
                                payload,
                            )
                            .await;
                        }
                        None => {
                            // Process gone, no exit code captured — treat as
                            // failure. Pre-slice this path silently emitted
                            // job.completed; that was the bug.
                            let payload = serde_json::json!({
                                "pipeline_run_id": signal.pipeline_run_id,
                                "error": "agent process exited with no captured exit code",
                            });
                            emit_job_event(
                                &kernel,
                                event_types::PIPELINE_RUN_FAILED,
                                &signal.pipeline_run_id,
                                &signal.slug,
                                payload,
                            )
                            .await;
                        }
                    }
                }
                ExitCause::SessionDestroyed => {
                    emit_executor_session_destroyed(
                        &kernel,
                        &signal.slug,
                        "tmux_window",
                        &signal.slug,
                        DestructionCause::External,
                    )
                    .await;
                    let payload = serde_json::json!({
                        "pipeline_run_id": signal.pipeline_run_id,
                        "error": "executor session destroyed externally (operator kill or substrate failure)",
                    });
                    emit_job_event(
                        &kernel,
                        event_types::PIPELINE_RUN_FAILED,
                        &signal.pipeline_run_id,
                        &signal.slug,
                        payload,
                    )
                    .await;
                }
            }
        }
    });
}

async fn emit_job_event(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    pipeline_run_id: &str,
    slug: &str,
    payload: serde_json::Value,
) {
    let envelope = match EventEnvelope::new(event_type, pipeline_run_id, &payload) {
        Ok(e) => e.with_actor_id("service.exit-handler"),
        Err(e) => {
            tracing::warn!(slug, event_type, "exit_handler: failed to build envelope: {e}");
            return;
        }
    };
    persist_and_route(kernel, envelope, slug, None).await;
}
