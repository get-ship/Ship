//! Helpers for emitting job dispatch events to the store and kernel.

use crate::events::{CallerKind, EmitContext, EventEnvelope};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Emit a `job.dispatched` event using the default store path.
pub async fn emit_job_dispatched(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    job_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    emit_job_dispatched_with_meta(kernel, job_id, worktree_path, slug, None, None).await;
}

/// Emit a `job.dispatched` event with optional provider metadata.
/// `ship_dir` overrides the default store path (used in tests).
pub async fn emit_job_dispatched_with_meta(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    job_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
    provider_meta: Option<serde_json::Value>,
    ship_dir: Option<&std::path::Path>,
) {
    let dispatched_payload = crate::events::job::JobDispatchedPayload {
        job_id: job_id.to_string(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
        provider_meta,
    };
    let dispatched_envelope = match EventEnvelope::new(
        crate::events::job::event_types::JOB_DISPATCHED,
        job_id,
        &dispatched_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "pipeline: failed to build job.dispatched event: {e}");
            return;
        }
    };

    persist_and_route(kernel, dispatched_envelope, slug, ship_dir).await;
    tracing::info!(slug, "pipeline: job dispatched successfully");
}

/// Emit a `job.step_dispatched` event for a specific pipeline step.
pub async fn emit_step_dispatched_event(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    job_id: &str,
    step_id: &str,
    step_type: &str,
    slug: &str,
    ship_dir: Option<&std::path::Path>,
) {
    let payload = serde_json::json!({
        "job_id": job_id, "slug": slug,
        "step_id": step_id, "step_type": step_type,
    });
    let envelope = match EventEnvelope::new(
        crate::events::job::event_types::STEP_DISPATCHED,
        job_id,
        &payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "pipeline: failed to build step_dispatched: {e}");
            return;
        }
    };

    persist_and_route(kernel, envelope, slug, ship_dir).await;
}

/// Emit a `job.failed` event when a job cannot be dispatched or encounters a fatal error.
pub async fn emit_job_failed(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    job_id: &str,
    slug: &str,
    error: &str,
) {
    let payload = crate::events::job::JobFailedPayload {
        job_id: job_id.to_string(),
        error: error.to_string(),
    };
    let envelope = match EventEnvelope::new(
        crate::events::job::event_types::JOB_FAILED,
        job_id,
        &payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::error!(slug, "emit_job_failed: failed to build event: {e}");
            return;
        }
    };

    persist_and_route(kernel, envelope, slug, None).await;
    tracing::error!(slug, error, "job failed");
}

/// Persist an event to platform.db and deliver it through the kernel.
///
/// Thin wrapper around `KernelRouter::emit` that logs failures at ERROR
/// with slug/event_type/entity_id context. The unit return type matches
/// the call-site contract (most orchestrator callers don't propagate);
/// when callers are rewritten to propagate, they should call `kernel.emit`
/// directly and delete their use of this helper.
///
/// `ship_dir` is accepted for backward compatibility and ignored — the
/// kernel always writes to the platform.db under its own `base_dir`.
pub async fn persist_and_route(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: EventEnvelope,
    slug: &str,
    _ship_dir: Option<&std::path::Path>,
) {
    let event_type = envelope.event_type.clone();
    let entity_id = envelope.entity_id.clone();

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: envelope.workspace_id.clone(),
        session_id: envelope.session_id.clone(),
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::error!(
            slug, event_type = %event_type, entity_id = %entity_id,
            "persist_and_route: kernel emit failed: {e}"
        );
    }
}
