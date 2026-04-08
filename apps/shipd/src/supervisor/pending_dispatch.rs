//! Pending job dispatch — scan for jobs whose dependencies are all resolved.
//!
//! Called on startup and after job.completed/job.merged to unblock queued jobs.

use runtime::projections::job::{load_jobs, JobStatus};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::pipeline_loader::ensure_pipeline;
use super::job_dispatch::DispatchContext;

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
