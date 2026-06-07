//! Sync read helpers for the `projects` projection table + the event-sourced
//! `ProjectsProjection` that materialises it.
//!
//! The `projects` table is the daemon's machine-wide record of known space
//! paths. It is written by `ship-space` (via the DataStore port + KernelLog)
//! and materialised here from `project.registered / seen / archived` events.
//! All reads are sync (sqlx + block_on) so they can be used in sync contexts
//! (op closures, the space-id resolver, the SpaceRegistry impl).
//!
//! No write path lives here. All mutations go through `ship_space::registry`.

use anyhow::Result;
use sqlx::Row as _;
use sqlx::SqliteConnection;

use crate::db::{block_on, open_runtime_db};
use crate::project::get_global_dir;
use crate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{PROJECT_ARCHIVED, PROJECT_REGISTERED, PROJECT_SEEN};
use ship_types::payload::{ProjectArchived, ProjectRegistered, ProjectSeen};

/// Public-facing registry record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpaceRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub created_at: String,
    pub last_seen_at: String,
}

/// Event-sourced materialiser for the `projects` table.
///
/// Folds `project.registered / seen / archived` events (Runtime-scoped, live
/// in `runtime.db`) into the `projects` sqlx table. The ship-space crate is
/// the sole writer of those events; this projection is the sole writer of the
/// read-side `projects` table rows.
pub struct ProjectsProjection;

impl Projection for ProjectsProjection {
    fn name(&self) -> &str {
        "projects"
    }

    fn event_types(&self) -> &[&str] {
        &[PROJECT_REGISTERED, PROJECT_SEEN, PROJECT_ARCHIVED]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            PROJECT_REGISTERED => apply_registered(event, conn),
            PROJECT_SEEN => apply_seen(event, conn),
            PROJECT_ARCHIVED => apply_archived(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM projects").execute(conn).await
        })?;
        Ok(())
    }
}

fn apply_registered(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: ProjectRegistered = serde_json::from_str(&event.payload_json)?;
    let space_id = event.entity_id.clone();
    block_on(async {
        sqlx::query(
            "INSERT INTO projects \
             (id, path, name, label, created_at, last_seen_at, archived_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, NULL, datetime('now')) \
             ON CONFLICT(id) DO UPDATE SET \
               path         = excluded.path, \
               name         = excluded.name, \
               label        = excluded.label, \
               last_seen_at = excluded.last_seen_at, \
               archived_at  = NULL, \
               updated_at   = datetime('now')",
        )
        .bind(&space_id)
        .bind(&payload.path)
        .bind(&payload.name)
        .bind(&payload.label)
        .bind(&payload.created_at)
        .bind(&payload.created_at)
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn apply_seen(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: ProjectSeen = serde_json::from_str(&event.payload_json)?;
    block_on(async {
        sqlx::query(
            "UPDATE projects SET last_seen_at = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&payload.seen_at)
        .bind(&event.entity_id)
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn apply_archived(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: ProjectArchived = serde_json::from_str(&event.payload_json)?;
    block_on(async {
        sqlx::query(
            "UPDATE projects SET archived_at = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&payload.archived_at)
        .bind(&event.entity_id)
        .execute(conn)
        .await
    })?;
    Ok(())
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
