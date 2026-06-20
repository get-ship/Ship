//! Pipeline domain view helpers — context builders for daemon-rendered views.
//!
//! Relocated from `shipd::services::pipelines::*` and `shipd::views::state`
//! pipeline sections (L2 → L1). These modules are pure view-composition:
//! they read projections and assemble serializable payloads for templates.

pub mod state;
pub mod view_state;
pub mod shape;

/// Flatten top-level re-exports that callers commonly need.
pub use state::{
    Filter, PipelineDef, RunCategory,
    filter_and_sort, list_pipeline_defs, list_pipeline_runs,
    aggregate_pipeline_stats, partition_runs, status_token, status_tone,
    pipeline_name_for_job_public,
};

