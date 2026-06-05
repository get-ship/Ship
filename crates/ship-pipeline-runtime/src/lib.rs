//! Pipeline runtime — CRUD over the pipeline spec tree plus the job
//! advancement / step dispatch / fanout / sweep / merge runtime.
//!
//! This crate absorbs the historic `ship_platform::services::pipelines` and
//! `ship_platform::services::orchestrator` modules. The orchestrator is NOT
//! a service: it owns no `orchestrator.*` events and is not registered in
//! the shipd service registry. It is the internal runtime of the pipeline
//! system — it authors `pipeline.*` events and advances runs.
//!
//! # Layout
//!
//! - [`pipelines`] — `PipelinesService` (CRUD over `<scope>/pipelines/*.jsonc`)
//!   and the spec state core. Owns the file I/O via `ship_platform::FilesService`.
//! - [`orchestrator`] — the run runtime: handlers, router, run_step,
//!   review_retry, sweep, fanout, replay, dispatch, pipeline_loader,
//!   pipeline_specs, step_io, step_monitor, projection_reader, spawn_policy,
//!   prompt_composer, gate_rules, git_helpers, pipeline.
//! - [`orchestrator::emit`], [`orchestrator::dedup`], [`orchestrator::exit_handler`]
//!   — re-exports of the L0 `ship-runner` mechanism so the historic
//!   `services::orchestrator::emit::*` import paths keep working.

pub mod orchestrator;
pub mod pipelines;
