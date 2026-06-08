//! Thin transport shell for view-edit-mode REST endpoints.
//!
//! Domain logic (capability grants) is in `ship_views::view_authoring::capability`.
//! These handlers were already delegating there; the file now exists purely
//! as an axum adapter with no logic of its own.
//!
//!   POST /api/v1/views/{name}/edit-mode/enter   body: { session_id }
//!   POST /api/v1/views/{name}/edit-mode/exit    body: { session_id }

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::auth::{AuthContext, Scope, require_scope};
use ship_views::view_authoring::{self, ViewEditModeAck};
use ship_views::view_authoring::tool_dispatch::is_valid_view_name;

/// Re-exported for callers that need the name-validation predicate.
pub use ship_views::view_authoring::tool_dispatch::is_valid_view_name as is_valid_view_name_pub;

fn err(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "ok": false,
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

pub async fn enter_edit_mode(
    Path(name): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(body): Json<ViewEditModeEnterBody>,
) -> Response {
    // ship.views.write (ViewService.EnterEditMode) — view-authoring write.
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    if !is_valid_view_name(&name) {
        return err(StatusCode::BAD_REQUEST, "invalid_view_name", "invalid view name");
    }
    let session_id = body.session_id.trim().to_string();
    if session_id.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "session_id_required",
            "session_id must be a non-empty string",
        );
    }
    view_authoring::grant(&session_id, &name);
    let ack = ViewEditModeAck { view: name, session_id, granted: true };
    (StatusCode::OK, Json(ack)).into_response()
}

pub async fn exit_edit_mode(
    Path(name): Path<String>,
    Extension(auth): Extension<AuthContext>,
    Json(body): Json<ViewEditModeExitBody>,
) -> Response {
    // ship.views.write (ViewService.ExitEditMode) — view-authoring write.
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    if !is_valid_view_name(&name) {
        return err(StatusCode::BAD_REQUEST, "invalid_view_name", "invalid view name");
    }
    let session_id = body.session_id.trim().to_string();
    if session_id.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "session_id_required",
            "session_id must be a non-empty string",
        );
    }
    view_authoring::revoke(&session_id, &name);
    let ack = ViewEditModeAck { view: name, session_id, granted: false };
    (StatusCode::OK, Json(ack)).into_response()
}

#[derive(serde::Deserialize)]
pub struct ViewEditModeEnterBody {
    pub session_id: String,
}

#[derive(serde::Deserialize)]
pub struct ViewEditModeExitBody {
    pub session_id: String,
}

#[cfg(test)]
#[path = "view_edit_mode_api_tests.rs"]
mod tests;
