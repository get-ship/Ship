//! Pipeline-run lifecycle emit helpers.
//!
//! Convenience emitters for `pipeline.run.*` / `pipeline.step.*` events.
//! Payload STRUCTS stay in `ship_substrate::events::pipeline`; only these
//! domain convenience fns live with the pipeline family.

use std::sync::Arc;

use ship_substrate::events::emit::persist_and_route;
use ship_substrate::events::{EventEnvelope, KernelRouter};
use tokio::sync::Mutex;

/// Build + persist an event under the `service.orchestrator` actor.
/// Logs failures at WARN; failures don't propagate (most callers can't
/// recover from an emit failure).
async fn emit_pipeline_run_dispatch_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    pipeline_run_id: &str,
    slug: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, pipeline_run_id, payload) {
        Ok(e) => e.with_actor_id("service.orchestrator"),
        Err(e) => {
            tracing::warn!(
                slug,
                event_type,
                "pipeline-run-dispatch: failed to build envelope: {e}"
            );
            return;
        }
    };
    persist_and_route(kernel, envelope, slug).await;
}

/// Emit a `pipeline.run.dispatched` event using the default store path.
pub async fn emit_pipeline_run_dispatched(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    emit_pipeline_run_dispatched_with_meta(kernel, pipeline_run_id, worktree_path, slug, None)
        .await;
}

/// Emit a `pipeline.run.dispatched` event with optional harness metadata.
pub async fn emit_pipeline_run_dispatched_with_meta(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
    harness_meta: Option<serde_json::Value>,
) {
    let payload = ship_substrate::events::pipeline::PipelineRunDispatchedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
        harness_meta,
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_RUN_DISPATCHED,
        pipeline_run_id,
        slug,
        &payload,
    )
    .await;
    tracing::info!(slug, "pipeline run dispatched");
}

/// Emit a `pipeline.step.spawn_failed` event for a dispatch failure.
///
/// `stage` identifies which phase of dispatch failed:
/// `"isolation_prepare"`, `"agent_compile"`, or `"executor_spawn"`.
/// Distinct from `pipeline.run.failed` (agent ran and produced a failure).
pub async fn emit_spawn_failed(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    stage: &str,
    err: &anyhow::Error,
) {
    let payload = ship_substrate::events::pipeline::PipelineStepSpawnFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        stage: stage.to_string(),
        error: format!("{err:#}"),
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_STEP_SPAWN_FAILED,
        pipeline_run_id,
        slug,
        &payload,
    )
    .await;
}

/// Emit a `pipeline.step.started` event for a specific pipeline step.
pub async fn emit_step_started_event(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    step_id: &str,
    step_type: &str,
    slug: &str,
) {
    let payload = serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug,
        "step_id": step_id, "step_type": step_type,
    });
    emit_pipeline_run_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_STEP_STARTED,
        pipeline_run_id,
        slug,
        &payload,
    )
    .await;
}

/// Emit `pipeline.step.failed` + `pipeline.run.failed` for a pipeline step error.
pub async fn emit_step_failed(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    step_id: &str,
    reason: &str,
) {
    use ship_substrate::events::pipeline::{
        PipelineRunFailedPayload, PipelineStepFailedPayload, event_types,
    };
    let step_fail = PipelineStepFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        step_id: step_id.to_string(),
        error: reason.to_string(),
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        event_types::PIPELINE_STEP_FAILED,
        pipeline_run_id,
        slug,
        &step_fail,
    )
    .await;
    let run_fail = PipelineRunFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        error: format!("step `{step_id}` failed: {reason}"),
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        event_types::PIPELINE_RUN_FAILED,
        pipeline_run_id,
        slug,
        &run_fail,
    )
    .await;
}

/// Emit a `pipeline.run.failed` event when a pipeline run cannot be dispatched
/// or encounters a fatal error.
pub async fn emit_pipeline_run_failed(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    error: &str,
) {
    let payload = ship_substrate::events::pipeline::PipelineRunFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        error: error.to_string(),
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_RUN_FAILED,
        pipeline_run_id,
        slug,
        &payload,
    )
    .await;
    tracing::error!(slug, error, "pipeline run failed");
}
