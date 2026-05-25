//! Pending job dispatch — scan for jobs whose dependencies are all resolved.
//!
//! Called on startup and after job.completed/job.merged to unblock queued jobs.

use crate::projections::pipeline_run::{load_pipeline_runs, PipelineRunStatus};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch::dispatch_job_internal;
use super::pipeline_loader::ensure_pipeline;
use super::OrchestratorContext;

/// Scan the job projection for pending jobs whose dependencies are all resolved.
/// Dispatch any that are ready. Called on startup and after job.completed/job.merged.
pub async fn dispatch_ready_pending_jobs(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
) {
    let jobs = match load_pipeline_runs() {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("job-dispatch: failed to load jobs for pending scan: {e}");
            return;
        }
    };

    let done_slugs: std::collections::HashSet<&str> = jobs.values()
        .filter(|j| matches!(j.status, PipelineRunStatus::Completed | PipelineRunStatus::GatePending | PipelineRunStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();

    let running_count = jobs.values()
        .filter(|j| j.status == PipelineRunStatus::Dispatched)
        .count();

    const MAX_CONCURRENT: usize = 3;
    let slots = MAX_CONCURRENT.saturating_sub(running_count);
    if slots == 0 {
        tracing::debug!(running_count, "job-dispatch: concurrency limit reached, skipping dispatch");
        return;
    }

    let ready: Vec<_> = jobs.values()
        .filter(|j| j.status == PipelineRunStatus::Pending)
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

    tracing::info!(count = ready.len(), "job-dispatch: found pending jobs with resolved dependencies");

    for job in ready {
        tracing::info!(slug = job.slug, pipeline_run_id = job.pipeline_run_id, "job-dispatch: dispatching previously-pending job");

        let payload = ship_substrate::events::pipeline::PipelineRunStartedPayload {
            pipeline_run_id: job.pipeline_run_id.clone(),
            slug: job.slug.clone(),
            agent: job.agent.clone(),
            branch: job.branch.clone(),
            spec_path: job.spec_path.clone(),
            source_spec_path: job.source_spec_path.clone(),
            depends_on: None,
            pipeline: job.pipeline.clone(),
            steps: job.steps.clone(),
            plan_id: None,
            model: None,
            harness: None,
            pipeline_name: None,
            vars: None,
            spec_content: None,
            step_specs: None,
            headless: job.headless,
            keepalive: false,
            prompt: None,
            resume: false,
            resume_session_id: None,
            baseline_sha: job.baseline_sha.clone(),
        };

        let slug_for_err = payload.slug.clone();
        let payload = match ensure_pipeline(payload) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(slug = slug_for_err, "pipeline: load failed: {e}");
                continue;
            }
        };
        dispatch_job_internal(kernel, &payload, octx).await;
    }
}
