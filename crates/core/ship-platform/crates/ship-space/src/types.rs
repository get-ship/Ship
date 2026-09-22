//! Domain record types for the space service.
//!
//! These are the crate's internal shapes; `wire.rs` maps them to the proto
//! messages. Kept free of any proto/wire types so the service core is
//! transport-agnostic.

/// One row of the `spaces` Runtime-scoped projection — a space this
/// device's runtime has opened.
///
/// `space_id` is the mint-once nanoid identity (the authoritative home of
/// identity per operator 2026-07-09); `path` is just where the space
/// currently is on this device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceRecord {
    pub space_id: String,
    pub path: String,
    pub name: String,
    pub opened_at: String,
    pub last_seen_at: String,
}

impl From<ship_substrate::db::spaces::SpaceRecord> for SpaceRecord {
    fn from(r: ship_substrate::db::spaces::SpaceRecord) -> Self {
        SpaceRecord {
            space_id: r.space_id,
            path: r.path,
            name: r.name,
            opened_at: r.opened_at,
            last_seen_at: r.last_seen_at,
        }
    }
}

/// Identity for the served space. Identity fields are `None` when no space
/// is bound for the request.
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_id: Option<String>,
    pub root: Option<String>,
    pub name: Option<String>,
}
