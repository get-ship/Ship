// -- Daemon (owner + supervisor + lifecycle) -----------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SupervisorShuttingDown {
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonOwnerBound {
    pub source: String,
    pub bound_at_unix: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonOwnerUnbound {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStartupCompleted {
    pub pid: u32,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonShutdownCompleted {
    pub pid: u32,
    #[serde(default)]
    pub uptime_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonPortBindFailed {
    pub port: u16,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonCrashed {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonPidfileAcquired {
    pub pid: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonPidfileStaleTaken {
    pub stale_pid: u32,
    pub new_pid: u32,
}
