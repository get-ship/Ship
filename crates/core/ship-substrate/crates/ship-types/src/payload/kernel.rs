// -- Kernel actor events -----------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KernelActorRestored {
    pub actor_id: String,
    pub event_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KernelActorSnapshot {
    pub actor_id: String,
    pub event_count: u64,
    pub last_event_id: Option<String>,
    pub size_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KernelActorSuspended {
    pub actor_id: String,
    pub reason: String,
}
