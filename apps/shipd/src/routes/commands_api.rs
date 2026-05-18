//! GET /api/v1/commands — slash-command registry for the command bar.
//!
//! Returns the snapshot of `ship_platform::commands::list()`. Built-ins are
//! seeded at daemon boot; runtime additions appear after seeded entries in
//! insertion order.

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::auth::{AuthContext, Scope, require_scope};
use ship_platform::commands::{self, CommandEntry};

#[derive(Serialize, utoipa::ToSchema)]
pub struct CommandsResponse {
    pub ok: bool,
    pub commands: Vec<CommandEntry>,
}

#[utoipa::path(
    operation_id = "listCommands",
    get,
    path = "/api/v1/commands",
    responses(
        (status = 200, description = "Registered slash commands", body = CommandsResponse),
    ),
    tag = "framework"
)]
pub async fn list_commands(
    Extension(auth): Extension<AuthContext>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    let commands = commands::list();
    (
        StatusCode::OK,
        Json(CommandsResponse { ok: true, commands }),
    ).into_response()
}
