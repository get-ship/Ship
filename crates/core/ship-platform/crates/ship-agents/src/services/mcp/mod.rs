//! MCP domain — method-oriented service trait + state.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/mcp/mod.rs`.

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
pub mod service;
pub mod state;
pub mod wire;
pub mod wire_types;

pub use wire::McpWire;
