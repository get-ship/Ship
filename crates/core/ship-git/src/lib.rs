//! Git domain — repository operations, status, diffs.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/git/mod.rs` (or the routes/ layer).

pub mod diff;
pub mod read;
pub mod service;
pub mod state;
pub mod write;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
