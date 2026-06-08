//! Pipelines view-data helpers.
//!
//! The context builders and domain state live in `ship-pipeline-runtime::view`
//! (L2 → L1 relocation). This module re-exports them so shipd call sites
//! (`ops_rpc.rs`) continue working without path changes.

pub use ship_pipeline_runtime::view::summaries;
pub use ship_pipeline_runtime::view::run_detail;
pub use ship_pipeline_runtime::view::history;
pub use ship_pipeline_runtime::view::shape;
pub use ship_pipeline_runtime::view::view_state;
pub use ship_pipeline_runtime::view::state;
