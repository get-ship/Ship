//! Job event router — dispatches kernel events to the appropriate handler.
//!
//! This is the entry point for all `job.*` and `step.*` events flowing
//! through the orchestrator.

use crate::events::job::StepObservedPayload;
use crate::events::EventEnvelope;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch::handle_job_created;
use super::gate_retry::{emit_job_failed_for_gate, try_gate_retry};
use super::handlers::{finalize_pipeline_if_done, handle_job_cleanup, handle_job_update};
use super::pipeline::advance_pipeline;
use super::OrchestratorContext;

/// Bootstrap the orchestrator: start liveness monitoring, reconcile state,
/// and begin processing events from the mailbox.
///
/// Called once during daemon startup. Returns when the mailbox closes.
pub async fn start_orchestrator(
    kernel: Arc<Mutex<crate::events::KernelRouter>>,
    octx: Arc<OrchestratorContext>,
) {
    let actor_id = "service.job-dispatch".to_string();
    let config = crate::events::ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec!["job.".to_string(), "step.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
    };

    let mailbox = {
        let mut k = kernel.lock().await;
        match k.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => mb,
            Err(e) => {
                tracing::warn!("job-dispatch: failed to spawn actor: {e}");
                return;
            }
        }
    };

    // Start the periodic liveness sweep.
    octx.platform.start_liveness_sweep(kernel.clone());

    // On startup: reconcile stale jobs, replay completions, dispatch pending.
    let replay_kernel = kernel.clone();
    let replay_octx = octx.clone();
    tokio::spawn(async move {
        super::replay::reconcile_dispatched_jobs(&replay_kernel, &replay_octx).await;
        super::replay::replay_unprocessed_completions(&replay_kernel, &replay_octx).await;
        super::pending::dispatch_ready_pending_jobs(&replay_kernel, &replay_octx).await;
        replay_octx.platform.cleanup_orphaned_worktrees().await;
    });

    let kr = kernel.clone();
    let mut mb = mailbox;
    while let Some(envelope) = mb.recv().await {
        handle_job_event(&kr, &envelope, &octx).await;
    }
    tracing::info!("job-dispatch: mailbox closed");
}

/// Handle a single job/step event.
///
/// Called from the mailbox receive loop for each `job.*` or `step.*` event.
pub async fn handle_job_event(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: &EventEnvelope,
    octx: &OrchestratorContext,
) {
    match envelope.event_type.as_str() {
        "job.created" => handle_job_created(kernel, envelope, octx).await,
        "job.update" => handle_job_update(kernel, envelope).await,
        "job.completed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.completed");
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
            }
            if !advance_pipeline(kernel, envelope, octx).await {
                octx.job_locks.lock().await.remove(&envelope.entity_id);
                super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
            }
        }
        "job.step_completed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.step_completed");
            if !advance_pipeline(kernel, envelope, octx).await {
                finalize_pipeline_if_done(kernel, envelope).await;
                super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
            }
        }
        "job.step_failed" => {
            tracing::warn!(entity_id = %envelope.entity_id, "job-dispatch: pipeline step failed — advancement halted");
            let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).unwrap_or_default();
            if let Some(slug) = payload["slug"].as_str() {
                octx.registry.unregister(slug).await;
            }
        }
        "job.merged" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.merged");
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
            }
            octx.job_locks.lock().await.remove(&envelope.entity_id);
            handle_job_cleanup(envelope, &*octx.platform).await;
            super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
        }
        "job.failed" => {
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.registry.unregister(slug).await;
                octx.done_flags.lock().await.remove(slug.as_str());
                octx.activity_txs.lock().await.remove(slug.as_str());
            }
            octx.job_locks.lock().await.remove(&envelope.entity_id);
        }
        "step.done_declared" => {
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                if let Some(flag) = octx.done_flags.lock().await.get(slug.as_str()) {
                    flag.store(true, Ordering::Release);
                    tracing::info!(slug, "job-dispatch: step_done declared");
                } else {
                    tracing::warn!(slug, "job-dispatch: step.done_declared but no active monitor");
                }
            }
        }
        "step.observed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling step.observed");
            let payload: StepObservedPayload = match serde_json::from_str(&envelope.payload_json) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("job-dispatch: malformed step.observed payload: {e}");
                    return;
                }
            };
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                octx.done_flags.lock().await.remove(slug.as_str());
                octx.activity_txs.lock().await.remove(slug.as_str());
            }

            use crate::events::job::StepVerdict;
            match &payload.verdict {
                StepVerdict::Advance => {
                    tracing::info!(
                        entity_id = %envelope.entity_id,
                        step = payload.step_name,
                        trigger = ?payload.trigger,
                        commits = payload.commits_since_start,
                        done_declared = payload.done_declared,
                        "job-dispatch: step evidence → advance"
                    );
                    if !advance_pipeline(kernel, envelope, octx).await {
                        finalize_pipeline_if_done(kernel, envelope).await;
                        super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
                    }
                }
                StepVerdict::Inconclusive => {
                    // No strong signal either way. Advance rather than hang —
                    // the gate step (if any) will catch problems.
                    tracing::warn!(
                        entity_id = %envelope.entity_id,
                        step = payload.step_name,
                        trigger = ?payload.trigger,
                        commits = payload.commits_since_start,
                        done_declared = payload.done_declared,
                        "job-dispatch: step evidence inconclusive — advancing (gate will review)"
                    );
                    if !advance_pipeline(kernel, envelope, octx).await {
                        finalize_pipeline_if_done(kernel, envelope).await;
                        super::pending::dispatch_ready_pending_jobs(kernel, octx).await;
                    }
                }
                StepVerdict::Failed => {
                    tracing::warn!(
                        entity_id = %envelope.entity_id,
                        step = payload.step_name,
                        trigger = ?payload.trigger,
                        reason = ?payload.reason,
                        ?payload.exit_code,
                        "job-dispatch: step failed"
                    );
                    if let Some(slug) = &slug {
                        octx.registry.unregister(slug).await;
                    }
                    if !try_gate_retry(kernel, octx, &payload).await {
                        emit_job_failed_for_gate(kernel, &payload).await;
                    }
                }
            }
        }
        _ => {}
    }
}

/// Extract the slug from an event envelope payload.
fn extract_slug(envelope: &EventEnvelope) -> Option<String> {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).ok()?;
    payload["slug"].as_str().map(str::to_string)
}
