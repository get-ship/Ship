//! Pipeline-domain helpers that ship-code-free callers (e.g. the dispatch
//! envelope path `run_core::dispatch_one`) still need.
//!
//! `orchestrator::*` is gated behind the `ship-code` feature because the
//! orchestrator runtime + step executor pull in process/session types from
//! ship-code. This module carries the always-on helpers `run_core` and any
//! other harness-free consumer depends on:
//!
//! - [`pipeline_loader`] — resolve + validate + interpolate a job's
//!   pipeline spec from disk. Used by `run_core::dispatch_one` to build
//!   the `pipeline.run.started` envelope.
//! - [`git_helpers`] — `resolve_head_sha` baseline capture used at
//!   dispatch time. Also called from `run_core`.
//!
//! Anything else gated behind `ship-code` lives in the crate-root
//! `orchestrator` module.
pub mod git_helpers;
pub mod pipeline_loader;

#[cfg(test)]
mod pipeline_loader_tests;
