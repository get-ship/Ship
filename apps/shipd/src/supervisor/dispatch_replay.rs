//! Replay unprocessed job.completed events on daemon startup.
//!
//! When the daemon restarts, jobs may be stuck in `Completed` status because
//! the supervisor wasn't running to advance pipelines or clean up. This module
//! scans for those jobs and re-runs the completion flow.

use runtime::events::EventEnvelope;
use runtime::projections::job::load_jobs;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch_pipeline::try_advance_pipeline;
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
        if !try_advance_pipeline(kernel, &envelope, dctx).await {
            tracing::info!(
                slug = job.slug,
                "job-dispatch: replay — no pipeline to advance, awaiting gate/merge"
            );
        }
    }

    tracing::info!("job-dispatch: replay complete");
}
