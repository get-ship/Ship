//! Agent-facing job-status + step-done REST handlers.
//!
//! `POST /api/v1/agent_sessions/{pipeline_run_id}/status` and
//! `POST /api/v1/agent_sessions/{slug}/step-done`. Thin transport adapters —
//! build a `PipelineDispatchAdapter` and delegate to the L1 cores in
//! `ship_pipeline_runtime::status_core`, mapping `StatusCoreError` onto the
//! shared `MeshResponse` envelope. Mirrors the `end_job` precedent.

use axum::{Json, extract::{Path, State}, http::StatusCode};

use ship_pipeline_runtime::status_core::{
    RunStatusReq, StatusCoreError, StepDoneReq, step_done_core, update_job_status_core,
};

use crate::pipeline_ports::PipelineDispatchAdapter;
use crate::routes::mesh_resp::{err_response, ok_response};
use crate::routes::rest_api::{ApiState, MeshResponse};

fn map_status_err(e: StatusCoreError) -> (StatusCode, Json<MeshResponse>) {
    match e {
        StatusCoreError::NotFound(m) => err_response(StatusCode::NOT_FOUND, &m),
        StatusCoreError::BadRequest(m) => err_response(StatusCode::BAD_REQUEST, &m),
        StatusCoreError::Internal(m) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &m),
    }
}

/// POST /api/v1/agent_sessions/{pipeline_run_id}/status — agent-driven
/// lifecycle updates. MCP's `update_job` tool POSTs here; the daemon builds the
/// matching `pipeline.*`/`gate.*` envelope and routes it through the kernel.
pub async fn update_job_status(
    State(state): State<ApiState>,
    _auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Path(pipeline_run_id): Path<String>,
    Json(body): Json<RunStatusReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let port = PipelineDispatchAdapter::new(state);
    match update_job_status_core(&port, &pipeline_run_id, body).await {
        Ok(value) => Ok(ok_response(value)),
        Err(e) => Err(map_status_err(e)),
    }
}

/// POST /api/v1/agent_sessions/{slug}/step-done — agent declares the current
/// pipeline step complete. The daemon resolves the active step from the
/// projection and emits `step.done_declared`.
pub async fn step_done(
    State(state): State<ApiState>,
    _auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Path(slug): Path<String>,
    Json(body): Json<StepDoneReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let port = PipelineDispatchAdapter::new(state);
    match step_done_core(&port, &slug, body).await {
        Ok(value) => Ok(ok_response(value)),
        Err(e) => Err(map_status_err(e)),
    }
}

#[cfg(test)]
#[path = "jobs_status_tests.rs"]
mod tests;
