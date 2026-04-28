//! Handler functions for workspace projection events.
//!
//! Each function applies a single event type to the workspace table.
//! The table holds only identity + process-attachment columns (branch,
//! id, path, tmux_session_name, created_at, updated_at).

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::db::block_on;

// ── Payloads ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub(super) struct CreatedPayload {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct TmuxAssignedPayload {
    pub tmux_session_name: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct StartedPayload {
    pub worktree_path: String,
    pub tmux_session_name: String,
}

// ── Handlers ────────────────────────────────────────────────────────────────

pub(super) fn apply_created(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CreatedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    let workspace_id = p.workspace_id.as_deref().unwrap_or(entity_id);
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace \
             (branch, id, path, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(branch) DO UPDATE SET \
               id         = excluded.id, \
               path       = excluded.path, \
               updated_at = excluded.updated_at",
        )
        .bind(entity_id)
        .bind(workspace_id)
        .bind(&p.path)
        .bind(&now)
        .bind(&now)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_deleted(entity_id: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query(
            "DELETE FROM workspace_session WHERE workspace_branch = ?",
        )
        .bind(entity_id)
        .execute(&mut *conn)
        .await?;
        sqlx::query("DELETE FROM workspace WHERE branch = ?")
            .bind(entity_id)
            .execute(conn)
            .await?;
        Ok(())
    })
}

pub(super) fn apply_tmux_assigned(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: TmuxAssignedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET tmux_session_name = ?, updated_at = ? WHERE branch = ?",
        )
        .bind(&p.tmux_session_name)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_started(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: StartedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET path = ?, tmux_session_name = ?, \
             updated_at = ? WHERE branch = ?",
        )
        .bind(&p.worktree_path)
        .bind(&p.tmux_session_name)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}
