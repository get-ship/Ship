//! Sync read helpers for the `projects` projection table.
//!
//! The `projects` table is the daemon's machine-wide record of known space
//! paths. It is written by `ship-space` (via the DataStore port + KernelLog)
//! and materialised from `project.registered / seen / archived` events by
//! `ship_space::projects_projection::ProjectsProjection` (moved there in
//! keystone-A Phase A bundle 4, 2026-06-10).
//!
//! All reads here are sync (sqlx + block_on) so they can be used in sync
//! contexts (op closures, the space-id resolver, the SpaceRegistry impl).
//! The L0 space-id resolver and SpaceRegistry depend on these helpers and
//! must keep working unchanged — they are intentionally kept at L0.
//!
//! No write path lives here. All mutations go through `ship_space::registry`.

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
