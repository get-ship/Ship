//! Thin transport shell for the `view.*` MCP tool dispatch endpoint.
//!
//! Domain logic relocated to `ship_views::view_authoring::tool_dispatch`
//! (Batch C routes-cull). Daemon capabilities injected via `ViewToolAdapter`
//! implementing `ship_views::view_ports::ViewToolPort`.
//!
//! POST /api/v1/views/{name}/edit/tool
//!   body: { session_id, op, args }

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::ApiState;
use crate::view_ports::ViewToolAdapter;
use ship_views::view_authoring::tool_dispatch::{DispatchResult, dispatch};

#[derive(serde::Deserialize)]
pub struct ToolBody {
    pub session_id: String,
    pub op: String,
    #[serde(default = "default_args")]
    pub args: serde_json::Value,
}

fn default_args() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

pub async fn dispatch_tool(
    Path(name): Path<String>,
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(body): Json<ToolBody>,
) -> Response {
    // ship.views.write (ViewService.Tool) — view-authoring write.
    if let Err(e) = require_scope(&auth, Scope::ManageRuns) {
        return e.into_response();
    }
    dispatch_tool_core(&state, name, body).await
}

/// Shared core for the REST handler and the ViewWire `tool_fn` closure.
pub(crate) async fn dispatch_tool_core(state: &ApiState, name: String, body: ToolBody) -> Response {
    let adapter = ViewToolAdapter::new(state.clone());
    let result = dispatch(&adapter, name, body.session_id, body.op, body.args).await;
    dispatch_result_to_response(result)
}

fn dispatch_result_to_response(r: DispatchResult) -> Response {
    match r {
        DispatchResult::Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        DispatchResult::BadRequest { code, message } => err(StatusCode::BAD_REQUEST, code, &message),
        DispatchResult::NotFound { code, message } => err(StatusCode::NOT_FOUND, code, &message),
        DispatchResult::Forbidden { code, message } => err(StatusCode::FORBIDDEN, code, &message),
        DispatchResult::Conflict { code, message } => err(StatusCode::CONFLICT, code, &message),
        DispatchResult::Internal { code, message } => {
            err(StatusCode::INTERNAL_SERVER_ERROR, code, &message)
        }
    }
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

#[cfg(test)]
#[path = "view_edit_tool_api_tests.rs"]
mod tests;
