//! `/api/v1/isolation/*` — capability probe + recommender results +
//! wizard endpoints (preview, recommendations, verify, init).
//!
//! Thin pass-through to `ship_platform::isolation` + `isolation_ops`.
//! Reads are `ReadWorkspace`; writes (rescan, verify which writes the
//! cached snapshot side-effectfully via decide(), init which mutates
//! daemon_settings) are `WriteWorkspace`.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use ship_platform::{isolation, isolation_ops};

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

#[utoipa::path(
    operation_id = "isolationRecommendations",
    get,
    path = "/api/v1/isolation/recommendations",
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "isolation"
)]
pub async fn isolation_recommendations(State(_state): State<ApiState>) -> impl IntoResponse {
    match isolation_ops::recommendations() {
        Ok(r) => ok_json(serde_json::to_value(&r).unwrap_or_default()),
        Err(e) => error(format!("recommendations failed: {e}")),
    }
}

#[derive(Debug, Deserialize)]
pub struct PreviewRequest {
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub strategy: Option<String>,
}

#[utoipa::path(
    operation_id = "isolationPreview",
    post,
    path = "/api/v1/isolation/preview",
    request_body = serde_json::Value,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "isolation"
)]
pub async fn isolation_preview(
    State(_state): State<ApiState>,
    Json(req): Json<PreviewRequest>,
) -> impl IntoResponse {
    let tier = req
        .tier
        .as_deref()
        .and_then(ship_isolation::tier::IsolationTier::from_id);
    let strategy = req
        .strategy
        .as_deref()
        .and_then(ship_isolation::tier::IsolationStrategy::from_id);
    let profile = ship_isolation::tier::IsolationProfile {
        tier,
        strategy,
        policy: None,
    };
    match isolation_ops::preview(profile) {
        Ok(decision) => ok_json(serde_json::to_value(&decision).unwrap_or_default()),
        Err(e) => error(format!("preview failed: {e}")),
    }
}

#[utoipa::path(
    operation_id = "isolationVerify",
    post,
    path = "/api/v1/isolation/verify",
    responses(
        (status = 200, body = MeshResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "isolation"
)]
pub async fn isolation_verify(State(_state): State<ApiState>) -> impl IntoResponse {
    let result = isolation_ops::verify_decision();
    ok_json(serde_json::to_value(&result).unwrap_or_default())
}

#[utoipa::path(
    operation_id = "isolationInit",
    post,
    path = "/api/v1/isolation/init",
    request_body = serde_json::Value,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "isolation"
)]
pub async fn isolation_init(
    State(_state): State<ApiState>,
    Json(req): Json<isolation_ops::InitRequest>,
) -> impl IntoResponse {
    match isolation_ops::init(req) {
        Ok(r) => {
            let status = if r.ok { StatusCode::OK } else { StatusCode::BAD_REQUEST };
            (
                status,
                Json(MeshResponse {
                    ok: r.ok,
                    data: serde_json::to_value(&r).unwrap_or_default(),
                }),
            )
        }
        Err(e) => error(format!("init failed: {e}")),
    }
}

fn ok_json(data: serde_json::Value) -> (StatusCode, Json<MeshResponse>) {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}
