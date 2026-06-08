//! Shared `MeshResponse` builders.
//!
//! `ok_response`/`err_response` are the single source of truth for the
//! `(StatusCode, Json<MeshResponse>)` success/error envelope returned by the
//! `/api/v1/*` JSON handlers across `routes/` (e.g. the relocated jobs-status
//! handlers in `runtime_api`), so every handler imports them from one place.

use axum::{Json, http::StatusCode};
use serde_json::json;

use crate::wire::MeshResponse;

pub(crate) fn ok_response(data: serde_json::Value) -> (StatusCode, Json<MeshResponse>) {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

pub(crate) fn err_response(status: StatusCode, msg: &str) -> (StatusCode, Json<MeshResponse>) {
    (
        status,
        Json(MeshResponse {
            ok: false,
            data: json!(msg),
        }),
    )
}
