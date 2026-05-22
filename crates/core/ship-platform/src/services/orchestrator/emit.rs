//! Helpers for emitting job dispatch events to the store and kernel.
//!
//! Every emit function follows the same envelope-build → log-on-failure →
//! persist-and-route pattern. Three private helpers consolidate that
//! boilerplate, one per service-actor-id family.

use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Generic typed emit helpers ────────────────────────────────────────────────

/// Build + persist an event under the `service.orchestrator` actor.
/// Logs failures at WARN; failures don't propagate (most orchestrator
/// callers can't recover from an emit failure).
async fn emit_job_dispatch_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    event_type: &str,
    pipeline_run_id: &str,
    slug: &str,
    payload: &P,
    ship_dir: Option<&std::path::Path>,
) {
    let envelope = match EventEnvelope::new(event_type, pipeline_run_id, payload) {
        Ok(e) => e.with_actor_id("service.orchestrator"),
        Err(e) => {
            tracing::warn!(
                slug, event_type,
                "job-dispatch: failed to build envelope: {e}"
            );
            return;
        }
    };
    persist_and_route(kernel, envelope, slug, ship_dir).await;
}

/// Build + persist a `process.*` event under `service.process-lifecycle`.
async fn emit_process_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    event_type: &str,
    slug: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, slug, payload) {
        Ok(e) => e.with_actor_id("service.process-lifecycle"),
        Err(e) => {
            tracing::warn!(slug, event_type, "process-events: failed to build envelope: {e}");
            return;
        }
    };
    persist_and_route(kernel, envelope, slug, None).await;
}

/// Build + persist an `executor.*` event under `service.executor-lifecycle`.
async fn emit_executor_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    event_type: &str,
    slug: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, slug, payload) {
        Ok(e) => e.with_actor_id("service.executor-lifecycle"),
        Err(e) => {
            tracing::warn!(slug, event_type, "executor-events: failed to build envelope: {e}");
            return;
        }
    };
    persist_and_route(kernel, envelope, slug, None).await;
}

// ── Public job-lifecycle emitters ─────────────────────────────────────────────

/// Emit a `job.dispatched` event using the default store path.
pub async fn emit_job_dispatched(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    pipeline_run_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    emit_job_dispatched_with_meta(kernel, pipeline_run_id, worktree_path, slug, None, None).await;
}

/// Emit a `job.dispatched` event with optional harness metadata.
/// `ship_dir` overrides the default store path (used in tests).
pub async fn emit_job_dispatched_with_meta(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    pipeline_run_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
    harness_meta: Option<serde_json::Value>,
    ship_dir: Option<&std::path::Path>,
) {
    let payload = ship_substrate::events::pipeline::PipelineRunDispatchedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
        harness_meta,
    };
    emit_job_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_RUN_DISPATCHED,
        pipeline_run_id,
        slug,
        &payload,
        ship_dir,
    )
    .await;
    tracing::info!(slug, "pipeline: job dispatched successfully");
}

/// Emit a `job.spawn_failed` event for a dispatch failure.
///
/// `stage` identifies which phase of dispatch failed:
/// `"isolation_prepare"`, `"agent_compile"`, or `"executor_spawn"`.
/// Distinct from `job.failed` (agent ran and produced a failure).
pub async fn emit_spawn_failed(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
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
    emit_job_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_STEP_SPAWN_FAILED,
        pipeline_run_id,
        slug,
        &payload,
        None,
    )
    .await;
}

/// Emit a `job.step_dispatched` event for a specific pipeline step.
pub async fn emit_step_dispatched_event(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    pipeline_run_id: &str,
    step_id: &str,
    step_type: &str,
    slug: &str,
    ship_dir: Option<&std::path::Path>,
) {
    let payload = serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug,
        "step_id": step_id, "step_type": step_type,
    });
    emit_job_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_STEP_STARTED,
        pipeline_run_id,
        slug,
        &payload,
        ship_dir,
    )
    .await;
}

/// Emit `job.step_failed` + `job.failed` for a pipeline step error.
pub async fn emit_step_failed(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    step_id: &str,
    reason: &str,
) {
    use ship_substrate::events::pipeline::{event_types, PipelineRunFailedPayload, PipelineStepFailedPayload};
    let step_fail = PipelineStepFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        step_id: step_id.to_string(),
        error: reason.to_string(),
    };
    emit_job_dispatch_event(kernel, event_types::PIPELINE_STEP_FAILED, pipeline_run_id, slug, &step_fail, None).await;
    let run_fail = PipelineRunFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        error: format!("step `{step_id}` failed: {reason}"),
    };
    emit_job_dispatch_event(kernel, event_types::PIPELINE_RUN_FAILED, pipeline_run_id, slug, &run_fail, None).await;
}

/// Emit a `job.failed` event when a job cannot be dispatched or encounters a fatal error.
pub async fn emit_job_failed(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    error: &str,
) {
    let payload = ship_substrate::events::pipeline::PipelineRunFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        error: error.to_string(),
    };
    emit_job_dispatch_event(
        kernel,
        ship_substrate::events::pipeline::event_types::PIPELINE_RUN_FAILED,
        pipeline_run_id,
        slug,
        &payload,
        None,
    )
    .await;
    tracing::error!(slug, error, "job failed");
}

// ── Process lifecycle event emitters ──────────────────────────────────────────

/// Emit `process.spawned` after an executor successfully creates a process.
pub async fn emit_process_spawned(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    slug: &str,
    pipeline_run_id: &str,
    pid: Option<u32>,
    cmd: &str,
    argv: &[String],
    env_keys: &[String],
    cwd: &str,
    executor_kind: &str,
    step_id: Option<&str>,
) {
    use ship_substrate::events::process::{
        ProcessSpawnedPayload, event_types, filter_env_keys, sanitize_argv,
    };
    let payload = ProcessSpawnedPayload {
        slug: slug.to_string(),
        pipeline_run_id: pipeline_run_id.to_string(),
        pid,
        cmd: cmd.to_string(),
        argv: sanitize_argv(argv),
        env_keys: filter_env_keys(env_keys),
        cwd: cwd.to_string(),
        executor_kind: executor_kind.to_string(),
        step_id: step_id.map(str::to_string),
    };
    emit_process_event(kernel, event_types::PROCESS_SPAWNED, slug, &payload).await;
}

/// Emit `process.exited` when a monitored process terminates.
pub async fn emit_process_exited(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    slug: &str,
    pid: Option<u32>,
    exit_code: Option<i32>,
    signal: Option<&str>,
    wall_duration_ms: u64,
    executor_kind: &str,
) {
    use ship_substrate::events::process::{ProcessExitedPayload, event_types};
    let payload = ProcessExitedPayload {
        slug: slug.to_string(),
        pid,
        exit_code,
        signal: signal.map(str::to_string),
        wall_duration_ms,
        executor_kind: executor_kind.to_string(),
    };
    emit_process_event(kernel, event_types::PROCESS_EXITED, slug, &payload).await;
}

/// Emit `process.signal_sent` before sending a signal to a process.
pub async fn emit_process_signal_sent(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    slug: &str,
    pid: Option<u32>,
    signum: &str,
    reason: &str,
    actor: &str,
) {
    use ship_substrate::events::process::{ProcessSignalSentPayload, event_types};
    let payload = ProcessSignalSentPayload {
        slug: slug.to_string(),
        pid,
        signum: signum.to_string(),
        reason: reason.to_string(),
        actor: actor.to_string(),
    };
    emit_process_event(kernel, event_types::PROCESS_SIGNAL_SENT, slug, &payload).await;
}

// ── Executor substrate event emitters ─────────────────────────────────────────

/// Emit `executor.session_destroyed` when a tmux session (or equivalent
/// executor substrate) is torn down. Used by the liveness handler to record
/// the why before emitting `job.failed`.
pub async fn emit_executor_session_destroyed(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    slug: &str,
    executor_kind: &str,
    session_name: &str,
    cause: ship_substrate::events::executor::DestructionCause,
) {
    use ship_substrate::events::executor::{EXECUTOR_SESSION_DESTROYED, ExecutorSessionDestroyedPayload};
    let payload = ExecutorSessionDestroyedPayload {
        executor_kind: executor_kind.to_string(),
        session_name: session_name.to_string(),
        slug: slug.to_string(),
        cause,
    };
    emit_executor_event(kernel, EXECUTOR_SESSION_DESTROYED, slug, &payload).await;
}

/// Emit `executor.window_destroyed` when an individual window within a
/// shared-session executor is torn down.
pub async fn emit_executor_window_destroyed(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    slug: &str,
    executor_kind: &str,
    session_name: &str,
    window_name: &str,
    step: Option<&str>,
    cause: ship_substrate::events::executor::DestructionCause,
) {
    use ship_substrate::events::executor::{EXECUTOR_WINDOW_DESTROYED, ExecutorWindowDestroyedPayload};
    let payload = ExecutorWindowDestroyedPayload {
        executor_kind: executor_kind.to_string(),
        session_name: session_name.to_string(),
        window_name: window_name.to_string(),
        slug: slug.to_string(),
        step: step.map(str::to_string),
        cause,
    };
    emit_executor_event(kernel, EXECUTOR_WINDOW_DESTROYED, slug, &payload).await;
}

// ── Routing ───────────────────────────────────────────────────────────────────

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
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: EventEnvelope,
    slug: &str,
    _ship_dir: Option<&std::path::Path>,
) {
    let event_type = envelope.event_type.clone();
    let entity_id = envelope.entity_id.clone();

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        project_id: envelope.project_id.clone(),
        session_id: envelope.session_id.clone(),
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::error!(
            slug, event_type = %event_type, entity_id = %entity_id,
            "persist_and_route: kernel emit failed: {e}"
        );
    }
}
