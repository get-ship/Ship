//! Pipelines domain — method-oriented service trait + state core.
//!
//! View-rendering helpers (focus, history, run_detail, shape, summaries,
//! view_state) stay in shipd because they depend on shipd's view types.
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/pipelines/mod.rs`.

pub mod service;
pub mod state;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
