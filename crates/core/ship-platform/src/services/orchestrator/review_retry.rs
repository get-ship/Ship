//! Review retry logic — re-dispatches the reviewed step when a review
//! agent's verdict is Failed.
//!
//! Distinct from the diff-rule gate (`GateSpec` attribute on any step).
//! This module only handles `StepType::Review` — the agent-backed reviewer.

use ship_substrate::events::pipeline::{PipelineStepObservedPayload, StepType};
use ship_substrate::events::EventEnvelope;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::pipeline::advance_pipeline;
use super::OrchestratorContext;

/// Attempt to retry a reviewed step after the review agent fails it.
///
/// Returns `true` if a retry was dispatched, `false` if no retry is possible.
pub async fn try_review_retry(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    octx: &OrchestratorContext,
    payload: &PipelineStepObservedPayload,
) -> bool {
    let jobs = match crate::projections::job::load_jobs() {
        Ok(j) => j,
        Err(_) => return false,
    };
    let job = match jobs.values().find(|j| j.job_id == payload.job_id) {
        Some(j) => j.clone(),
        None => return false,
    };
    let pipeline = match &job.steps {
        Some(p) => p,
        None => return false,
    };

    let review_step = match pipeline.steps.iter().find(|s| s.id == payload.step_name) {
        Some(s) => s,
        None => return false,
    };
    if review_step.step_type != StepType::Review {
        return false;
    }

    let gated_step_id = match review_step.after.first() {
        Some(id) => id.clone(),
        None => return false,
    };
    let gated_step = match pipeline.steps.iter().find(|s| s.id == gated_step_id) {
        Some(s) => s,
        None => return false,
    };

    let max_retries = gated_step.max_retries.unwrap_or(0);
    if max_retries == 0 {
        return false;
    }

    let retries_used = job.step_retries.get(&gated_step_id).copied().unwrap_or(0);
    if retries_used >= max_retries {
        tracing::warn!(
            slug = %payload.slug, step = %gated_step_id,
            retries_used, max_retries, "gate-retry: retries exhausted"
        );
        return false;
    }

    let attempt = retries_used + 1;
    tracing::info!(
        slug = %payload.slug, step = %gated_step_id,
        attempt, max_retries, "gate-retry: retrying gated step"
    );

    // Emit step.retry event
    let retry_payload = ship_substrate::events::pipeline::PipelineStepRetryPayload {
        job_id: payload.job_id.clone(),
        slug: payload.slug.clone(),
        step_id: gated_step_id.clone(),
        step_name: gated_step_id.clone(),
        attempt,
        reason: "gate_failed".to_string(),
    };
    let retry_envelope = match EventEnvelope::new(
        ship_substrate::events::pipeline::event_types::PIPELINE_STEP_RETRY,
        &payload.job_id,
        &retry_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug = %payload.slug, "gate-retry: failed to build event: {e}");
            return false;
        }
    };

    crate::services::orchestrator::emit::persist_and_route(
        kernel, retry_envelope, &payload.slug, None,
    ).await;

    // Build synthetic envelope for advance_pipeline
    let synthetic = match EventEnvelope::new(
        "pipeline.step.retry",
        &payload.job_id,
        &serde_json::json!({ "job_id": &payload.job_id, "slug": &payload.slug }),
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(_) => return false,
    };

    advance_pipeline(kernel, &synthetic, octx).await
}

/// Emit `job.failed` when a review fails with no retries remaining.
pub async fn emit_job_failed_for_review(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineStepObservedPayload,
) {
    let jobs = match crate::projections::job::load_jobs() {
        Ok(j) => j,
        Err(_) => return,
    };
    let job = match jobs.values().find(|j| j.job_id == payload.job_id) {
        Some(j) => j,
        None => return,
    };
    let pipeline = match &job.steps {
        Some(p) => p,
        None => return,
    };

    let is_review = pipeline
        .steps
        .iter()
        .any(|s| s.id == payload.step_name && s.step_type == StepType::Review);
    if !is_review {
        return;
    }

    let reason = payload
        .reason
        .clone()
        .unwrap_or_else(|| "review failed".to_string());
    let error = match &payload.log_tail {
        Some(tail) => format!("{reason}\n\nagent log tail:\n{tail}"),
        None => reason,
    };
    let fail_payload = ship_substrate::events::pipeline::PipelineRunFailedPayload {
        job_id: payload.job_id.clone(),
        error,
    };
    let env = match EventEnvelope::new(
        ship_substrate::events::pipeline::event_types::PIPELINE_RUN_FAILED,
        &payload.job_id,
        &fail_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(_) => return,
    };
    crate::services::orchestrator::emit::persist_and_route(
        kernel, env, &payload.slug, None,
    ).await;
}
