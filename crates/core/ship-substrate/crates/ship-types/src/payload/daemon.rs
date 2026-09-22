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
pub struct DaemonPortBindFailed {
    pub port: u16,
    pub error: String,
}
