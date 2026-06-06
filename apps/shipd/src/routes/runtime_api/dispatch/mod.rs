//! Single-agent dispatch — `POST /api/v1/agent_sessions/dispatch`.
//!
//! This module is a thin HTTP transport adapter. All dispatch domain logic
//! lives in `ship-agents::services::agents::dispatch_core`. The route handler
//! enforces the MCP workspace guard (transport-boundary auth concern) then
//! delegates to the domain core via `AgentDispatchPort`.
//!
//! `dispatch_one` remains here for `pipeline_run.rs` (Batch B target) which
//! needs it until pipeline dispatch moves to ship-pipeline-runtime.

pub mod request;
pub mod resume;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_typed_result;

pub use request::DispatchRequest;

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use ship_agents::services::agents::dispatch_core::{
    AgentDispatchPort, DispatchCoreError, dispatch_agent_session_core,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, event_types};

use crate::agent_event_ports::AgentDispatchAdapter;
use crate::auth::AuthContext;
use crate::routes::rest_api::{ApiState, MeshResponse};

/// POST /api/v1/agent_sessions/dispatch
///
/// Agent-centric single-agent dispatch. Rejects payloads carrying a
/// `pipeline` field (those belong at `POST /api/v1/pipelines/run`).
pub async fn dispatch_agent_session(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<DispatchRequest>,
) -> impl IntoResponse {
    // Per-project enforcement teeth: an MCP session bound to project A
    // cannot dispatch work into project B. Trusted (CLI/SDK) callers and
    // unbound sessions pass through. See auth::identity. This guard lives in
    // the REST handler because it needs the request's AuthContext; the
    // transport-neutral core below is shared with the ConnectRPC path.
    if let Some(message) =
        crate::auth::identity::mcp_workspace_violation(&auth, req.workspace.as_deref())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(message),
            }),
        );
    }

    let port = AgentDispatchAdapter::new(state);
    match dispatch_agent_session_core(&port, req).await {
        Ok(outcome) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({
                    "session_id": outcome.session_id,
                    "slug": outcome.slug,
                    "branch": outcome.branch,
                    "resume": outcome.resume,
                    "studio_url": outcome.studio_url,
                    "billing_warning": outcome.billing_warning,
                }),
            }),
        ),
        Err(DispatchCoreError::BadRequest(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse { ok: false, data: serde_json::json!(msg) }),
        ),
        Err(DispatchCoreError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(msg) }),
        ),
    }
}

/// Shared dispatch primitive — builds the `pipeline.run.started` envelope
/// from `payload`, runs `ensure_pipeline` (so multi-step callers get their
/// DAG validated/interpolated), persists + routes it, and returns the
/// standard `MeshResponse` body.
///
/// Callable from the pipeline-run endpoint. Stays here until Batch B moves
/// pipeline dispatch into ship-pipeline-runtime.
pub(crate) async fn dispatch_one(
    state: &ApiState,
    payload: PipelineRunStartedPayload,
    pipeline_run_id: &str,
) -> (StatusCode, Json<MeshResponse>) {
    let payload =
        match ship_pipeline_runtime::orchestrator::pipeline_loader::ensure_pipeline(payload) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(MeshResponse { ok: false, data: serde_json::json!(e) }),
                );
            }
        };

    let envelope = match EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, pipeline_run_id, &payload)
        .map(|e| e.with_actor_id("studio"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(e.to_string()),
                }),
            );
        }
    };

    match state.persist_and_route(&envelope).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({
                    "pipeline_run_id": pipeline_run_id,
                    "slug": payload.slug,
                    "branch": payload.branch,
                    "resume": payload.resume,
                }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}
