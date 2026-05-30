pub const RUN_STARTED: &str = "pipeline.run.started";
pub const RUN_COMPLETED: &str = "pipeline.run.completed";
pub const RUN_FAILED: &str = "pipeline.run.failed";
pub const RUN_DISPATCHED: &str = "pipeline.run.dispatched";
pub const RUN_UPDATED: &str = "pipeline.run.updated";
pub const CHANGED: &str = "pipeline.changed";
pub const STEP_COMPLETED: &str = "pipeline.step.completed";
pub const STEP_FAILED: &str = "pipeline.step.failed";
pub const RUN_CANCELLED: &str = "pipeline.run.cancelled";
pub const RUN_MERGED: &str = "pipeline.run.merged";
pub const STEP_SPAWN_FAILED: &str = "pipeline.step.spawn_failed";
pub const STEP_RETRY: &str = "pipeline.step.retry";

pub const ALL_PIPELINE: &[&str] = &[
    RUN_STARTED,
    RUN_COMPLETED,
    RUN_FAILED,
    RUN_DISPATCHED,
    RUN_UPDATED,
    CHANGED,
    STEP_COMPLETED,
    STEP_FAILED,
    RUN_CANCELLED,
    RUN_MERGED,
    STEP_SPAWN_FAILED,
    STEP_RETRY,
];
