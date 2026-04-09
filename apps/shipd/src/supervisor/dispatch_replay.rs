//! Replay and reconciliation for job dispatch.
//!
//! On daemon startup (and after job completion/merge events), this module:
//! 1. Replays stale `job.completed` events that arrived while daemon was down
//! 2. Reconciles jobs stuck in `Dispatched` state after a daemon restart
//!
//! Pending job dispatch lives in `pending_dispatch.rs`.

use runtime::events::EventEnvelope;
use runtime::projections::job::{load_jobs, JobStatus};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch_pipeline::advance_pipeline;
use super::job_dispatch::DispatchContext;

/// Replay unprocessed `job.completed` events that arrived while the daemon was down.
///
/// Loads the job projection and finds jobs stuck in `Completed` status (not yet
/// `Merged` or `Failed`). For each, synthesizes a `job.completed` envelope and
/// re-runs `try_advance_pipeline` → cleanup → unblock flow.
pub(crate) async fn replay_unprocessed_completions(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    dctx: &DispatchContext,
) {
    use runtime::projections::job::JobStatus;

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

    tracing::info!(
        count = stale.len(),
        "job-dispatch: replaying stale job.completed events"
    );

    for job in stale {
        let payload = serde_json::json!({
            "job_id": job.job_id,
            "slug": job.slug,
        });
        let envelope = match EventEnvelope::new(
            runtime::events::job::event_types::JOB_COMPLETED,
            &job.job_id,
            &payload,
        ) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(
                    slug = job.slug,
                    "job-dispatch: replay failed to build envelope: {e}"
                );
                continue;
            }
        };

        tracing::info!(
            slug = job.slug,
            job_id = job.job_id,
            "job-dispatch: replaying job.completed"
        );

        // Try to advance pipeline. If no pipeline, just log — do NOT clean up.
        // Cleanup only happens on job.merged.
        if !advance_pipeline(kernel, &envelope, dctx).await {
            tracing::info!(
                slug = job.slug,
                "job-dispatch: replay — no pipeline to advance, awaiting gate/merge"
            );
        }
    }

    tracing::info!("job-dispatch: replay complete");
}

/// Reconcile jobs stuck in `Dispatched` state after daemon restart.
///
/// For each Dispatched job, checks if the process is still alive via the
/// executor. If the process is dead, emits `job.step_failed` with reason
/// "daemon restarted, agent not running". If alive, re-attaches liveness
/// monitoring by registering the handle in ProcessRegistry.
///
/// Uses `load_jobs` (global dir) for production. Tests use the `_at` variant.
pub(crate) async fn reconcile_dispatched_jobs(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    dctx: &DispatchContext,
) {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: reconcile failed to load jobs: {e}");
            return;
        }
    };

    reconcile_dispatched_inner(kernel, dctx, jobs, None).await;
}

/// Path-aware variant for tests.
#[cfg(test)]
pub(crate) async fn reconcile_dispatched_jobs_at(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    dctx: &DispatchContext,
    ship_dir: &std::path::Path,
) {
    let jobs = match runtime::projections::job::load_jobs_at(ship_dir) {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: reconcile failed to load jobs: {e}");
            return;
        }
    };

    reconcile_dispatched_inner(kernel, dctx, jobs, Some(ship_dir)).await;
}

async fn reconcile_dispatched_inner(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    dctx: &DispatchContext,
    jobs: std::collections::HashMap<String, runtime::projections::job::JobRecord>,
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

    tracing::info!(
        count = stale.len(),
        "job-dispatch: reconciling stale dispatched jobs"
    );

    for job in stale {
        // Construct a temporary handle for the liveness check.
        // After restart, the opaque inner state is lost — real executors
        // will return false (downcast fails), which is the safe default.
        let handle = runtime::services::dispatch_ports::ExecutorHandle {
            pid: None,
            inner: Box::new(()),
        };

        if dctx.executor.is_alive(&handle).await {
            tracing::info!(
                slug = job.slug,
                "job-dispatch: reconcile — process still alive, re-attaching"
            );
            // Register in ProcessRegistry so graceful shutdown can stop it
            let resume_handle = runtime::services::dispatch_ports::ExecutorHandle {
                pid: None,
                inner: Box::new(()),
            };
            dctx.registry.lock().await.register(&job.slug, resume_handle);
            continue;
        }

        // Dead process — emit step_failed so the pipeline halts cleanly
        tracing::info!(
            slug = job.slug,
            job_id = job.job_id,
            "job-dispatch: reconcile — process dead, marking step failed"
        );

        emit_step_failed(
            kernel,
            &job.job_id,
            &job.slug,
            "daemon restarted, agent not running",
            store_dir,
        )
        .await;
    }

    tracing::info!("job-dispatch: reconciliation complete");
}

/// Emit job.step_failed and persist to event store.
async fn emit_step_failed(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    job_id: &str,
    slug: &str,
    error: &str,
    store_dir: Option<&std::path::Path>,
) {
    use runtime::events::job::event_types;

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
        Some(dir) => runtime::events::SqliteEventStore::new_at(dir)
            .and_then(|s| runtime::events::EventStore::append(&s, &envelope)),
        None => runtime::events::SqliteEventStore::new()
            .and_then(|s| runtime::events::EventStore::append(&s, &envelope)),
    };
    if let Err(e) = persist_result {
        tracing::warn!(slug, "job-dispatch: reconcile — failed to persist step_failed: {e}");
    }

    let ctx = runtime::events::validator::EmitContext {
        caller_kind: runtime::events::validator::CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!(slug, "job-dispatch: reconcile — failed to emit step_failed: {e}");
    }
}
