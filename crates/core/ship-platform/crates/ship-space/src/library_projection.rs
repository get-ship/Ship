//! Event-sourced `LibraryProjection` — folds `stack.installed` (insert) and
//! `stack.unregistered` (delete) into the Runtime-scoped `library` table.
//!
//! Membership is a registry-installed stack; rows are device-local and
//! keyed by `(path, version)` — the registry-issued identity pair — NOT by
//! the envelope `entity_id` (which is the runtime's synthetic install
//! identity, not the stack's). The projection is the sole writer of the
//! `library` SQL table.

use anyhow::Result;
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{STACK_INSTALLED, STACK_UNREGISTERED};
use ship_types::payload::{StackInstalled, StackUnregistered};

/// Event-sourced materialiser for the `library` table.
pub struct LibraryProjection;

impl Projection for LibraryProjection {
    fn name(&self) -> &str {
        "library"
    }

    fn event_types(&self) -> &[&str] {
        &[STACK_INSTALLED, STACK_UNREGISTERED]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            STACK_INSTALLED => apply_installed(event, conn),
            STACK_UNREGISTERED => apply_unregistered(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async { sqlx::query("DELETE FROM library").execute(conn).await })?;
        Ok(())
    }
}

fn apply_installed(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: StackInstalled = serde_json::from_str(&event.payload_json)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO library (path, version, root_path, content_hash, installed_at, enabled) \
             VALUES (?1, ?2, ?3, ?4, ?5, 1) \
             ON CONFLICT(path, version) DO UPDATE SET \
               root_path    = excluded.root_path, \
               content_hash = excluded.content_hash, \
               installed_at = excluded.installed_at, \
               enabled      = 1",
        )
        .bind(&payload.path)
        .bind(&payload.version)
        .bind(&payload.root_path)
        .bind(&payload.content_hash)
        .bind(&payload.installed_at)
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn apply_unregistered(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: StackUnregistered = serde_json::from_str(&event.payload_json)?;
    block_on(async {
        sqlx::query("DELETE FROM library WHERE path = ?1 AND version = ?2")
            .bind(&payload.path)
            .bind(&payload.version)
            .execute(conn)
            .await
    })?;
    Ok(())
}
