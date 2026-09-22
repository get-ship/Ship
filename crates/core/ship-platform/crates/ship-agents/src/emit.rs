//! Domain emit helpers for process/executor lifecycle events.
//!
//! Moved from `ship-substrate::events::emit` (j-07): payload structs stay in
//! the engine; only these convenience emitters live with their consumers.

use std::sync::Arc;

use ship_substrate::events::emit::persist_and_route;
use ship_substrate::events::{EventEnvelope, KernelRouter};
use tokio::sync::Mutex;

/// Build + persist a `process.*` event under `service.process-lifecycle`.
async fn emit_process_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    slug: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, slug, payload) {
        Ok(e) => e.with_actor_id("service.process-lifecycle"),
        Err(e) => {
            tracing::warn!(
                slug,
                event_type,
                "process-events: failed to build envelope: {e}"
            );
            return;
        }
    };
    persist_and_route(kernel, envelope, slug).await;
}

/// Build + persist an `executor.*` event under `service.executor-lifecycle`.
async fn emit_executor_event<P: serde::Serialize>(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    slug: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, slug, payload) {
        Ok(e) => e.with_actor_id("service.executor-lifecycle"),
        Err(e) => {
            tracing::warn!(
                slug,
                event_type,
                "executor-events: failed to build envelope: {e}"
            );
            return;
        }
    };
    persist_and_route(kernel, envelope, slug).await;
}

/// Emit `process.spawned` after an executor successfully creates a process.
#[allow(clippy::too_many_arguments)]
pub async fn emit_process_spawned(
    kernel: &Arc<Mutex<KernelRouter>>,
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
    use ship_substrate::events::process::{ProcessSpawnedPayload, filter_env_keys, sanitize_argv};
    use ship_types::event_types;
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
    kernel: &Arc<Mutex<KernelRouter>>,
    slug: &str,
    pid: Option<u32>,
    exit_code: Option<i32>,
    signal: Option<&str>,
    wall_duration_ms: u64,
    executor_kind: &str,
) {
    use ship_substrate::events::process::ProcessExitedPayload;
    use ship_types::event_types;
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
    kernel: &Arc<Mutex<KernelRouter>>,
    slug: &str,
    pid: Option<u32>,
    signum: &str,
    reason: &str,
    actor: &str,
) {
    use ship_substrate::events::process::ProcessSignalSentPayload;
    use ship_types::event_types;
    let payload = ProcessSignalSentPayload {
        slug: slug.to_string(),
        pid,
        signum: signum.to_string(),
        reason: reason.to_string(),
        actor: actor.to_string(),
    };
    emit_process_event(kernel, event_types::PROCESS_SIGNAL_SENT, slug, &payload).await;
}

/// Emit `executor.session_destroyed` when a tmux session (or equivalent
/// executor substrate) is torn down.
pub async fn emit_executor_session_destroyed(
    kernel: &Arc<Mutex<KernelRouter>>,
    slug: &str,
    executor_kind: &str,
    session_name: &str,
    cause: ship_substrate::events::executor::DestructionCause,
) {
    use ship_substrate::events::executor::{
        EXECUTOR_SESSION_DESTROYED, ExecutorSessionDestroyedPayload,
    };
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
    kernel: &Arc<Mutex<KernelRouter>>,
    slug: &str,
    executor_kind: &str,
    session_name: &str,
    window_name: &str,
    step: Option<&str>,
    cause: ship_substrate::events::executor::DestructionCause,
) {
    use ship_substrate::events::executor::{
        EXECUTOR_WINDOW_DESTROYED, ExecutorWindowDestroyedPayload,
    };
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
