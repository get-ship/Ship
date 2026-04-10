//! Job dispatch subscriber — reacts to job lifecycle events from the kernel.
//!
//! When `job.created` flows through the kernel, prepares isolation and spawns
//! the agent via injected trait objects. When `job.completed` arrives, advances
//! the pipeline DAG. When `job.merged` arrives, cleans up resources.

use runtime::events::job::StepObservedPayload;
use runtime::events::{ActorConfig, EventEnvelope};
use runtime::services::dispatch_ports::{IsolationStrategy, JobExecutor};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::adapters;
use super::dispatch_handlers::{finalize_pipeline_if_done, handle_job_cleanup, handle_job_update};
pub(crate) use super::dispatch_initial::handle_job_created;
pub(crate) use super::dispatch_initial::dispatch_job_internal;
use super::dispatch_pipeline::advance_pipeline;
pub(crate) use super::registry::ProcessRegistry;
#[cfg(test)]
pub(crate) use super::dispatch_pipeline::advance_pipeline_at;
#[cfg(test)]
pub(crate) use super::dispatch_replay::reconcile_dispatched_jobs_at;
#[cfg(test)]
pub(crate) use super::graceful_shutdown;

/// Bundles the injected isolation and execution strategies for dispatch.
#[derive(Clone)]
pub(crate) struct DispatchContext {
    pub isolation: Arc<dyn IsolationStrategy>,
    pub executor: Arc<dyn JobExecutor>,
    pub registry: Arc<Mutex<ProcessRegistry>>,
    /// Maps `slug` → done_declared flag for active step monitors.
    pub done_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

/// Handle returned by `subscribe_job_events` — gives the shutdown path
/// access to the process registry and executor for graceful shutdown.
pub struct DispatchHandle {
    pub(crate) registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn JobExecutor>,
}

/// Subscribe to `job.*` kernel events and dispatch jobs to worktrees.
///
/// Returns a `DispatchHandle` that the shutdown path uses to drain
/// all managed agents on SIGTERM/SIGINT.
pub async fn subscribe_job_events(
    kernel: Arc<Mutex<runtime::events::KernelRouter>>,
    _ship_dir: PathBuf,
) -> Option<DispatchHandle> {
    let actor_id = "service.job-dispatch".to_string();
    let config = ActorConfig {
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
                return None;
            }
        }
    };

    let executor: Arc<dyn runtime::services::dispatch_ports::JobExecutor> =
        if std::env::var("SHIP_EXECUTOR").as_deref() == Ok("process") {
            tracing::info!("job-dispatch: using ProcessExecutor");
            Arc::new(super::executors::ProcessExecutor)
        } else {
            tracing::info!("job-dispatch: using TmuxExecutor");
            Arc::new(super::executors::TmuxExecutor)
        };

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    let dctx = Arc::new(DispatchContext {
        isolation: Arc::new(adapters::GitWorktreeIsolation),
        executor: executor.clone(),
        registry: registry.clone(),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
    });

    // Start the periodic liveness sweep (AC2 + AC3).
    super::liveness::start_liveness_sweep(kernel.clone(), registry.clone(), executor.clone());

    // On startup: reconcile stale dispatched jobs, replay completions, dispatch pending.
    let replay_kernel = kernel.clone();
    let replay_dctx = dctx.clone();
    tokio::spawn(async move {
        super::dispatch_replay::reconcile_dispatched_jobs(&replay_kernel, &*replay_dctx).await;
        super::dispatch_replay::replay_unprocessed_completions(&replay_kernel, &*replay_dctx).await;
        super::pending_dispatch::dispatch_ready_pending_jobs(&replay_kernel, &*replay_dctx).await;
        super::orphan_cleanup::cleanup_orphaned_worktrees().await;
    });

    let kr = kernel.clone();
    tokio::spawn(async move {
        let mut mb = mailbox;
        while let Some(envelope) = mb.recv().await {
            handle_job_event(&kr, &envelope, &dctx).await;
        }
        tracing::info!("job-dispatch: mailbox closed");
    });

    Some(DispatchHandle { registry, executor })
}

pub(crate) async fn handle_job_event(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
) {
    match envelope.event_type.as_str() {
        "job.created" => handle_job_created(kernel, envelope, dctx).await,
        "job.update" => handle_job_update(kernel, envelope).await,
        "job.completed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.completed");
            // Unregister completed agent from process registry
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                dctx.registry.lock().await.unregister(slug);
            }
            if !advance_pipeline(kernel, envelope, dctx).await {
                // Pipeline done or no pipeline — scan projection for unblocked jobs
                super::pending_dispatch::dispatch_ready_pending_jobs(kernel, dctx).await;
            }
        }
        "job.step_completed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.step_completed");
            if !advance_pipeline(kernel, envelope, dctx).await {
                // No more steps to dispatch — finalize the job if all steps are done
                finalize_pipeline_if_done(kernel, envelope).await;
                super::pending_dispatch::dispatch_ready_pending_jobs(kernel, dctx).await;
            }
        }
        "job.step_failed" => {
            tracing::warn!(entity_id = %envelope.entity_id, "job-dispatch: pipeline step failed — advancement halted");
            let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).unwrap_or_default();
            if let Some(slug) = payload["slug"].as_str() {
                dctx.registry.lock().await.unregister(slug);
            }
        }
        "job.merged" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.merged");
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                dctx.registry.lock().await.unregister(slug);
            }
            handle_job_cleanup(envelope).await;
            // Scan projection for jobs whose deps are now resolved
            super::pending_dispatch::dispatch_ready_pending_jobs(kernel, dctx).await;
        }
        "job.failed" => {
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                dctx.registry.lock().await.unregister(slug);
                dctx.done_flags.lock().await.remove(slug.as_str());
            }
        }
        "step.done_declared" => {
            // Agent called step_done — set the done flag for the step monitor
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                if let Some(flag) = dctx.done_flags.lock().await.get(slug.as_str()) {
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
            // Clean up done flag
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                dctx.done_flags.lock().await.remove(slug.as_str());
            }
            if payload.succeeded {
                if !advance_pipeline(kernel, envelope, dctx).await {
                    finalize_pipeline_if_done(kernel, envelope).await;
                    super::pending_dispatch::dispatch_ready_pending_jobs(kernel, dctx).await;
                }
            } else {
                tracing::warn!(
                    entity_id = %envelope.entity_id,
                    step = payload.step_name,
                    reason = ?payload.reason,
                    "job-dispatch: step failed"
                );
                // Unregister if still present
                if let Some(slug) = &slug {
                    dctx.registry.lock().await.unregister(slug);
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

#[cfg(test)]
#[path = "tests_job_dispatch.rs"]
mod tests;
