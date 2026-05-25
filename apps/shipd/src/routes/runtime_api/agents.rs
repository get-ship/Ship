//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::auth::{AuthContext};
use crate::routes::rest_api::{ApiState, MeshResponse};

// ---- Agent control ----

#[derive(Deserialize)]
pub struct SteerAgentRequest {
    pub message: String,
}

/// POST /api/v1/runtime/agents/{slug}/stop — stop a running agent by slug.
///
/// Uses the process registry and executor to terminate the agent process.
pub async fn stop_agent(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let handle = {
        let mut reg = dh.registry.lock().await;
        match reg.take(&slug) {
            Some(h) => h,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
                );
            }
        }
    };

    match dh.executor.stop(&handle).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("stopped: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to stop agent: {e}")) }),
        ),
    }
}

/// POST /api/v1/runtime/agents/{slug}/steer — inject a message into a running agent.
pub async fn steer_agent(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(slug): Path<String>,
    Json(req): Json<SteerAgentRequest>,
) -> impl IntoResponse {
    let dh = match state.dispatch_handle() {
        Some(dh) => dh,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(MeshResponse { ok: false, data: serde_json::json!("dispatch service not initialized") }),
            );
        }
    };

    let reg = dh.registry.lock().await;
    let handle = match reg.get(&slug) {
        Some(h) => h,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("no running agent with slug '{slug}'")) }),
            );
        }
    };

    match dh.executor.send(handle, &req.message).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: serde_json::json!(format!("steered: {slug}")) }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(format!("failed to steer agent: {e}")) }),
        ),
    }
}

