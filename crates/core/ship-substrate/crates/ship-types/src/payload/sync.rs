// -- Sync events (added in Spec 6) ---------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPushFailed {
    pub error: String,
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStarted {
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStopped {
    pub target: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPushCompleted {
    pub events_pushed: u64,
    pub duration_ms: u64,
}
