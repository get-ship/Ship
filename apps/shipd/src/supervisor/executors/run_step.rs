//! Run step executor — executes shell commands as pipeline steps.
//!
//! A run step is a pipeline step with `type: run` (or `run: <command>` shorthand).
//! It executes a shell command in the job's worktree directory, capturing output
//! and emitting step lifecycle events to the kernel.
//!
//! On exit 0: emits `job.step_output` then `job.step_completed`.
//! On non-zero exit: emits `job.step_output` then `job.step_failed`.

use runtime::events::job::event_types;
use runtime::events::{CallerKind, EmitContext, EventEnvelope, EventStore, SqliteEventStore};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

async fn persist_and_route(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: EventEnvelope,
    ship_dir: Option<&Path>,
) {
    let store_result = match ship_dir {
        Some(dir) => SqliteEventStore::new_at(dir),
        None => SqliteEventStore::new(),
    };
    if let Ok(store) = store_result {
        if let Err(e) = EventStore::append(&store, &envelope) {
            tracing::warn!("run-step: failed to persist event: {e}");
        }
    }
    let ctx = runtime_ctx();
    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!("run-step: failed to route event: {e}");
    }
}

/// Execute a shell command as a pipeline step.
///
/// Emits `job.step_dispatched`, then runs `sh -c <command>` in `work_dir`.
/// On success: emits `job.step_output` + `job.step_completed`.
/// On failure: emits `job.step_output` + `job.step_failed`.
pub(crate) async fn execute_run_step(
    job_id: &str,
    slug: &str,
    step_id: &str,
    command: &str,
    work_dir: &Path,
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    ship_dir: Option<&Path>,
) {
    tracing::info!(slug, step_id, %command, "run-step: executing");

    // Emit step_dispatched
    let dispatched_payload = serde_json::json!({
        "job_id": job_id, "slug": slug,
        "step_id": step_id, "step_type": "run",
    });
    if let Ok(env) = EventEnvelope::new(event_types::STEP_DISPATCHED, job_id, &dispatched_payload) {
        persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), ship_dir).await;
    }

    // Run the command
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(work_dir)
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

            // Emit step_output
            let output_payload = serde_json::json!({
                "job_id": job_id, "step_id": step_id,
                "stdout": if stdout.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(stdout) },
                "stderr": if stderr.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(stderr) },
            });
            if let Ok(env) = EventEnvelope::new(event_types::STEP_OUTPUT, job_id, &output_payload) {
                persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), ship_dir).await;
            }

            if out.status.success() {
                tracing::info!(slug, step_id, "run-step: completed successfully");
                let payload = serde_json::json!({
                    "job_id": job_id, "slug": slug, "step_id": step_id,
                });
                if let Ok(env) = EventEnvelope::new(event_types::STEP_COMPLETED, job_id, &payload) {
                    persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), ship_dir).await;
                }
            } else {
                let exit_code = out.status.code().unwrap_or(-1);
                let stderr_str = String::from_utf8_lossy(&out.stderr);
                tracing::warn!(slug, step_id, exit_code, "run-step: command failed");
                let payload = serde_json::json!({
                    "job_id": job_id, "slug": slug, "step_id": step_id,
                    "error": format!("command exited with code {exit_code}: {stderr_str}"),
                });
                if let Ok(env) = EventEnvelope::new(event_types::STEP_FAILED, job_id, &payload) {
                    persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), ship_dir).await;
                }
            }
        }
        Err(e) => {
            tracing::error!(slug, step_id, "run-step: failed to spawn command: {e}");
            let payload = serde_json::json!({
                "job_id": job_id, "slug": slug, "step_id": step_id,
                "error": format!("failed to spawn sh: {e}"),
            });
            if let Ok(env) = EventEnvelope::new(event_types::STEP_FAILED, job_id, &payload) {
                persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), ship_dir).await;
            }
        }
    }
}
