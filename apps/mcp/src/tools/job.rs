//! MCP tool handlers for job lifecycle management.
//!
//! These handlers are pure transport — they POST to the daemon's REST API
//! and return its response. The daemon owns event construction, persistence,
//! kernel routing, projection updates, and audit. Routing through the
//! daemon is what makes projection-rebuilds and pipeline advancement happen
//! on the same single rail used by every other writer.

use crate::network_client;
use crate::requests::{CreateJobRequest, StepDoneRequest, UpdateJobRequest};

/// Create a job by delegating to the daemon's REST API.
pub async fn create_job(req: CreateJobRequest) -> String {
    let body = serde_json::json!({
        "slug": req.slug,
        "agent": req.agent,
        "branch": req.branch,
        "spec_path": req.spec_path,
        "depends_on": req.depends_on,
    });
    match network_client::create_job_via_daemon(body).await {
        Ok(resp) => resp,
        Err(e) => format!("Error creating job: {e}"),
    }
}

/// Forward a job-status transition to the daemon. The daemon validates the
/// status, builds the appropriate `job.*` envelope, persists it, and routes
/// it through the kernel — driving projection updates and pipeline
/// advancement through the same code path operators use.
pub async fn update_job(req: UpdateJobRequest) -> String {
    let body = serde_json::json!({
        "status": req.status,
        "worktree": req.worktree,
        "pid": req.pid,
        "gate_agent": req.gate_agent,
        "blocker": req.blocker,
        "error": req.error,
    });
    match network_client::update_job_status_via_daemon(&req.job_id, body).await {
        Ok(_) => format!("Job {} updated to status '{}'", req.job_id, req.status),
        Err(e) => format!("Error updating job: {e}"),
    }
}

/// Declare the current pipeline step complete.
///
/// Reads `SHIP_MESH_ID` to identify the job and forwards to the daemon's
/// step-done endpoint. The daemon resolves the active step from its
/// projection and emits `step.done_declared` through the kernel.
pub async fn step_done(req: StepDoneRequest) -> String {
    let slug = match runtime::env::read_raw(runtime::env::Var::MeshId) {
        Ok(id) if !id.is_empty() => id,
        _ => {
            return "Error: SHIP_MESH_ID not set — step_done can only be called from a \
                    dispatched job"
                .to_string();
        }
    };
    let body = serde_json::json!({
        "summary": req.summary,
        "outputs": req.outputs,
    });
    match network_client::job_step_done_via_daemon(&slug, body).await {
        Ok(_) => format!("Step marked done for job '{slug}'"),
        Err(e) => format!("Error marking step done: {e}"),
    }
}
