//! Rules domain — method-oriented service trait + state.
//!
//! `service::RulesService` is the L1 service surface (trait + Impl).
//! `state` is the pure file-tree state layer the impl wraps.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/rules/mod.rs`.

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
pub mod service;
pub mod state;
pub mod wire;
pub mod wire_types;

pub use wire::RulesWire;
