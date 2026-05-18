//! `/api/v1/isolation/*` — capability probe + recommender results.
//!
//! Thin pass-through to `ship_platform::isolation`. Read endpoint is
//! `ReadWorkspace`; rescan is `WriteWorkspace` because it mutates the
//! cached snapshot in `daemon_settings`.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use ship_platform::isolation;

use crate::routes::rest_api::{ApiState, MeshResponse};

#[utoipa::path(
    operation_id = "isolationStatus",
    get,
    path = "/api/v1/isolation/status",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "isolation"
)]
pub async fn isolation_status(State(_state): State<ApiState>) -> impl IntoResponse {
    build_response(false)
}

#[utoipa::path(
    operation_id = "isolationRescan",
    post,
    path = "/api/v1/isolation/rescan",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "isolation"
)]
pub async fn isolation_rescan(State(_state): State<ApiState>) -> impl IntoResponse {
    build_response(true)
}

fn build_response(rescan: bool) -> (StatusCode, Json<MeshResponse>) {
    let caps = if rescan {
        match isolation::rescan_capabilities() {
            Ok(c) => c,
            Err(e) => return error(format!("rescan failed: {e}")),
        }
    } else {
        match isolation::cached_capabilities() {
            Ok(c) => c,
            Err(e) => return error(format!("capability probe failed: {e}")),
        }
    };
    let decision = isolation::decide(None, None);
    let payload = serde_json::json!({
        "capabilities": caps,
        "decision": match &decision {
            Ok(d) => serde_json::to_value(d).unwrap_or(serde_json::Value::Null),
            Err(e) => serde_json::json!({ "error": e.to_string() }),
        },
    });
    (
        StatusCode::OK,
        Json(MeshResponse { ok: true, data: payload }),
    )
}

fn error(msg: String) -> (StatusCode, Json<MeshResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(MeshResponse {
            ok: false,
            data: serde_json::json!({ "error": msg }),
        }),
    )
}
