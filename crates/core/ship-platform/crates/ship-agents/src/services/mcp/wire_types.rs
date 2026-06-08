//! Named wire shapes for `mcp.*` ops (formerly anonymous `json!`).
//! Sibling to `state.rs` per the typed-op single-source convention.

use serde::Serialize;

use super::state::{IndexEntry, ScopeTag};

/// Wire shape of `ship.mcp.list()`.
#[derive(Serialize)]
pub struct McpIndexResponse {
    pub items: Vec<IndexEntry>,
}

/// Wire shape of `ship.mcp.get()` — raw MCP entry at a resolved scope.
#[derive(Serialize)]
pub struct McpGetResponse {
    pub id: String,
    pub scope: ScopeTag,
    pub entry: serde_json::Value,
}
