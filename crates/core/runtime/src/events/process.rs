//! Process-lifecycle events. Distinct from `job.*` because a single
//! job can spawn many processes (per step), and a process is an OS
//! resource the supervisor must observe directly.
//!
//! Slice 3 introduces only `process.killed`. Slice 4 adds spawned,
//! exited, signal_sent, orphaned (see design doc).

use serde::{Deserialize, Serialize};
use specta::Type;

pub mod event_types {
    pub const PROCESS_KILLED: &str = "process.killed";
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcessKilledPayload {
    /// Slug of the job whose process was killed.
    pub slug: String,
    /// PID at time of kill, if known. None when the registry was
    /// empty and the kill went via tmux session destruction.
    pub pid: Option<u32>,
    /// Signal sent (e.g. "SIGTERM", "SIGKILL", "tmux_kill_session").
    pub signal: String,
    /// Why the process was killed: "cancellation", "shutdown",
    /// "timeout", "force", "supervisor_sweep".
    pub reason: String,
    /// Who initiated: "operator", "supervisor", "policy:<name>".
    pub actor: String,
}
