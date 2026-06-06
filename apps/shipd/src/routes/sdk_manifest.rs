//! Thin transport shell for SDK manifest endpoints.
//!
//! Domain logic relocated to `ship_views::sdk_manifest` (Batch C routes-cull).
//! No daemon state — all three handlers are pure reads over embedded JSON.

use axum::{Json, extract::Path as AxumPath, http::StatusCode, response::IntoResponse};
use ship_views::sdk_manifest::{SdkQuery, query_sdk, sdk_root, view_meta};

/// GET /api/v1/views/{name}/meta
pub async fn get_view_meta(AxumPath(name): AxumPath<String>) -> axum::response::Response {
    match view_meta(&name) {
        Ok(meta) => (StatusCode::OK, Json(meta)).into_response(),
        Err(not_found) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "unknown view",
                "available": not_found.available,
            })),
        )
            .into_response(),
    }
}

/// GET /api/v1/sdk
pub async fn get_sdk_root() -> axum::response::Response {
    (StatusCode::OK, Json(sdk_root())).into_response()
}

/// GET /api/v1/sdk/query?q=term
pub async fn query_sdk_handler(
    axum::extract::Query(q): axum::extract::Query<SdkQuery>,
) -> axum::response::Response {
    let resp = query_sdk(q.q);
    (StatusCode::OK, Json(resp)).into_response()
}
