//! Thin transport shell for theme contrast endpoint.
//!
//! Domain logic relocated to `ship_views::themes_contrast` (Batch C
//! routes-cull). This file is a minimal axum adapter.

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use ship_views::themes_contrast::ContrastRequest;

use crate::auth::AuthContext;

/// POST /api/v1/themes/contrast
pub async fn theme_contrast(
    Extension(_auth): Extension<AuthContext>,
    Json(req): Json<ContrastRequest>,
) -> axum::response::Response {
    let report = ship_views::themes_contrast::compute_contrast(req);
    (StatusCode::OK, Json(report)).into_response()
}
