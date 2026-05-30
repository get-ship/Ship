pub const STARTED: &str = "session.started";
pub const PROGRESS: &str = "session.progress";
pub const ENDED: &str = "session.ended";
pub const DRAIN_STARTED: &str = "session.drain_started";
pub const DRAIN_COMPLETED: &str = "session.drain_completed";
pub const DRAIN_ABORTED: &str = "session.drain_aborted";
pub const TOOL_COUNT_INCREMENTED: &str = "session.tool_count_incremented";
pub const ATTACHED: &str = "session.attached";
pub const HEARTBEAT: &str = "session.heartbeat";
pub const STATUS_CHANGED: &str = "session.status_changed";
pub const TURN_FAILED: &str = "session.turn_failed";

pub const ALL_SESSION: &[&str] = &[
    STARTED,
    PROGRESS,
    ENDED,
    DRAIN_STARTED,
    DRAIN_COMPLETED,
    DRAIN_ABORTED,
    TOOL_COUNT_INCREMENTED,
    ATTACHED,
    HEARTBEAT,
    STATUS_CHANGED,
    TURN_FAILED,
];
