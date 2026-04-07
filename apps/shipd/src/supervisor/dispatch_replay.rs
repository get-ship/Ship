//! Replay and reconciliation for job dispatch.
//!
//! On daemon startup (and after job completion/merge events), this module:
//! 1. Replays stale `job.completed` events that arrived while daemon was down
//! 2. Scans for pending jobs whose dependencies are fully resolved and dispatches them
//!
//! This eliminates the need for an in-memory pending map — the event log is the source of truth.

use runtime::events::EventEnvelope;
use runtime::projections::job::{load_jobs, JobStatus};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch_pipeline::{advance_pipeline, ensure_pipeline};
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
/// executor. If the process is dead, emits `job.failed` with reason
/// "daemon restart, process not found".
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
    use runtime::events::job::event_types;

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
                "job-dispatch: reconcile — process still alive, resuming"
            );
            continue;
        }

        tracing::warn!(
            slug = job.slug,
            job_id = job.job_id,
            "job-dispatch: reconcile — process dead, marking failed"
        );

        let payload = serde_json::json!({
            "job_id": job.job_id,
            "error": "daemon restart, process not found",
        });
        let envelope = match EventEnvelope::new(
            event_types::JOB_FAILED,
            &job.job_id,
            &payload,
        ) {
            Ok(e) => e.with_actor_id("service.job-dispatch"),
            Err(e) => {
                tracing::warn!(
                    slug = job.slug,
                    "job-dispatch: reconcile — failed to build envelope: {e}"
                );
                continue;
            }
        };

        // Persist to event store so projection reflects the failure
        let persist_result = match store_dir {
            Some(dir) => runtime::events::SqliteEventStore::new_at(dir)
                .and_then(|s| runtime::events::EventStore::append(&s, &envelope)),
            None => runtime::events::SqliteEventStore::new()
                .and_then(|s| runtime::events::EventStore::append(&s, &envelope)),
        };
        if let Err(e) = persist_result {
            tracing::warn!(
                slug = job.slug,
                "job-dispatch: reconcile — failed to persist job.failed: {e}"
            );
        }

        let ctx = runtime::events::validator::EmitContext {
            caller_kind: runtime::events::validator::CallerKind::Runtime,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
            tracing::warn!(
                slug = job.slug,
                "job-dispatch: reconcile — failed to emit job.failed: {e}"
            );
        }
    }

    tracing::info!("job-dispatch: reconciliation complete");
}

/// Scan the job projection for pending jobs whose dependencies are all resolved.
/// Dispatch any that are ready. Called on startup and after job.completed/job.merged.
pub(crate) async fn dispatch_ready_pending_jobs(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    dctx: &DispatchContext,
) {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: failed to load jobs for pending scan: {e}");
            return;
        }
    };

    // Build set of completed/merged job slugs
    let done_slugs: std::collections::HashSet<&str> = jobs.values()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::GatePending | JobStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();

    // Count currently running jobs
    let running_count = jobs.values()
        .filter(|j| j.status == JobStatus::Dispatched)
        .count();

    // Concurrency limit — don't overwhelm the machine
    const MAX_CONCURRENT: usize = 3;
    let slots = MAX_CONCURRENT.saturating_sub(running_count);
    if slots == 0 {
        tracing::debug!(running_count, "job-dispatch: concurrency limit reached, skipping dispatch");
        return;
    }

    // Find pending jobs with all deps resolved
    let ready: Vec<_> = jobs.values()
        .filter(|j| j.status == JobStatus::Pending)
        .filter(|j| {
            match &j.depends_on {
                None => true,
                Some(deps) => deps.iter().all(|d| done_slugs.contains(d.as_str())),
            }
        })
        .take(slots)
        .collect();

    if ready.is_empty() {
        tracing::debug!("job-dispatch: no pending jobs ready to dispatch");
        return;
    }

    tracing::info!(
        count = ready.len(),
        "job-dispatch: found pending jobs with resolved dependencies"
    );

    for job in ready {
        tracing::info!(
            slug = job.slug,
            job_id = job.job_id,
            "job-dispatch: dispatching previously-pending job"
        );

        // Build a JobCreatedPayload from the projection record
        let payload = runtime::events::job::JobCreatedPayload {
            job_id: job.job_id.clone(),
            slug: job.slug.clone(),
            agent: job.agent.clone(),
            branch: job.branch.clone(),
            spec_path: job.spec_path.clone(),
            depends_on: None, // deps are resolved
            pipeline: job.pipeline.clone(),
            steps: job.steps.clone(),
            plan_id: None,
            model: None,
            provider: None,
            pipeline_name: None,
        };

        let payload = ensure_pipeline(payload);
        super::job_dispatch::dispatch_job_internal(kernel, &payload, dctx).await;
    }
}
