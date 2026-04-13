//! Request types for the `view.*` MCP tool family.
//!
//! Each tool delegates to the corresponding daemon REST endpoint; these
//! structs match the JSON body shape of those endpoints (see
//! `apps/shipd/src/view_sessions_api.rs`).

use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ViewOpenRequest {
    /// Registered minijinja template (bundled first-party; no inline HTML).
    pub template: String,
    /// Optional human-facing title shown in canvas chrome.
    pub title: Option<String>,
    /// Optional JSON context passed to the template.
    pub initial_context: Option<serde_json::Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ViewPatchRequest {
    pub view_id: String,
    pub selector: String,
    pub html: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ViewAppendRequest {
    pub view_id: String,
    pub selector: String,
    pub html: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ViewSignalRequest {
    pub view_id: String,
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct ViewCloseRequest {
    pub view_id: String,
}
