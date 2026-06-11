//! Integration tests exercising shipd's dispatch and executor machinery
//! (ship-code + ship-agents + ship-pipeline-runtime) together with shipd
//! internals (`ApiState`, `ShipRuntime`, test harness, etc.) — they live
//! here because they depend on shipd surface. (The historic ship-supervisor
//! crate this doc once pointed at was deleted.)
//!
//! Tests that don't need shipd internals belong inside the supervisor
//! crate itself (move them when added).
//!
//! `#[cfg(test)]` is applied at the declaring site (`lib.rs`); no
//! inner `#![cfg(test)]` needed.

#[path = "tests_daemon_startup.rs"]
mod tests_daemon_startup;

#[path = "tests_orphan_cleanup.rs"]
mod tests_orphan_cleanup;

#[path = "tests_executor_events.rs"]
mod tests_executor_events;

#[path = "tests_pipeline_e2e.rs"]
mod tests_pipeline_e2e;
