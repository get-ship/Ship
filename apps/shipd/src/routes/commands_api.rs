//! GET /api/v1/commands — slash-command registry for the command bar.
//!
//! Returns the snapshot of `ship_platform::commands::list()`. Built-ins are
//! seeded at daemon boot; runtime additions appear after seeded entries in
//! insertion order.

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::auth::{AuthContext};
use ship_platform::commands::{self, CommandEntry};

#[derive(Serialize)]
pub struct CommandsResponse {
    pub ok: bool,
    pub commands: Vec<CommandEntry>,
}

pub async fn list_commands(
    Extension(_auth): Extension<AuthContext>,
) -> axum::response::Response {
    let commands = commands::list();
    (
        StatusCode::OK,
        Json(CommandsResponse { ok: true, commands }),
    ).into_response()
}
