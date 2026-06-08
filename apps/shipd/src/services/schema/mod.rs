#![allow(clippy::doc_overindented_list_items)]
//! `ship.*` op catalog — the single source of truth for what view
//! templates (`ship.<domain>.<op>`), `window.ship.*` codegen, and MCP
//! tool auto-registration all read.
//!
//! Submodules:
//! - `types`    — OpDescriptor, ArgDescriptor, CommandDescriptor structs
//! - `commands` — COMMANDS const (the full mutation catalog)
//!
//! Served at `GET /api/v1/ship/schema`.

pub mod types;
mod commands;

pub use types::{OpDescriptor, ArgDescriptor, CommandDescriptor};
pub use commands::COMMANDS;

use axum::Json;
use crate::wire::MeshResponse;

/// Read-side op catalog. All domains have migrated to typed Connect RPCs;
/// this slice is empty and kept only for xtask / SDK codegen compatibility.
pub const OPS: &[OpDescriptor] = &[];

/// `GET /api/v1/ship/schema` — catalog of every `ship.<domain>.<op>` the
/// registry exposes. Consumers:
/// - `@ship/shell` TS SDK: codegen typed `window.ship.*` methods
/// - Lint: flag template calls that don't match a known op
/// - MCP server: auto-register ops as tools
/// - Third-party view authors: editor autocomplete via `/api/v1/ship/schema`
pub async fn serve_schema() -> Json<MeshResponse> {
    Json(MeshResponse { ok: true, data: schema_catalog() })
}

/// The `{ ops, commands }` op-catalog value, shared by the REST handler
/// (until #137) and `DiscoveryService.Schema` (#141).
pub fn schema_catalog() -> serde_json::Value {
    serde_json::json!({ "ops": OPS, "commands": COMMANDS })
}
