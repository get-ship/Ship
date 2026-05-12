//! Skills domain — method-oriented service trait + state + view discovery.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/skills/mod.rs`.

pub mod service;
pub mod state;
pub mod view_state;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "view_state_tests.rs"]
mod view_state_tests;
