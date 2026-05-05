//! `projects` projection — the daemon's registry of project paths.
//!
//! Materialized from `project.registered` / `project.seen` / `project.archived`
//! events. CLI `ship status` and Studio's project switcher read here.

use anyhow::Result;
use sqlx::{Row, SqliteConnection};

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{PROJECT_ARCHIVED, PROJECT_REGISTERED, PROJECT_SEEN};
use ship_types::payload::{ProjectArchived, ProjectRegistered, ProjectSeen};

/// Read model row for the `projects` table.
#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: String,
    pub path: String,
    pub name: String,
    pub label: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
    pub archived_at: Option<String>,
}

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
    let project_id = event.entity_id.clone();
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
        .bind(&project_id)
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

/// List all live (non-archived) projects, sorted by last_seen_at desc.
pub fn list_active() -> Result<Vec<ProjectRow>> {
    ship_substrate::db::ensure_db()?;
    let mut conn = ship_substrate::db::open_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, path, name, label, created_at, last_seen_at, archived_at \
             FROM projects WHERE archived_at IS NULL ORDER BY last_seen_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.into_iter().map(row_to_project).collect())
}

/// Lookup by id.
pub fn get(id: &str) -> Result<Option<ProjectRow>> {
    ship_substrate::db::ensure_db()?;
    let mut conn = ship_substrate::db::open_db()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, path, name, label, created_at, last_seen_at, archived_at \
             FROM projects WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(row_to_project))
}

/// Lookup by path.
pub fn find_by_path(path: &str) -> Result<Option<ProjectRow>> {
    ship_substrate::db::ensure_db()?;
    let mut conn = ship_substrate::db::open_db()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, path, name, label, created_at, last_seen_at, archived_at \
             FROM projects WHERE path = ?",
        )
        .bind(path)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(row_to_project))
}

fn row_to_project(row: sqlx::sqlite::SqliteRow) -> ProjectRow {
    ProjectRow {
        id: row.get(0),
        path: row.get(1),
        name: row.get(2),
        label: row.try_get(3).ok(),
        created_at: row.get(4),
        last_seen_at: row.get(5),
        archived_at: row.try_get(6).ok(),
    }
}
