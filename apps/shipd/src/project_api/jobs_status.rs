//! POST /api/v1/agent_sessions/{pipeline_run_id}/status — agent-driven lifecycle updates.
//!
//! MCP's `update_job` tool POSTs here instead of writing to the event store
//! directly. The daemon builds the appropriate `job.*` envelope, persists it
//! through the kernel, and lets the projection rebuild advance.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::project_api::{err_response, ok_response};
use crate::routes::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize)]
pub struct RunStatusReq {
    /// One of: dispatched, gate_requested, gate_passed, gate_failed, blocked,
    /// completed, merged, failed.
    pub status: String,
    pub worktree: Option<String>,
    pub pid: Option<u32>,
    pub gate_agent: Option<String>,
    pub blocker: Option<String>,
    pub error: Option<String>,
}

pub async fn update_job_status(
    State(state): State<ApiState>,
    _auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Path(pipeline_run_id): Path<String>,
    Json(body): Json<RunStatusReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    match update_job_status_core(&state, &pipeline_run_id, body).await {
        Ok(value) => Ok(ok_response(value)),
        Err((code, msg)) => Err(err_response(code, &msg)),
    }
}

/// Core of the job-status command, shared by the REST handler above and the
/// ConnectRPC `CommandOpsService`. Builds the matching `pipeline.*`/`gate.*`
/// envelope from `body.status` and persists it. Errors are transport-neutral
/// `(StatusCode, String)` so each surface maps them to its own wire shape.
pub(crate) async fn update_job_status_core(
    state: &ApiState,
    pipeline_run_id: &str,
    body: RunStatusReq,
) -> Result<serde_json::Value, (StatusCode, String)> {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::{
        PipelineRunBlockedPayload, PipelineRunCompletedPayload, PipelineRunDispatchedPayload, PipelineRunFailedPayload,
        PipelineGateFailedPayload, PipelineGatePassedPayload, PipelineGateRequestedPayload, PipelineRunMergedPayload,
        event_types,
    };

    let require = |field: Option<String>, what: &str| {
        field.ok_or_else(|| (StatusCode::BAD_REQUEST, format!("{what} is required for status '{}'", body.status)))
    };

    let envelope_result = match body.status.as_str() {
        "dispatched" => {
            let worktree = require(body.worktree.clone(), "worktree")?;
            EventEnvelope::new(
                event_types::PIPELINE_RUN_DISPATCHED,
                pipeline_run_id,
                &PipelineRunDispatchedPayload {
                    pipeline_run_id: pipeline_run_id.to_string(),
                    // External status-update endpoint only has pipeline_run_id;
                    // slug is unknown here. Consumers that need filtering by slug
                    // resolve it from the pipeline_run record or the dispatch path.
                    slug: String::new(),
                    worktree,
                    pid: body.pid,
                    harness_meta: None,
                },
            )
        }
        "gate_requested" => {
            let gate_agent = require(body.gate_agent.clone(), "gate_agent")?;
            EventEnvelope::new(
                event_types::PIPELINE_GATE_REQUESTED,
                pipeline_run_id,
                &PipelineGateRequestedPayload {
                    pipeline_run_id: pipeline_run_id.to_string(),
                    gate_agent,
                },
            )
        }
        "gate_passed" => EventEnvelope::new(
            event_types::PIPELINE_GATE_PASSED,
            pipeline_run_id,
            &PipelineGatePassedPayload { pipeline_run_id: pipeline_run_id.to_string() },
        ),
        "gate_failed" => {
            let reason = require(body.error.clone(), "error")?;
            EventEnvelope::new(
                event_types::PIPELINE_GATE_FAILED,
                pipeline_run_id,
                &PipelineGateFailedPayload { pipeline_run_id: pipeline_run_id.to_string(), reason },
            )
        }
        "blocked" => {
            let blocker = require(body.blocker.clone(), "blocker")?;
            EventEnvelope::new(
                event_types::PIPELINE_RUN_BLOCKED,
                pipeline_run_id,
                &PipelineRunBlockedPayload {
                    pipeline_run_id: pipeline_run_id.to_string(),
                    blocker,
                    needs_human: true,
                },
            )
        }
        "completed" => {
            let slug = resolve_slug(state.ship_dir(), pipeline_run_id);
            EventEnvelope::new(
                event_types::PIPELINE_RUN_COMPLETED,
                pipeline_run_id,
                &PipelineRunCompletedPayload { pipeline_run_id: pipeline_run_id.to_string(), slug },
            )
        }
        "merged" => {
            let slug = resolve_slug(state.ship_dir(), pipeline_run_id);
            EventEnvelope::new(
                event_types::PIPELINE_RUN_MERGED,
                pipeline_run_id,
                &PipelineRunMergedPayload { pipeline_run_id: pipeline_run_id.to_string(), slug },
            )
        }
        "failed" => {
            let error = require(body.error.clone(), "error")?;
            EventEnvelope::new(
                event_types::PIPELINE_RUN_FAILED,
                pipeline_run_id,
                &PipelineRunFailedPayload { pipeline_run_id: pipeline_run_id.to_string(), error },
            )
        }
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "unknown status '{other}'. Valid: dispatched, gate_requested, gate_passed, \
                    gate_failed, blocked, completed, merged, failed"
                ),
            ));
        }
    };

    let envelope = envelope_result.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state
        .persist_and_route(&envelope)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(json!({ "pipeline_run_id": pipeline_run_id, "status": body.status }))
}

fn resolve_slug(ship_dir: &std::path::Path, pipeline_run_id: &str) -> String {
    ship_platform::projections::pipeline_run::load_pipeline_runs_at(ship_dir)
        .ok()
        .and_then(|jobs| jobs.get(pipeline_run_id).map(|r| r.slug.clone()))
        .unwrap_or_default()
}

// ---- Step-done declaration ----

#[derive(Deserialize)]
pub struct StepDoneReq {
    pub summary: Option<String>,
    /// Map of `produces.name → artifact_id` for step outputs.
    #[serde(default)]
    pub outputs: std::collections::BTreeMap<String, String>,
}

/// POST /api/v1/agent_sessions/{slug}/step-done — agent declares the current pipeline step
/// complete. Daemon resolves the active step from the projection, builds a
/// `step.done_declared` envelope, and routes it through the kernel.
pub async fn step_done(
    State(state): State<ApiState>,
    _auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Path(slug): Path<String>,
    Json(body): Json<StepDoneReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    match step_done_core(&state, &slug, body).await {
        Ok(value) => Ok(ok_response(value)),
        Err((code, msg)) => Err(err_response(code, &msg)),
    }
}

/// Core of the step-done command, shared by the REST handler and the ConnectRPC
/// `CommandOpsService`. Resolves the active step from the projection and emits
/// `step.done_declared`. Transport-neutral errors.
pub(crate) async fn step_done_core(
    state: &ApiState,
    slug: &str,
    body: StepDoneReq,
) -> Result<serde_json::Value, (StatusCode, String)> {
    let jobs = ship_platform::projections::pipeline_run::load_pipeline_runs_at(state.ship_dir())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let job = jobs
        .values()
        .find(|j| j.slug == slug)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("no job with slug '{slug}'")))?;
    let step_id = job
        .current_step
        .clone()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("no active step for job '{slug}'")))?;

    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::{PipelineStepDoneDeclaredPayload, event_types};

    let payload = PipelineStepDoneDeclaredPayload {
        pipeline_run_id: job.pipeline_run_id.clone(),
        slug: slug.to_string(),
        step_id: step_id.clone(),
        summary: body.summary,
        outputs: body.outputs,
    };
    let envelope =
        EventEnvelope::new(event_types::PIPELINE_STEP_DONE_DECLARED, &job.pipeline_run_id, &payload)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state
        .persist_and_route(&envelope)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(json!({ "slug": slug, "step_id": step_id }))
}

#[cfg(test)]
#[path = "jobs_status_tests.rs"]
mod tests;
