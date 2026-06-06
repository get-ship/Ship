//! MCP domain — method-oriented service trait + state.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/mcp/mod.rs`.

pub mod service;
pub mod state;
pub mod wire_types;
#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
