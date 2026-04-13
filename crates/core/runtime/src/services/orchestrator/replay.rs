//! Replay and reconciliation for job dispatch.
//!
//! On daemon startup:
//! 1. Replays stale `job.completed` events that arrived while daemon was down
//! 2. Reconciles jobs stuck in `Dispatched` state after a daemon restart

use crate::events::EventEnvelope;
use crate::projections::job::{load_jobs, JobStatus};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::pipeline::advance_pipeline;
use super::OrchestratorContext;

/// Replay unprocessed `job.completed` events that arrived while the daemon was down.
pub async fn replay_unprocessed_completions(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    octx: &OrchestratorContext,
) {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: replay failed to load jobs: {e}");
            return;
        }
    };

    let stale: Vec<_> = jobs
        .values()
        .filter(|j| j.status == JobStatus::Completed)
        .collect();

    if stale.is_empty() {
        tracing::debug!("job-dispatch: no stale completed jobs to replay");
        return;
    }

    tracing::info!(count = stale.len(), "job-dispatch: replaying stale job.completed events");

    for job in stale {
        let payload = serde_json::json!({
            "job_id": job.job_id,
            "slug": job.slug,
        });
        let envelope = match EventEnvelope::new(
            crate::events::job::event_types::JOB_COMPLETED,
            &job.job_id,
            &payload,
        ) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(slug = job.slug, "job-dispatch: replay failed to build envelope: {e}");
                continue;
            }
        };

        tracing::info!(slug = job.slug, job_id = job.job_id, "job-dispatch: replaying job.completed");

        if !advance_pipeline(kernel, &envelope, octx).await {
            tracing::info!(slug = job.slug, "job-dispatch: replay — no pipeline to advance, awaiting gate/merge");
        }
    }

    tracing::info!("job-dispatch: replay complete");
}

/// Reconcile jobs stuck in `Dispatched` state after daemon restart.
pub async fn reconcile_dispatched_jobs(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    octx: &OrchestratorContext,
) {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: reconcile failed to load jobs: {e}");
            return;
        }
    };

    reconcile_dispatched_inner(kernel, octx, jobs, None).await;
}

/// Path-aware variant for tests.
pub async fn reconcile_dispatched_jobs_at(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    octx: &OrchestratorContext,
    ship_dir: &std::path::Path,
) {
    let jobs = match crate::projections::job::load_jobs_at(ship_dir) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: reconcile failed to load jobs: {e}");
            return;
        }
    };

    reconcile_dispatched_inner(kernel, octx, jobs, Some(ship_dir)).await;
}

async fn reconcile_dispatched_inner(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    octx: &OrchestratorContext,
    jobs: std::collections::HashMap<String, crate::projections::job::JobRecord>,
    store_dir: Option<&std::path::Path>,
) {
    let stale: Vec<_> = jobs
        .values()
        .filter(|j| j.status == JobStatus::Dispatched)
        .collect();

    if stale.is_empty() {
        tracing::debug!("job-dispatch: no stale dispatched jobs to reconcile");
        return;
    }

    tracing::info!(count = stale.len(), "job-dispatch: reconciling stale dispatched jobs");

    for job in stale {
        let handle = crate::services::dispatch_ports::ExecutorHandle {
            pid: None,
            inner: Box::new(()),
        };

        if octx.executor.is_alive(&handle).await {
            tracing::info!(slug = job.slug, "job-dispatch: reconcile — process still alive, re-attaching");
            let resume_handle = crate::services::dispatch_ports::ExecutorHandle {
                pid: None,
                inner: Box::new(()),
            };
            octx.registry.register(&job.slug, resume_handle).await;
            continue;
        }

        tracing::info!(
            slug = job.slug, job_id = job.job_id,
            "job-dispatch: reconcile — process dead, marking step failed"
        );

        emit_step_failed(kernel, &job.job_id, &job.slug, "daemon restarted, agent not running", store_dir).await;
    }

    tracing::info!("job-dispatch: reconciliation complete");
}

/// Emit job.step_failed and persist to event store.
async fn emit_step_failed(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    job_id: &str,
    slug: &str,
    error: &str,
    store_dir: Option<&std::path::Path>,
) {
    use crate::events::job::event_types;

    let payload = serde_json::json!({
        "job_id": job_id,
        "error": error,
    });
    let envelope = match EventEnvelope::new(event_types::STEP_FAILED, job_id, &payload) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "job-dispatch: reconcile — failed to build envelope: {e}");
            return;
        }
    };

    let persist_result = match store_dir {
        Some(dir) => crate::events::SqliteEventStore::new_at(dir)
            .and_then(|s| crate::events::EventStore::append(&s, &envelope)),
        None => crate::events::SqliteEventStore::new()
            .and_then(|s| crate::events::EventStore::append(&s, &envelope)),
    };
    if let Err(e) = persist_result {
        tracing::warn!(slug, "job-dispatch: reconcile — failed to persist step_failed: {e}");
    }

    let ctx = crate::events::EmitContext {
        caller_kind: crate::events::CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!(slug, "job-dispatch: reconcile — failed to emit step_failed: {e}");
    }
}
