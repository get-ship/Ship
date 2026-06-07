//! Domain record types for the space service.
//!
//! These are the crate's internal shapes; `wire.rs` maps them to the proto
//! messages. Kept free of any proto/wire types so the service core is
//! transport-agnostic.

use serde_json::Value;

/// One registered space (a `.ship/` directory the daemon has been pointed at).
/// Backed by a `projects` projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub created_at: String,
    pub last_seen_at: String,
}

/// Identity + effective settings for a space. The settings are layered
/// **global ⟵ space**: the daemon-global base overlaid per-key by the space's
/// event-sourced overrides. Identity fields are `None` when no space resolved.
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_id: Option<String>,
    pub root: Option<String>,
    pub name: Option<String>,
    pub settings: Value,
}

/// The echoed shape of a per-space setting write: the key plus the re-serialized
/// (canonical) JSON value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingValue {
    pub key: String,
    pub value_json: String,
}
