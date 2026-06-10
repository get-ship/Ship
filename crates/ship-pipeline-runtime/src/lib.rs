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

pub mod log_stream;
pub mod orchestrator;
pub mod pipeline_ports;
pub mod pipelines;
pub mod projections;
pub mod read_core;
pub mod run_core;
pub mod status_core;
pub mod view;
pub mod watch;
pub mod wire;

use std::sync::Once;

static INSTALL_PROJECTIONS: Once = Once::new();

/// Register the pipeline-run projection with the substrate event bus. Each
/// domain contributes its own projections through the shared substrate API;
/// this is the pipeline domain's contribution. Idempotent — call once at app
/// startup, alongside `ship_platform::install_defaults`.
pub fn install_projections() {
    INSTALL_PROJECTIONS.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![
                Box::new(projections::PipelineRunProjection),
                // P0.1 — the normalized per-step read model. Additive: the
                // run-level projection above is untouched. Folds the durable
                // step events + child-session terminals into pipeline_step_runs.
                Box::new(projections::PipelineStepRunProjection),
            ]
        }));
    });
}
