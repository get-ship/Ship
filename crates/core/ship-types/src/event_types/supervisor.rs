pub const SHUTTING_DOWN: &str = "supervisor.shutting_down";
pub const SWEEP_STARTED: &str = "supervisor.sweep_started";
pub const SWEEP_COMPLETED: &str = "supervisor.sweep_completed";
pub const DRIFT_DETECTED: &str = "supervisor.drift_detected";
pub const COHERENCE_RESTORED: &str = "supervisor.coherence_restored";

pub const ALL_SUPERVISOR: &[&str] = &[
    SHUTTING_DOWN,
    SWEEP_STARTED,
    SWEEP_COMPLETED,
    DRIFT_DETECTED,
    COHERENCE_RESTORED,
];
