//! Handler functions for workspace projection events.
//!
//! All handlers are no-ops as of phase 2 of workspace removal. The
//! workspace primitive is being retired; the events still flow through
//! the log (events table) for audit / replay, but no projection state
//! is materialised. Apply `apply_deleted` cleans the legacy
//! `workspace_session` rows for the deleted branch since session
//! analytics still key on `workspace_branch` (renamed in phase 6).

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::db::block_on;

pub(super) fn apply_created(
    _entity_id: &str,
    _payload_json: &str,
    _conn: &mut SqliteConnection,
) -> Result<()> {
    Ok(())
}

pub(super) fn apply_deleted(entity_id: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query("DELETE FROM workspace_session WHERE workspace_branch = ?")
            .bind(entity_id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    })
}

pub(super) fn apply_tmux_assigned(
    _entity_id: &str,
    _payload_json: &str,
    _conn: &mut SqliteConnection,
) -> Result<()> {
    Ok(())
}

pub(super) fn apply_started(
    _entity_id: &str,
    _payload_json: &str,
    _conn: &mut SqliteConnection,
) -> Result<()> {
    Ok(())
}
