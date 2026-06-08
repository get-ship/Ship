#![allow(clippy::doc_overindented_list_items)]
//! `ship.*` op catalog — the single source of truth for what view
//! templates (`ship.<domain>.<op>`), `window.ship.*` codegen, and MCP
//! tool auto-registration all read.
//!
//! The catalog types and constants now live in `ship-ops-rpc::catalog` so CLI
//! and gateway can reach them without depending on the daemon. This module
//! re-exports them for backward-compat and hosts the thin HTTP handler.
//!
//! Served at `GET /api/v1/ship/schema`.

pub use ship_ops_rpc::catalog::{OpDescriptor, ArgDescriptor, CommandDescriptor, COMMANDS, OPS};

use axum::Json;
use crate::wire::MeshResponse;

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
