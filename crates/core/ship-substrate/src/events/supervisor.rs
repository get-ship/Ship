//! Supervisor lifecycle events. The reconciler's own audit trail.

use serde::{Deserialize, Serialize};
pub mod event_types {
    pub const SUPERVISOR_SWEEP_STARTED: &str = "supervisor.sweep_started";
    pub const SUPERVISOR_SWEEP_COMPLETED: &str = "supervisor.sweep_completed";
    pub const SUPERVISOR_DRIFT_DETECTED: &str = "supervisor.drift_detected";
    pub const SUPERVISOR_COHERENCE_RESTORED: &str = "supervisor.coherence_restored";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepStartedPayload {
    pub jobs_in_scope: usize,
    pub processes_known: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepCompletedPayload {
    pub duration_ms: u64,
    pub drift_count: usize,
    pub actions_taken: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectedPayload {
    pub slug: String,
    #[serde(alias = "job_id")]
    pub pipeline_run_id: String,
    pub expected_status: String,
    /// "no_process" | "stray_process"
    pub actual_state: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceRestoredPayload {
    pub slug: String,
    /// "marked_failed" | "process_killed"
    pub action: String,
    pub detail: String,
}
