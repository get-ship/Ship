//! Thin transport shell for view-error ring-buffer endpoints.
//!
//! Domain logic relocated to `ship_views::view_errors` (Batch C routes-cull).
//!
//! # Capability gap
//! No Connect wire covers these three REST routes. The ring buffer is
//! REST-only for now. A future `ViewErrorsService` proto can call
//! `ship_views::view_errors::{record, query, clear}` directly without
//! touching this file.

use axum::{Extension, Json, extract::Query, http::StatusCode, response::IntoResponse};
use ship_views::view_errors::{ListQuery, PostErrorReq, clear, query, record};

use crate::auth::AuthContext;

/// POST /api/v1/view-errors
pub async fn post_error(
    Extension(_auth): Extension<AuthContext>,
    Json(req): Json<PostErrorReq>,
) -> axum::response::Response {
    let id = record(req.into());
    (StatusCode::OK, Json(serde_json::json!({ "id": id, "accepted": true }))).into_response()
}

/// GET /api/v1/view-errors
pub async fn list_errors(
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<ListQuery>,
) -> axum::response::Response {
    let limit = q.limit.unwrap_or(50).min(200);
    let (filtered, total) = query(q.since, q.view.as_deref(), limit);
    (StatusCode::OK, Json(serde_json::json!({ "errors": filtered, "total": total })))
        .into_response()
}

/// DELETE /api/v1/view-errors
pub async fn clear_errors(
    Extension(_auth): Extension<AuthContext>,
) -> axum::response::Response {
    let cleared = clear();
    (StatusCode::OK, Json(serde_json::json!({ "cleared": cleared }))).into_response()
}
