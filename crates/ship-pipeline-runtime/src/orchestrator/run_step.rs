//! Run step executor — executes shell commands as pipeline steps.
//!
//! A run step is a pipeline step with `type: run`. It executes a shell
//! command in the job's worktree directory, capturing output and emitting
//! step lifecycle events to the kernel.
//!
//! On exit 0: emits `job.step_output` then `job.step_completed`.
//! On non-zero exit: emits `job.step_output` then `job.step_failed`.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_substrate::StepExecutor;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::emit::persist_and_route;

/// Execute a shell command as a pipeline step.
///
/// Emits `job.step_dispatched`, then runs the command via the `StepExecutor`
/// port (backed by `spawn_bounded` — 2 GB virtual-memory cap, 30 s timeout).
/// On success: emits `job.step_output` + `job.step_completed`.
/// On failure: emits `job.step_output` + `job.step_failed`.
pub async fn execute_run_step(
    pipeline_run_id: &str,
    slug: &str,
    step_id: &str,
    command: &str,
    work_dir: &Path,
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    ship_dir: Option<&Path>,
) {
    tracing::info!(slug, step_id, %command, "run-step: executing");

    // Emit step_dispatched
    let dispatched_payload = serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug,
        "step_id": step_id, "step_type": "run",
    });
    if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_STARTED, pipeline_run_id, &dispatched_payload) {
        persist_and_route(kernel, env.with_actor_id("service.orchestrator"), slug, ship_dir).await;
    }

    // Declare intent: "run a pipeline step's shell command with bounded resources"
    let result = StepExecutor.run(command, work_dir).await;

    match result {
        Ok(out) => {
            // Emit step_output
            let output_payload = serde_json::json!({
                "pipeline_run_id": pipeline_run_id, "step_id": step_id,
                "stdout": if out.stdout.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(out.stdout.clone()) },
                "stderr": if out.stderr.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(out.stderr.clone()) },
            });
            if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_OUTPUT, pipeline_run_id, &output_payload) {
                persist_and_route(kernel, env.with_actor_id("service.orchestrator"), slug, ship_dir).await;
            }

            if out.success {
                tracing::info!(slug, step_id, "run-step: completed successfully");
                let payload = serde_json::json!({
                    "pipeline_run_id": pipeline_run_id, "slug": slug, "step_id": step_id,
                });
                if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_COMPLETED, pipeline_run_id, &payload) {
                    persist_and_route(kernel, env.with_actor_id("service.orchestrator"), slug, ship_dir).await;
                }
            } else {
                tracing::warn!(slug, step_id, exit_code = out.exit_code, "run-step: command failed");
                let payload = serde_json::json!({
                    "pipeline_run_id": pipeline_run_id, "slug": slug, "step_id": step_id,
                    "error": format!("command exited with code {}: {}", out.exit_code, out.stderr),
                });
                if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_FAILED, pipeline_run_id, &payload) {
                    persist_and_route(kernel, env.with_actor_id("service.orchestrator"), slug, ship_dir).await;
                }
            }
        }
        Err(e) => {
            tracing::error!(slug, step_id, "run-step: step executor error: {e}");
            let payload = serde_json::json!({
                "pipeline_run_id": pipeline_run_id, "slug": slug, "step_id": step_id,
                "error": format!("step executor: {e}"),
            });
            if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_FAILED, pipeline_run_id, &payload) {
                persist_and_route(kernel, env.with_actor_id("service.orchestrator"), slug, ship_dir).await;
            }
        }
    }
}
