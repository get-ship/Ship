//! Request shapes for the `view.*` MCP tools.
//!
//! Each shape mirrors the corresponding contract type in
//! `ship_platform::view_authoring::types` but uses `rmcp::schemars::JsonSchema`
//! so the MCP runtime can document the tool surface to clients.

use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ViewName {
    /// Name of the view (top segment under `views/`).
    pub view: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ViewWriteSource {
    pub view: String,
    /// New `.sui` source bytes to write.
    pub source: String,
    /// Optional human-facing label for the resulting snapshot.
    #[serde(default)]
    pub turn_label: Option<String>,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct ViewCollectionsSchema {
    /// Optional collection id. Omit to list every project-visible schema.
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct ViewNoArgs {}

#[derive(Deserialize, JsonSchema)]
pub struct ViewAnnotate {
    pub view: String,
    /// CSS selector relative to the view root.
    pub selector: String,
    /// Free-text note shown on the chrome side.
    pub note: String,
    /// Pin styling. Defaults to `info`.
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ViewListSnapshotsArgs {
    pub view: String,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ViewRestoreSnapshotArgs {
    pub view: String,
    pub snapshot_id: String,
}
