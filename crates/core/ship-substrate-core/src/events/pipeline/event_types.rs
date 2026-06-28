//! Event type constants for the `pipeline.` namespace.
//!
//! These string keys are the discriminator used in the event log. Callers
//! compare against them when routing or filtering — the strings themselves
//! are part of the wire contract.

/// Event type constants for the `pipeline.` namespace.
pub const PIPELINE_RUN_STARTED: &str = "pipeline.run.started";
pub const PIPELINE_RUN_DISPATCHED: &str = "pipeline.run.dispatched";
pub const PIPELINE_GATE_REQUESTED: &str = "pipeline.gate.requested";
pub const PIPELINE_GATE_PASSED: &str = "pipeline.gate.passed";
pub const PIPELINE_GATE_FAILED: &str = "pipeline.gate.failed";
pub const PIPELINE_RUN_BLOCKED: &str = "pipeline.run.blocked";
pub const PIPELINE_RUN_COMPLETED: &str = "pipeline.run.completed";
pub const PIPELINE_RUN_MERGED: &str = "pipeline.run.merged";
pub const PIPELINE_RUN_FAILED: &str = "pipeline.run.failed";
pub const PIPELINE_STEP_SPAWN_FAILED: &str = "pipeline.step.spawn_failed";
pub const PIPELINE_RUN_CANCELLED: &str = "pipeline.run.cancelled";
pub const PIPELINE_RUN_UPDATED: &str = "pipeline.run.updated";
pub const PIPELINE_STEP_STARTED: &str = "pipeline.step.started";
pub const PIPELINE_STEP_COMPLETED: &str = "pipeline.step.completed";
pub const PIPELINE_STEP_FAILED: &str = "pipeline.step.failed";
pub const PIPELINE_STEP_OUTPUT: &str = "pipeline.step.output";
pub const PIPELINE_STEP_RETRY: &str = "pipeline.step.retry";
pub const PIPELINE_STEP_DONE_DECLARED: &str = "pipeline.step.done_declared";
pub const PIPELINE_STEP_OBSERVED: &str = "pipeline.step.observed";
