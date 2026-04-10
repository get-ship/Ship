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

/// Persist an event to the store and route through the kernel.
pub(super) async fn persist_and_route(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: EventEnvelope,
    slug: &str,
    ship_dir: Option<&std::path::Path>,
) {
    let store_result = match ship_dir {
        Some(dir) => crate::events::SqliteEventStore::new_at(dir),
        None => crate::events::SqliteEventStore::new(),
    };
    if let Ok(store) = store_result {
        if let Err(e) = crate::events::EventStore::append(&store, &envelope) {
            tracing::warn!(slug, "pipeline: failed to persist event: {e}");
        }
    }

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!(slug, "pipeline: event routing failed: {e}");
    }
}
