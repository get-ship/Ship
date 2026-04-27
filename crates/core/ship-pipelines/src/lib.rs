//! Pipeline pure-logic primitives — no fs / process / network / tokio.
//!
//! Per modular-runtime spec 01, this crate is intended to be portable
//! to the Ship cloud worker (Fly Sprite, full Rust) and the eventual
//! Ship browser embed without daemon imports. Acceptance: `cargo
//! check --target aarch64-linux-android -p ship-pipelines` succeeds.

pub mod gate;
pub mod types;
pub mod validate;
pub mod variables;

pub use gate::{GateOutcome, GateRule, GateSpec};
pub use types::{
    FanoutJudge, FanoutLane, FanoutSpec, OutputScope, Pipeline, PipelinePhase, PipelineStep,
    StepOutput, StepPromote, StepType,
};
pub use validate::{
    FAILED_MARKER_NAME, parse_lane_scoped_ref, validate_output_name, validate_pipeline,
    validate_pipeline_io,
};
pub use variables::{interpolate, interpolate_step};
