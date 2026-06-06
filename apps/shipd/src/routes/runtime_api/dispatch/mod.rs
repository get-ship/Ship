//! Single-agent dispatch — `POST /api/v1/agent_sessions/dispatch`.
//!
//! This module is a thin HTTP transport adapter. All dispatch domain logic
//! lives in `ship-agents::services::agents::dispatch_core`. The route handler
//! enforces the MCP workspace guard (transport-boundary auth concern) then
//! delegates to the domain core via `AgentDispatchPort`.
//!
//! `dispatch_one` has moved to `ship_pipeline_runtime::run_core::dispatch_one`.

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

