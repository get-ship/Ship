//! Daemon lifecycle events. Observability for the daemon itself.
//!
//! Crashes, port-bind failures, pidfile takeovers — every state change in
//! the daemon's life lands here. Slice 4 (`ship daemon status` upgrades)
//! consumes these via a `DaemonRecord` projection; this slice only emits.

use serde::{Deserialize, Serialize};
pub mod event_types {
    pub use ship_types::event_types::{
        DAEMON_CRASH_DETECTED, DAEMON_CRASHED, DAEMON_PIDFILE_ACQUIRED,
        DAEMON_PIDFILE_STALE_TAKEN, DAEMON_PORT_BIND_FAILED, DAEMON_SHUTDOWN_COMPLETED,
        DAEMON_STARTUP_COMPLETED,
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupCompletedPayload {
    pub pid: u32,
    pub port: u16,
    pub version: String,
    pub commit_sha: String,
    pub event_count_at_startup: i64,
    pub latest_event_id: Option<String>,
    pub init_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownCompletedPayload {
    pub pid: u32,
    pub uptime_secs: u64,
    pub agents_drained: usize,
    /// "sigterm" | "sigint" | "operator" | "version_mismatch" | "child_exited_cleanly"
    pub trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashedPayload {
    pub pid: u32,
    pub uptime_secs: u64,
    pub exit_code: Option<i32>,
    /// "SIGSEGV" etc; None for clean non-zero exits.
    pub signal: Option<String>,
    pub last_known_event_id: Option<String>,
    pub restart_will_be_attempted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortBindFailedPayload {
    pub port: u16,
    pub already_bound_by_pid: Option<u32>,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidfileAcquiredPayload {
    pub pid: u32,
    pub pidfile_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidfileStaleTakenPayload {
    pub new_pid: u32,
    pub prior_pid: u32,
    pub prior_started_at: String,
    pub prior_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDetectedPayload {
    /// The `daemon.startup_completed` event id for the run that never
    /// emitted a matching clean shutdown.
    pub prior_started_event_id: String,
    /// Filenames of `crash-*.json` reports found in the crashes dir
    /// (shipctl's kernel-less crash-report channel), most recent first.
    pub correlated_crash_reports: Vec<String>,
    pub crash_report_count: usize,
}
