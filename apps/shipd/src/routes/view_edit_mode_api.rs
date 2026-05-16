//! REST handlers for the view-edit-mode handshake.
//!
//!   POST /api/v1/views/{name}/edit-mode/enter   body: { session_id }
//!   POST /api/v1/views/{name}/edit-mode/exit    body: { session_id }
//!
//! Grants the calling session access to the `view.*` MCP tools scoped to
//! this view. State is in-memory (process-singleton in `ship_platform::
//! view_authoring::capability`) — cleared on daemon restart, on `/exit`,
//! and on explicit session end.

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::auth::AuthContext;
use ship_platform::view_authoring::{self, ViewEditModeAck};

pub fn is_valid_view_name_pub(name: &str) -> bool {
    is_valid_view_name(name)
}

fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

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

#[utoipa::path(
    operation_id = "enterViewEditMode",
    post,
    path = "/api/v1/views/{name}/edit-mode/enter",
    request_body(
        content_type = "application/json",
        description = "Grant the calling session view-edit access to {name}"
    ),
    responses(
        (status = 200, description = "Edit-mode grant recorded"),
        (status = 400, description = "Invalid view name or body"),
        (status = 401, description = "Unauthenticated"),
    ),
    tag = "view-authoring"
)]
pub async fn enter_edit_mode(
    Path(name): Path<String>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<ViewEditModeEnterBody>,
) -> Response {
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
    let ack = ViewEditModeAck {
        view: name,
        session_id,
        granted: true,
    };
    (StatusCode::OK, Json(ack)).into_response()
}

#[utoipa::path(
    operation_id = "exitViewEditMode",
    post,
    path = "/api/v1/views/{name}/edit-mode/exit",
    request_body(
        content_type = "application/json",
        description = "Revoke the calling session's view-edit grant for {name}"
    ),
    responses(
        (status = 200, description = "Edit-mode grant revoked (idempotent)"),
        (status = 400, description = "Invalid view name or body"),
    ),
    tag = "view-authoring"
)]
pub async fn exit_edit_mode(
    Path(name): Path<String>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<ViewEditModeExitBody>,
) -> Response {
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
    // Idempotent — revoking a missing grant is fine.
    view_authoring::revoke(&session_id, &name);
    let ack = ViewEditModeAck {
        view: name,
        session_id,
        granted: false,
    };
    (StatusCode::OK, Json(ack)).into_response()
}

// Wire bodies are local newtypes around the contract structs so axum's
// JSON extractor matches `Deserialize` without dragging the contract
// crate's serde derives into a code-gen path.
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
