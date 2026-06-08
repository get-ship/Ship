//! Pipeline-run projections — read models the runtime owns.
//!
//! Relocated from the platform `projections/` grab-bag so the pipeline domain
//! owns its own read model. The projection *framework* (the `Projection`
//! trait, `EventBus`) stays in `ship_substrate::projections`.

pub mod pipeline_run;
pub mod pipeline_run_transitions;

pub use pipeline_run::{
    PipelineRun, PipelineRunProjection, PipelineRunStatus, load_pipeline_run_by_id,
    load_pipeline_run_by_id_at, load_pipeline_run_by_slug, load_pipeline_run_by_slug_at,
    load_pipeline_runs, load_run_events_at, project as project_pipeline_runs,
};
