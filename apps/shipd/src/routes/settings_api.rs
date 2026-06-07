//! Thin transport shell — harness / provider listing endpoint.
//!
//! Domain logic lives in `ship_agents::agent_domain::harnesses::list_providers`.
//! This handler is a one-liner: pull the cached detection result and wrap it
//! in the standard `MeshResponse` envelope.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::routes::rest_api::{ApiState, MeshResponse};

pub async fn list_harnesses(
    State(_state): State<ApiState>,
) -> impl IntoResponse {
    let harnesses = ship_agents::agent_domain::harnesses::list_providers();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!(harnesses),
        }),
    )
}
