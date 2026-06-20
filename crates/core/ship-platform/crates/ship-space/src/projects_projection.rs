//! Event-sourced `ProjectsProjection` — folds project.* events into the
//! `projects` SQL table.
//!
//! Moved from `ship_substrate::db::projects` (keystone-A Phase A bundle 4,
//! 2026-06-10): the SPACE domain owns its projection at L1.
//!
//! The `projects` table is the daemon's machine-wide record of known space
//! paths. This projection folds `project.registered / seen` events
//! (Runtime-scoped, live in `runtime.db`) into the `projects` sqlx table.
//! The ship-space crate is the sole writer of those events; this projection
//! is the sole writer of the read-side `projects` table rows.
//!
//! The SYNC SQL READ HELPERS (list_active / get / find_by_path) STAY in
//! `ship_substrate::db::projects` — L0 reads the table by SQL (the space-id
//! resolver and SpaceRegistry impl depend on them; they must keep working
//! unchanged). This module only owns the fold/write side.

use anyhow::Result;
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{PROJECT_REGISTERED, PROJECT_SEEN};
use ship_types::payload::{ProjectRegistered, ProjectSeen};

/// Event-sourced materialiser for the `projects` table.
///
/// Folds `project.registered / seen` events (Runtime-scoped, live in
/// `runtime.db`) into the `projects` sqlx table. The ship-space crate is the
/// sole writer of those events; this projection is the sole writer of the
/// read-side `projects` table rows.
pub struct ProjectsProjection;

impl Projection for ProjectsProjection {
    fn name(&self) -> &str {
        "projects"
    }

    fn event_types(&self) -> &[&str] {
        &[PROJECT_REGISTERED, PROJECT_SEEN]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            PROJECT_REGISTERED => apply_registered(event, conn),
            PROJECT_SEEN => apply_seen(event, conn),
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
             (id, path, name, created_at, last_seen_at, archived_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, NULL, datetime('now')) \
             ON CONFLICT(id) DO UPDATE SET \
               path         = excluded.path, \
               name         = excluded.name, \
               last_seen_at = excluded.last_seen_at, \
               archived_at  = NULL, \
               updated_at   = datetime('now')",
        )
        .bind(&space_id)
        .bind(&payload.path)
        .bind(&payload.name)
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

