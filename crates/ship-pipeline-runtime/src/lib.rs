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
//! - [`loader`] — always-on helpers: pipeline spec loader (`ensure_pipeline` /
//!   `load_pipeline_file`) and git baseline capture. The dispatch entry path
//!   (`run_core::dispatch_one`) depends on these WITHOUT ship-code.
//! - [`orchestrator`] — the run runtime: handlers, router, run_step,
//!   review_retry, sweep, fanout, replay, dispatch, pipeline_specs, step_io,
//!   step_monitor, projection_reader, spawn_policy, prompt_composer,
//!   gate_rules, pipeline. Gated behind the `ship-code` feature — the
//!   orchestrator depends on process/session types from `ship-code`.
//! - [`orchestrator::emit`] — re-export of the L0 `ship-runner` emit mechanism;
//!   [`orchestrator::dedup`], [`orchestrator::exit_handler`] — re-exports of the
//!   L1 `ship_code::runner` mechanism. Historic `services::orchestrator::*`
//!   import paths keep working for consumers built with `ship-code` on.

pub mod sweep_events;
pub mod log_stream;
// `loader` is the always-on submodule that hosts the pipeline spec loader
// + git baseline helpers used by `run_core::dispatch_one`. The orchestrator
// runtime (which depends on ship-code) is gated below; `run_core` and any
// other harness-free consumer stays on `crate::loader::{pipeline_loader,
// git_helpers}`.
pub mod loader;
#[cfg(feature = "ship-code")]
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
