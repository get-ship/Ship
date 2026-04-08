//! Helpers for emitting `job.dispatched` events to the store and kernel.
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Emit a `job.dispatched` event using the default store path.
pub(crate) async fn emit_job_dispatched(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    job_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    emit_job_dispatched_with_meta(kernel, job_id, worktree_path, slug, None, None).await;
}

/// Emit a `job.dispatched` event with optional provider metadata.
/// `ship_dir` overrides the default store path (used in tests).
pub(crate) async fn emit_job_dispatched_with_meta(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    job_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
    provider_meta: Option<serde_json::Value>,
    ship_dir: Option<&std::path::Path>,
) {
    let dispatched_payload = runtime::events::job::JobDispatchedPayload {
        job_id: job_id.to_string(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
        provider_meta,
    };
    let dispatched_envelope = match EventEnvelope::new(
        runtime::events::job::event_types::JOB_DISPATCHED,
        job_id,
        &dispatched_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "pipeline: failed to build job.dispatched event: {e}");
            return;
        }
    };

    let store_result = match ship_dir {
        Some(dir) => runtime::events::SqliteEventStore::new_at(dir),
        None => runtime::events::SqliteEventStore::new(),
    };
    if let Ok(store) = store_result {
        if let Err(e) = runtime::events::EventStore::append(&store, &dispatched_envelope) {
            tracing::warn!(slug, "pipeline: failed to persist job.dispatched: {e}");
        }
    }

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(dispatched_envelope, &ctx).await {
        tracing::warn!(slug, "pipeline: job.dispatched routing failed: {e}");
    }

    tracing::info!(slug, "pipeline: job dispatched successfully");
}
