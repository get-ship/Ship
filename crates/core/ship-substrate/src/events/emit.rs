//! Kernel-emit plumbing for the runner mechanism.
//!
//! Two layers:
//!   1. [`EventSink`] — the injected kernel-emit *port*. The runner mechanism
//!      depends on this trait, never on a concrete [`KernelRouter`]. The L0
//!      concrete impl ([`KernelEventSink`]) is provided at construction. This
//!      is the seam that resolves the parked layer-direction violation: an L0
//!      supervisor / runner that needs to emit no longer reaches up into an L1
//!      `emit` module — it calls the port (or the `persist_and_route` helper
//!      directly with a substrate kernel handle, which is L0-to-L0).
//!   2. The substrate-typed emit helpers — every one follows the same
//!      envelope-build → log-on-failure → persist-and-route pattern, and every
//!      signature uses only substrate / primitive types. Domain-free.

use async_trait::async_trait;
use crate::events::{CallerKind, EmitContext, EventEnvelope};
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Kernel-emit port ───────────────────────────────────────────────────────

/// Port: emit an event envelope. The runner mechanism takes a `&dyn EventSink`
/// so it never names [`KernelRouter`] directly; the concrete kernel-backed impl
/// is injected at construction.
///
/// Domain-free: the only types crossing the seam are [`EventEnvelope`] and the
/// substrate primitives it carries.
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Persist + route a single envelope. Implementations log their own
    /// failures; the unit return matches the fire-and-forget call sites.
    async fn emit(&self, envelope: EventEnvelope);
}

/// The L0 concrete [`EventSink`]: wraps the substrate [`KernelRouter`] and
/// drives it through [`persist_and_route`]. Built by the host (daemon) and
/// handed to the runner mechanism as `Arc<dyn EventSink>`.
#[derive(Clone)]
pub struct KernelEventSink {
    kernel: Arc<Mutex<crate::events::KernelRouter>>,
}

impl KernelEventSink {
    pub fn new(kernel: Arc<Mutex<crate::events::KernelRouter>>) -> Self {
        Self { kernel }
    }
}

#[async_trait]
impl EventSink for KernelEventSink {
    async fn emit(&self, envelope: EventEnvelope) {
        let slug = envelope
            .space_id()
            .map(str::to_string)
            .unwrap_or_default();
        persist_and_route(&self.kernel, envelope, &slug).await;
    }
}

// ── Generic typed emit helpers ─────────────────────────────────────────────

/// Build + persist an event under the `service.orchestrator` actor.
/// Logs failures at WARN; failures don't propagate (most callers can't
/// recover from an emit failure).
async fn emit_pipeline_run_dispatch_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    event_type: &str,
    pipeline_run_id: &str,
    slug: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, pipeline_run_id, payload) {
        Ok(e) => e.with_actor_id("service.orchestrator"),
        Err(e) => {
            tracing::warn!(
                slug, event_type,
                "pipeline-run-dispatch: failed to build envelope: {e}"
            );
            return;
        }
    };
    persist_and_route(kernel, envelope, slug).await;
}

/// Build + persist a `process.*` event under `service.process-lifecycle`.
async fn emit_process_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
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
    persist_and_route(kernel, envelope, slug).await;
}

/// Build + persist an `executor.*` event under `service.executor-lifecycle`.
async fn emit_executor_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
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
    persist_and_route(kernel, envelope, slug).await;
}

// ── Public pipeline-run-lifecycle emitters ─────────────────────────────────

/// Emit a `pipeline.run.dispatched` event using the default store path.
pub async fn emit_pipeline_run_dispatched(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    pipeline_run_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    emit_pipeline_run_dispatched_with_meta(kernel, pipeline_run_id, worktree_path, slug, None).await;
}

/// Emit a `pipeline.run.dispatched` event with optional harness metadata.
pub async fn emit_pipeline_run_dispatched_with_meta(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    pipeline_run_id: &str,
    worktree_path: &std::path::Path,
    slug: &str,
    harness_meta: Option<serde_json::Value>,
) {
    let payload = crate::events::pipeline::PipelineRunDispatchedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
        harness_meta,
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        crate::events::pipeline::event_types::PIPELINE_RUN_DISPATCHED,
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
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    stage: &str,
    err: &anyhow::Error,
) {
    let payload = crate::events::pipeline::PipelineStepSpawnFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        stage: stage.to_string(),
        error: format!("{err:#}"),
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        crate::events::pipeline::event_types::PIPELINE_STEP_SPAWN_FAILED,
        pipeline_run_id,
        slug,
        &payload,
    )
    .await;
}

/// Emit a `pipeline.step.started` event for a specific pipeline step.
pub async fn emit_step_started_event(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
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
        crate::events::pipeline::event_types::PIPELINE_STEP_STARTED,
        pipeline_run_id,
        slug,
        &payload,
    )
    .await;
}

/// Emit `pipeline.step.failed` + `pipeline.run.failed` for a pipeline step error.
pub async fn emit_step_failed(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    step_id: &str,
    reason: &str,
) {
    use crate::events::pipeline::{event_types, PipelineRunFailedPayload, PipelineStepFailedPayload};
    let step_fail = PipelineStepFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        step_id: step_id.to_string(),
        error: reason.to_string(),
    };
    emit_pipeline_run_dispatch_event(kernel, event_types::PIPELINE_STEP_FAILED, pipeline_run_id, slug, &step_fail).await;
    let run_fail = PipelineRunFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        error: format!("step `{step_id}` failed: {reason}"),
    };
    emit_pipeline_run_dispatch_event(kernel, event_types::PIPELINE_RUN_FAILED, pipeline_run_id, slug, &run_fail).await;
}

/// Emit a `pipeline.run.failed` event when a pipeline run cannot be dispatched or encounters a fatal error.
pub async fn emit_pipeline_run_failed(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    error: &str,
) {
    let payload = crate::events::pipeline::PipelineRunFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        error: error.to_string(),
    };
    emit_pipeline_run_dispatch_event(
        kernel,
        crate::events::pipeline::event_types::PIPELINE_RUN_FAILED,
        pipeline_run_id,
        slug,
        &payload,
    )
    .await;
    tracing::error!(slug, error, "pipeline run failed");
}

// ── Process lifecycle event emitters ───────────────────────────────────────

/// Emit `process.spawned` after an executor successfully creates a process.
#[allow(clippy::too_many_arguments)]
pub async fn emit_process_spawned(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
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
    use crate::events::process::{
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
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    slug: &str,
    pid: Option<u32>,
    exit_code: Option<i32>,
    signal: Option<&str>,
    wall_duration_ms: u64,
    executor_kind: &str,
) {
    use crate::events::process::{ProcessExitedPayload, event_types};
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
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    slug: &str,
    pid: Option<u32>,
    signum: &str,
    reason: &str,
    actor: &str,
) {
    use crate::events::process::{ProcessSignalSentPayload, event_types};
    let payload = ProcessSignalSentPayload {
        slug: slug.to_string(),
        pid,
        signum: signum.to_string(),
        reason: reason.to_string(),
        actor: actor.to_string(),
    };
    emit_process_event(kernel, event_types::PROCESS_SIGNAL_SENT, slug, &payload).await;
}

// ── Executor substrate event emitters ──────────────────────────────────────

/// Emit `executor.session_destroyed` when a tmux session (or equivalent
/// executor substrate) is torn down. Used by the liveness handler to record
/// the why before emitting `pipeline.run.failed`.
pub async fn emit_executor_session_destroyed(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    slug: &str,
    executor_kind: &str,
    session_name: &str,
    cause: crate::events::executor::DestructionCause,
) {
    use crate::events::executor::{EXECUTOR_SESSION_DESTROYED, ExecutorSessionDestroyedPayload};
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
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    slug: &str,
    executor_kind: &str,
    session_name: &str,
    window_name: &str,
    step: Option<&str>,
    cause: crate::events::executor::DestructionCause,
) {
    use crate::events::executor::{EXECUTOR_WINDOW_DESTROYED, ExecutorWindowDestroyedPayload};
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

// ── Routing ─────────────────────────────────────────────────────────────────

/// Persist an event and deliver it through the kernel.
///
/// Thin wrapper around `KernelRouter::emit` that logs failures at ERROR
/// with slug/event_type/entity_id context. The unit return type matches
/// the call-site contract (most callers don't propagate); when callers are
/// rewritten to propagate, they should call `kernel.emit` directly and
/// delete their use of this helper.
pub async fn persist_and_route(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: EventEnvelope,
    slug: &str,
) {
    let event_type = envelope.event_type.clone();
    let entity_id = envelope.entity_id.clone();

    // Relay: carry the envelope's own resolved space through the binding,
    // preserving its session. Daemon-internal mint (trusted relay).
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(
            envelope.space_id().map(ship_types::SpaceId::new),
            ship_types::Authority::Sdk,
        ),
        session_id: envelope.session_id.clone(),
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::error!(
            slug, event_type = %event_type, entity_id = %entity_id,
            "persist_and_route: kernel emit failed: {e}"
        );
    }
}
