//! Pipeline coherence-sweep events — the reconciler's own audit trail.
//!
//! Moved from ship-substrate/src/events/supervisor.rs (keystone-A, 2026-06-10):
//! the sweep is ship-pipeline-runtime's mechanism (orchestrator/sweep.rs is the
//! only emitter), so its vocabulary lives here. The sweep itself is slated for
//! deletion by the R13/#27 dissolution — this family dies with it.

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

// Registrations: this crate owns the sweep vocabulary (scope, retention,
// carried validator).
use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

register_event!("supervisor.coherence_restored", [EventScope::Runtime], Retention::Durable, Stability::Stable, "CoherenceRestoredPayload", crate::sweep_events::CoherenceRestoredPayload);
register_event!("supervisor.drift_detected", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DriftDetectedPayload", crate::sweep_events::DriftDetectedPayload);
register_event!("supervisor.sweep_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SweepCompletedPayload", crate::sweep_events::SweepCompletedPayload);
register_event!("supervisor.sweep_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SweepStartedPayload", crate::sweep_events::SweepStartedPayload);
