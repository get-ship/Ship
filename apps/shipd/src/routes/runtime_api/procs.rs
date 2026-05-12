//! Split off from runtime_api.rs (mr-04 follow-up).

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use crate::routes::rest_api::{ApiState, MeshResponse};

// ---- Procs / events / daemon status (slice 6) ----

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ProcsQuery {
    pub slug: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/v1/runtime/procs[?slug=X][&limit=N]
///
/// Returns process records from the `process.*` projection. Without
/// `slug`, returns the most recent `limit` records (default 100), sorted
/// newest first.
#[utoipa::path(
    operation_id = "listProcs",
    get,
    path = "/api/v1/procs",
    params(ProcsQuery),
    responses(
        (status = 200, body = MeshResponse),
        (status = 401, description = "unauthorized", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn list_procs(
    State(state): State<ApiState>,
    Query(params): Query<ProcsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100);
    let slug = params.slug.as_deref();
    match super::jobs::list_processes_at(state.ship_dir(), slug) {
        Ok(mut records) => {
            // Newest first; cap to limit.
            records.sort_by(|a, b| b.spawned_at.cmp(&a.spawned_at));
            if records.len() > limit {
                records.truncate(limit);
            }
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "procs": records }),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) }),
        ),
    }
}
