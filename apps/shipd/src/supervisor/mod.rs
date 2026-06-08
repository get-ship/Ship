//! Test scaffolding for the supervisor crate.
//!
//! Production supervisor code lives in `crates/core/ship-supervisor/`. This
//! module declares the integration tests that exercise the supervisor
//! together with shipd internals (`ApiState`, `ShipRuntime`, test
//! harness, etc.) — those tests can't move to the supervisor crate
//! because they depend on shipd surface.
//!
//! Tests that don't need shipd internals belong inside the supervisor
//! crate itself (move them when added).
//!
//! `#[cfg(test)]` is applied at the declaring site (`lib.rs`); no
//! inner `#![cfg(test)]` needed.

#[path = "tests_service_architecture.rs"]
mod tests_service_architecture;

#[path = "tests_daemon_startup.rs"]
mod tests_daemon_startup;

#[path = "tests_orphan_cleanup.rs"]
mod tests_orphan_cleanup;

#[path = "tests_graceful_shutdown.rs"]
mod tests_graceful_shutdown;

#[path = "tests_executor_events.rs"]
mod tests_executor_events;

#[path = "tests_pipeline_e2e.rs"]
mod tests_pipeline_e2e;
