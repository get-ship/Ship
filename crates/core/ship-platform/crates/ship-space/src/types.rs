//! Domain record types for the space service.
//!
//! These are the crate's internal shapes; `wire.rs` maps them to the proto
//! messages. Kept free of any proto/wire types so the service core is
//! transport-agnostic.

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

/// Identity for a space. Per-space settings live on the unified
/// `settings_overlay` (S4) — consumers read those via `SettingsService`.
/// Identity fields are `None` when no space resolved.
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_id: Option<String>,
    pub root: Option<String>,
    pub name: Option<String>,
}
