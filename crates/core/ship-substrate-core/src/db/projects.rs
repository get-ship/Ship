//! Sync read helpers for the `projects` projection table.
//!
//! The `projects` table is the daemon's machine-wide record of known space
//! paths. It is written exclusively by
//! `ship_space::projects_projection::ProjectsProjection` (moved there in
//! keystone-A Phase A bundle 4, 2026-06-10) which folds `project.registered /
//! seen / archived` events into the SQL table.
//!
//! Two read surfaces live here:
//!
//! * **Global helpers** (`list_active`, `get`, `find_by_path`) use
//!   `get_global_dir()` to open the daemon's runtime.db. Used by the sync
//!   space-id resolver and the `SpaceRegistry` impl — contexts that do NOT
//!   have a ship_dir in scope.
//!
//! * **Path-based helpers** (`list_active_at`, `get_at`, `find_by_path_at`)
//!   accept an explicit `ship_dir: &Path`. Used by `ship_space::registry` so
//!   it can read from the correct (possibly test-temp) runtime.db without
//!   importing sqlx. They are the async-read seam for the dissolved
//!   `space_registry` DataStore table (keystone-A dual-read-model dissolution).
//!
//! All reads use `block_on`/`block_in_place` and are safe to call from async
//! Tokio contexts (multi-threaded runtime). No write path lives here — all
//! mutations go through `ship_space::registry` via events + ProjectsProjection.

use std::path::Path;

use anyhow::Result;
use sqlx::Row as _;
use sqlx::SqliteConnection;

use crate::db::{block_on, open_runtime_db};
use crate::project::get_global_dir;

/// Public-facing registry record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpaceRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub created_at: String,
    pub last_seen_at: String,
}

fn open_projects_conn() -> Result<SqliteConnection> {
    let ship_dir = get_global_dir()?;
    open_runtime_db(&ship_dir)
}

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> SpaceRecord {
    SpaceRecord {
        id: row.get(0),
        path: row.get(1),
        name: row.get(2),
        created_at: row.get(3),
        last_seen_at: row.get(4),
    }
}

/// List all live (non-archived) spaces, sorted by `last_seen_at` desc.
pub fn list_active() -> Result<Vec<SpaceRecord>> {
    let mut conn = open_projects_conn()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, path, name, created_at, last_seen_at \
             FROM projects WHERE archived_at IS NULL ORDER BY last_seen_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(row_to_record).collect())
}

/// Look up a single project row by id.
pub fn get(id: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_projects_conn()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, path, name, created_at, last_seen_at \
             FROM projects WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}

/// Look up a project by filesystem path.
pub fn find_by_path(path: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_projects_conn()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, path, name, created_at, last_seen_at \
             FROM projects WHERE path = ?",
        )
        .bind(path)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}

// ── Path-based helpers (explicit ship_dir) ────────────────────────────────────
//
// These mirror the global helpers above but open the runtime.db under an
// explicit `ship_dir` rather than looking up `get_global_dir()`. Ship-space's
// registry uses these to read from the correct db in both production and tests
// without importing sqlx.

fn open_runtime_conn_at(ship_dir: &Path) -> Result<SqliteConnection> {
    open_runtime_db(ship_dir)
}

/// List all live (non-archived) spaces under `ship_dir`, newest `last_seen_at` first.
pub fn list_active_at(ship_dir: &Path) -> Result<Vec<SpaceRecord>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, path, name, created_at, last_seen_at \
             FROM projects WHERE archived_at IS NULL ORDER BY last_seen_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(row_to_record).collect())
}

/// Look up a single project row by id under `ship_dir`.
pub fn get_at(ship_dir: &Path, id: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, path, name, created_at, last_seen_at \
             FROM projects WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}

/// Look up a project by filesystem path under `ship_dir`.
pub fn find_by_path_at(ship_dir: &Path, path: &str) -> Result<Option<SpaceRecord>> {
    let mut conn = open_runtime_conn_at(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, path, name, created_at, last_seen_at \
             FROM projects WHERE path = ?",
        )
        .bind(path)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(row_to_record))
}
