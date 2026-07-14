//! Sync read helpers for the `spaces` projection table.
//!
//! The `spaces` table is the daemon's device-local record of opened spaces.
//! It is written exclusively by `ship_space::spaces_projection::SpacesProjection`,
//! which folds `space.opened` and `space.seen` events into the SQL rows.
//!
//! Identity lives in `spaces.space_id` and is minted ONCE by
//! `ship_space::identity::ensure_identity_for_root` — never derived from a
//! path. Read helpers here therefore key by `space_id` (the identity) and
//! `path` (location only); no canonical-path resolution happens at the L0
//! read seam (the resolution is owned by the identity accessor at L1).
//!
//! Two read surfaces live here:
//!
//! * **Global helpers** (`list_open`, `get`, `find_by_path`) use
//!   `get_global_dir()` to open the daemon's `runtime.db`. Used by
//!   `ship_substrate::project::resolve_active_space` (a path-keyed lookup
//!   for callers that have no ship_dir in scope) and other sync read sites.
//!
//! * **Path-based helpers** (`list_open_at`, `get_at`, `find_by_path_at`)
//!   accept an explicit `ship_dir: &Path`. Used by `ship_space::spaces` and
//!   tests that need to read from a specific runtime.db without importing
//!   sqlx in production.
//!
//! All reads use `block_on`/`block_in_place` and are safe to call from async
//! Tokio contexts (multi-threaded runtime). No write path lives here — all
//! mutations go through `ship_space::spaces` (or the Add RPC) via events
//! + the `SpacesProjection`.

use std::path::Path;

use anyhow::Result;
use sqlx::Row as _;
use sqlx::SqliteConnection;

use crate::db::{block_on, open_runtime_db};
use crate::project::get_global_dir;

/// Public-facing record for a space the runtime has opened on this device.
///
/// Mirrors the columns of the `spaces` projection table. `path` is the
/// location; `space_id` is the mint-once nanoid identity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpaceRecord {
    pub space_id: String,
    pub path: String,
    pub name: String,
    pub opened_at: String,
    pub last_seen_at: String,
}

fn open_spaces_conn() -> Result<SqliteConnection> {
    let ship_dir = get_global_dir()?;
    open_runtime_db(&ship_dir)
}

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> SpaceRecord {
    SpaceRecord {
        space_id: row.get(0),
        path: row.get(1),
        name: row.get(2),
        opened_at: row.get(3),
        last_seen_at: row.get(4),
    }
}

/// List every space currently recorded on this device, newest `last_seen_at` first.
pub fn list_open() -> Result<Vec<SpaceRecord>> {
    let mut conn = open_spaces_conn()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT space_id, path, name, opened_at, last_seen_at \
             FROM spaces ORDER BY last_seen_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(row_to_record).collect())
}

/// Look up a space by its identity (the mint-once `space_id`).
///
/// Identity is the canonical key — callers reach for this when they have a
/// `space_id` and want to validate it. `Path` lookups live in `find_by_path`.
pub fn get(space_id: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_spaces_conn()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT space_id, path, name, opened_at, last_seen_at \
             FROM spaces WHERE space_id = ?",
        )
        .bind(space_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}

/// Look up a space by its filesystem path. Used by callers that have a path
/// in hand but no identity yet (e.g. `resolve_active_space` and the run-time
/// boot path's pre-boot reverse-lookup). Returns `None` for paths the runtime
/// has never opened — opening one is a deliberate act and goes through the
/// `ship_space::identity::ensure_identity_for_root` accessor.
pub fn find_by_path(path: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_spaces_conn()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT space_id, path, name, opened_at, last_seen_at \
             FROM spaces WHERE path = ?",
        )
        .bind(path)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}

// ── Path-based helpers (explicit ship_dir) ─────────────────────────────────
//
// These mirror the global helpers above but open the runtime.db under an
// explicit `ship_dir` rather than looking up `get_global_dir()`. Ship-space's
// spaces module uses these to read from the correct db in both production
// and tests without importing sqlx.

fn open_runtime_conn_at(ship_dir: &Path) -> Result<SqliteConnection> {
    open_runtime_db(ship_dir)
}

/// List all spaces under `ship_dir`, newest `last_seen_at` first.
pub fn list_open_at(ship_dir: &Path) -> Result<Vec<SpaceRecord>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT space_id, path, name, opened_at, last_seen_at \
             FROM spaces ORDER BY last_seen_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(row_to_record).collect())
}

/// Look up a space by its identity under `ship_dir`.
pub fn get_at(ship_dir: &Path, space_id: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT space_id, path, name, opened_at, last_seen_at \
             FROM spaces WHERE space_id = ?",
        )
        .bind(space_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}

/// Look up a space by filesystem path under `ship_dir`.
pub fn find_by_path_at(ship_dir: &Path, path: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT space_id, path, name, opened_at, last_seen_at \
             FROM spaces WHERE path = ?",
        )
        .bind(path)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}

/// A stack registered as a member of a space (device-local row).
#[derive(Debug, Clone)]
pub struct StackRow {
    pub space_id: String,
    pub root_path: String,
    pub registered_at: String,
}

/// List the stacks registered as members of `space_id` (global runtime db),
/// in registration order (the composition order).
pub fn list_stacks(space_id: &str) -> Result<Vec<StackRow>> {
    let ship_dir = get_global_dir()?;
    list_stacks_at(&ship_dir, space_id)
}

/// List the stacks registered as members of `space_id` under `ship_dir`,
/// in registration order (the composition order).
pub fn list_stacks_at(ship_dir: &Path, space_id: &str) -> Result<Vec<StackRow>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT space_id, root_path, registered_at \
             FROM space_stacks WHERE space_id = ? ORDER BY registered_at, root_path",
        )
        .bind(space_id)
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows
        .iter()
        .map(|r| {
            use sqlx::Row as _;
            StackRow {
                space_id: r.get("space_id"),
                root_path: r.get("root_path"),
                registered_at: r.get("registered_at"),
            }
        })
        .collect())
}

/// A library row — a registry-installed stack sitting on this device under
/// the managed library dir. Folded from `stack.installed` /
/// `stack.unregistered` events by `ship_space::library_projection`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstalledStack {
    pub path: String,
    pub version: String,
    pub root_path: String,
    pub content_hash: String,
    pub installed_at: String,
    pub enabled: bool,
}

impl InstalledStack {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Self {
        use sqlx::Row as _;
        let enabled_int: i64 = row.get("enabled");
        Self {
            path: row.get("path"),
            version: row.get("version"),
            root_path: row.get("root_path"),
            content_hash: row.get("content_hash"),
            installed_at: row.get("installed_at"),
            enabled: enabled_int != 0,
        }
    }
}

/// List every library row in the global runtime db (no filter — these are
/// device-local, every space sees the same set; the `space_id` filter on the
/// wire is dropped at L1 to match the runtime-scoped projection).
pub fn list_installed() -> Result<Vec<InstalledStack>> {
    let ship_dir = get_global_dir()?;
    list_installed_at(&ship_dir)
}

/// List every library row under `ship_dir`, newest install first.
pub fn list_installed_at(ship_dir: &Path) -> Result<Vec<InstalledStack>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT path, version, root_path, content_hash, installed_at, enabled \
             FROM library ORDER BY installed_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(InstalledStack::from_row).collect())
}

/// Look up a library row by `(path, version)` under `ship_dir`.
pub fn find_installed_at(
    ship_dir: &Path,
    path: &str,
    version: &str,
) -> Result<Option<InstalledStack>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT path, version, root_path, content_hash, installed_at, enabled \
             FROM library WHERE path = ? AND version = ?",
        )
        .bind(path)
        .bind(version)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(InstalledStack::from_row))
}

/// Look up a library row by `root_path` (the absolute stack root on disk).
/// Used by the wire layer's `unregister_stack` to resolve the row from the
/// proto's `root_path` field.
pub fn find_installed_at_root(
    ship_dir: &Path,
    root_path: &str,
) -> Result<Option<InstalledStack>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT path, version, root_path, content_hash, installed_at, enabled \
             FROM library WHERE root_path = ?",
        )
        .bind(root_path)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(InstalledStack::from_row))
}
