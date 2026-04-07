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
    use runtime::services::dispatch_ports::JobContext;

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
            // Register in ProcessRegistry so graceful shutdown can stop it
            let resume_handle = runtime::services::dispatch_ports::ExecutorHandle {
                pid: None,
                inner: Box::new(()),
            };
            dctx.registry.lock().await.register(&job.slug, resume_handle);
            continue;
        }

        // Dead process — relaunch in existing worktree
        tracing::info!(
            slug = job.slug,
            job_id = job.job_id,
            "job-dispatch: reconcile — process dead, relaunching"
        );

        let work_dir = job
            .worktree
            .as_deref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| super::helpers::worktrees_dir().join(&job.slug));

        // Recompile agent config in case it changed
        if let Err(e) = super::helpers::compile_agent_config(&work_dir, &job.agent) {
            tracing::warn!(
                slug = job.slug,
                "job-dispatch: reconcile — agent config recompile failed: {e}"
            );
            // Continue with relaunch anyway — stale config is better than no agent
        }

        let resume_session = job.provider_meta.as_ref()
            .and_then(|m| m.get("session_id"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let job_ctx = JobContext {
            job_id: job.job_id.clone(),
            slug: job.slug.clone(),
            agent: job.agent.clone(),
            branch: job.branch.clone(),
            spec_path: job.spec_path.clone(),
            work_dir,
            env: [("SHIP_MESH_ID".to_string(), job.slug.clone())].into(),
            launch: runtime::services::dispatch_ports::LaunchProfile {
                provider: job.provider.clone(),
                model: job.model.clone(),
                resume_session,
                persist_session: true,
                ..Default::default()
            },
        };

        match dctx.executor.spawn(&job_ctx).await {
            Ok(new_handle) => {
                tracing::info!(
                    slug = job.slug,
                    "job-dispatch: reconcile — agent relaunched"
                );
                dctx.registry.lock().await.register(&job.slug, new_handle);
            }
            Err(e) => {
                tracing::error!(
                    slug = job.slug,
                    "job-dispatch: reconcile — relaunch failed: {e}"
                );
                emit_job_failed(
                    kernel,
                    &job.job_id,
                    &job.slug,
                    &format!("relaunch failed: {e}"),
                    store_dir,
                )
                .await;
            }
        }
    }

    tracing::info!("job-dispatch: reconciliation complete");
}

/// Emit job.failed and persist to event store.
async fn emit_job_failed(
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
    let envelope = match EventEnvelope::new(event_types::JOB_FAILED, job_id, &payload) {
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
        tracing::warn!(slug, "job-dispatch: reconcile — failed to persist job.failed: {e}");
    }

    let ctx = runtime::events::validator::EmitContext {
        caller_kind: runtime::events::validator::CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!(slug, "job-dispatch: reconcile — failed to emit job.failed: {e}");
    }
}
