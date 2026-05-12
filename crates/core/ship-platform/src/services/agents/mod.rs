//! Agents domain — method-oriented service trait + state + view discovery.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/agents/mod.rs`.

pub mod create;
pub mod service;
pub mod settings_view;
pub mod state;
pub mod view_state;
pub mod wire;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "view_state_tests.rs"]
mod view_state_tests;
