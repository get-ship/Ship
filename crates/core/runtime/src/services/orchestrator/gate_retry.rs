//! Gate retry logic — re-dispatches gated steps after gate failure.
//!
//! When a gate step fails (`step.observed` with `succeeded: false`), this module
//! checks the retry policy on the gated step and either re-dispatches it or
//! emits `job.failed` if retries are exhausted.

use crate::events::job::{StepObservedPayload, StepType};
use crate::events::EventEnvelope;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::pipeline::advance_pipeline;
use super::OrchestratorContext;

/// Attempt to retry a gated step after gate failure.
///
/// Returns `true` if a retry was dispatched, `false` if no retry is possible.
pub async fn try_gate_retry(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    octx: &OrchestratorContext,
    payload: &StepObservedPayload,
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

    let gate_step = match pipeline.steps.iter().find(|s| s.id == payload.step_name) {
        Some(s) => s,
        None => return false,
    };
    if gate_step.step_type != StepType::Gate {
        return false;
    }

    let gated_step_id = match gate_step.after.first() {
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
    let retry_payload = crate::events::job::StepRetryPayload {
        job_id: payload.job_id.clone(),
        slug: payload.slug.clone(),
        step_id: gated_step_id.clone(),
        step_name: gated_step_id.clone(),
        attempt,
        reason: "gate_failed".to_string(),
    };
    let retry_envelope = match EventEnvelope::new(
        crate::events::job::event_types::STEP_RETRY,
        &payload.job_id,
        &retry_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug = %payload.slug, "gate-retry: failed to build event: {e}");
            return false;
        }
    };

    if let Ok(store) = crate::events::SqliteEventStore::new() {
        if let Err(e) = crate::events::EventStore::append(&store, &retry_envelope) {
            tracing::warn!(slug = %payload.slug, "gate-retry: failed to persist: {e}");
        }
    }
    let ctx = crate::events::EmitContext {
        caller_kind: crate::events::CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    let _ = kernel.lock().await.route(retry_envelope, &ctx).await;

    // Build synthetic envelope for advance_pipeline
    let synthetic = match EventEnvelope::new(
        "step.retry",
        &payload.job_id,
        &serde_json::json!({ "job_id": &payload.job_id, "slug": &payload.slug }),
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(_) => return false,
    };

    advance_pipeline(kernel, &synthetic, octx).await
}

/// Emit `job.failed` when a gate fails with no retries remaining.
pub async fn emit_job_failed_for_gate(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    payload: &StepObservedPayload,
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

    let is_gate = pipeline
        .steps
        .iter()
        .any(|s| s.id == payload.step_name && s.step_type == StepType::Gate);
    if !is_gate {
        return;
    }

    let reason = payload
        .reason
        .clone()
        .unwrap_or_else(|| "gate failed".to_string());
    let fail_payload = crate::events::job::JobFailedPayload {
        job_id: payload.job_id.clone(),
        error: reason,
    };
    let env = match EventEnvelope::new(
        crate::events::job::event_types::JOB_FAILED,
        &payload.job_id,
        &fail_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(_) => return,
    };
    if let Ok(store) = crate::events::SqliteEventStore::new() {
        let _ = crate::events::EventStore::append(&store, &env);
    }
    let ctx = crate::events::EmitContext {
        caller_kind: crate::events::CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    let _ = kernel.lock().await.route(env, &ctx).await;
}
