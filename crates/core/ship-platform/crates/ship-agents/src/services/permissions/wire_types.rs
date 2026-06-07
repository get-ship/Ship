//! Named wire shapes for `permissions.*` ops (formerly anonymous `json!`).
//! Sibling to `state.rs` per the typed-op single-source convention.

use serde::Serialize;

use super::state::{IndexEntry, ScopeTag};

/// Wire shape of `ship.permissions.list()`.
#[derive(Serialize)]
pub struct PermissionsIndexResponse {
    pub items: Vec<IndexEntry>,
}

/// Wire shape of `ship.permissions.get()` — raw preset at a resolved scope.
#[derive(Serialize)]
pub struct PermissionGetResponse {
    pub id: String,
    pub scope: ScopeTag,
    pub entry: serde_json::Value,
}

/// One tool-pattern capability row for the agent builder autocomplete.
#[derive(Serialize)]
pub struct ToolPattern {
    pub pattern: String,
    pub label: String,
    pub source: String,
}

/// Wire shape of `ship.permissions.tool_patterns()`.
#[derive(Serialize)]
pub struct ToolPatternsResponse {
    pub patterns: Vec<ToolPattern>,
}
