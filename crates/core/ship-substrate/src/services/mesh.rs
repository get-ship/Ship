//! Agent mesh service — registration, directed messaging, broadcast, discovery.
//!
//! The mesh is a thin validator and address resolver. It creates `EventEnvelope`s
//! with `target_actor_id` set, pushes them to an outbox channel, and the caller
//! drains the outbox through the kernel.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Shared, read-optimized view of the mesh registry.
pub type SharedMeshRegistry = Arc<RwLock<HashMap<String, MeshEntry>>>;

/// Status of a registered agent in the mesh.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active,
    Busy,
    Idle,
}

/// A registered agent in the mesh registry.
#[derive(Clone, Debug)]
pub struct MeshEntry {
    pub agent_id: String,
    pub label: String,
    pub capabilities: Vec<String>,
    pub registered_at: DateTime<Utc>,
    pub status: AgentStatus,
}

// MeshService DELETED (residue cull, 2026-06-10): it was the in-process
// handler for the killed mesh.* op surface (shipd never instantiated it —
// "Real inter-agent messaging is now native via the kernel's directed-event
// delivery"). The registry TYPES above (SharedMeshRegistry / MeshEntry /
// AgentStatus) are the live mailbox-registry mechanism and stay.
//
// sender_from / status_str / parse_status DELETED: post-MeshService residue.
// These helpers were internal to MeshService methods that no longer exist.
// No other code referenced them after the mesh.* op surface was retired.
