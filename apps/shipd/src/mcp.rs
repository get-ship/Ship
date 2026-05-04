//! MCP transport — apps bridge + per-server registry.
//!
//! Modular-runtime-04 PR 3: collect MCP-specific transport under one
//! umbrella. `apps_bridge.rs` translates HTTP requests to MCP tool
//! invocations; `registry/` tracks per-server lifecycle.

pub mod apps_bridge;
pub mod registry;
