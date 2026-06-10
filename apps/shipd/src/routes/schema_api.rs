//! `GET /api/v1/ship/schema` handler — the `ship.*` op catalog served live.
//!
//! The catalog types and constants are the single source of truth in
//! `ship_ops_rpc::catalog`; this is the thin HTTP shell that serves them so
//! view templates (`ship.<domain>.<op>`), `window.ship.*` codegen, lint, and
//! MCP tool auto-registration all read one catalog.
//!
//! Consumers:
//! - `@ship/shell` TS SDK: codegen typed `window.ship.*` methods
//! - Lint: flag template calls that don't match a known op
//! - MCP server: auto-register ops as tools
//! - Third-party view authors: editor autocomplete via `/api/v1/ship/schema`

use axum::Json;

use ship_ops_rpc::catalog::{COMMANDS, OPS};

use crate::wire::MeshResponse;

/// `GET /api/v1/ship/schema` — catalog of every `ship.<domain>.<op>` the
/// registry exposes.
pub async fn serve_schema() -> Json<MeshResponse> {
    Json(MeshResponse { ok: true, data: schema_catalog() })
}

/// The `{ ops, commands }` op-catalog value.
pub fn schema_catalog() -> serde_json::Value {
    serde_json::json!({ "ops": OPS, "commands": COMMANDS })
}
