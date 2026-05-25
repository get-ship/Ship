//! Rules domain — method-oriented service trait + state.
//!
//! `service::RulesService` is the L1 service surface (trait + Impl).
//! `state` is the pure file-tree state layer the impl wraps.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/rules/mod.rs`.

pub mod service;
pub mod state;
pub mod wire_types;
#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
