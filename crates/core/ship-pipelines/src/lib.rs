//! Pipeline pure-logic primitives — no fs / process / network / tokio.
//!
//! Per modular-runtime spec 01, this crate is intended to be portable
//! to the Ship cloud worker (Fly Sprite, full Rust) and the eventual
//! Ship browser embed without daemon imports. Acceptance: `cargo
//! check --target aarch64-linux-android -p ship-pipelines` succeeds.

pub mod gate;
pub mod types;

pub use gate::{GateOutcome, GateRule, GateSpec};
pub use types::{
    FanoutJudge, FanoutLane, FanoutSpec, OutputScope, Pipeline, PipelinePhase, PipelineStep,
    StepOutput, StepPromote, StepType,
};
