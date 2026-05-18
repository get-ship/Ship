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
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::emit::persist_and_route;

/// Execute a shell command as a pipeline step.
///
/// Emits `job.step_dispatched`, then runs `sh -c <command>` in `work_dir`.
/// On success: emits `job.step_output` + `job.step_completed`.
/// On failure: emits `job.step_output` + `job.step_failed`.
pub async fn execute_run_step(
    job_id: &str,
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
        "job_id": job_id, "slug": slug,
        "step_id": step_id, "step_type": "run",
    });
    if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_STARTED, job_id, &dispatched_payload) {
        persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), slug, ship_dir).await;
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
            if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_OUTPUT, job_id, &output_payload) {
                persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), slug, ship_dir).await;
            }

            if out.status.success() {
                tracing::info!(slug, step_id, "run-step: completed successfully");
                let payload = serde_json::json!({
                    "job_id": job_id, "slug": slug, "step_id": step_id,
                });
                if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_COMPLETED, job_id, &payload) {
                    persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), slug, ship_dir).await;
                }
            } else {
                let exit_code = out.status.code().unwrap_or(-1);
                let stderr_str = String::from_utf8_lossy(&out.stderr);
                tracing::warn!(slug, step_id, exit_code, "run-step: command failed");
                let payload = serde_json::json!({
                    "job_id": job_id, "slug": slug, "step_id": step_id,
                    "error": format!("command exited with code {exit_code}: {stderr_str}"),
                });
                if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_FAILED, job_id, &payload) {
                    persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), slug, ship_dir).await;
                }
            }
        }
        Err(e) => {
            tracing::error!(slug, step_id, "run-step: failed to spawn command: {e}");
            let payload = serde_json::json!({
                "job_id": job_id, "slug": slug, "step_id": step_id,
                "error": format!("failed to spawn sh: {e}"),
            });
            if let Ok(env) = EventEnvelope::new(event_types::PIPELINE_STEP_FAILED, job_id, &payload) {
                persist_and_route(kernel, env.with_actor_id("service.job-dispatch"), slug, ship_dir).await;
            }
        }
    }
}
