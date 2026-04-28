//! Handler functions for workspace projection events.
//!
//! Each function applies a single event type to the worktree kv-state
//! registry (`runtime::worktrees`). The runtime DB no longer has a
//! `workspace` projection table — slice 05 of workspace-retirement
//! moved storage to kv. Workspace-session rows still live in the DB;
//! the apply_deleted handler cleans them up alongside the kv entry.
//!
//! All kv writes use `_with_conn` variants so they share the projection
//! transaction's connection — avoids SQLite "database is locked" from a
//! parallel writer.

use anyhow::Result;
use chrono::Utc;
use sqlx::SqliteConnection;

use crate::db::block_on;
use crate::worktrees::{self, WorktreeRecord};

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
    let now = Utc::now().to_rfc3339();
    let id = p.workspace_id.unwrap_or_else(|| entity_id.to_string());
    let existing = worktrees::get_with_conn(conn, entity_id)?;
    let record = WorktreeRecord {
        branch: entity_id.to_string(),
        id,
        path: p.path.or_else(|| existing.as_ref().and_then(|e| e.path.clone())),
        tmux_session_name: existing.as_ref().and_then(|e| e.tmux_session_name.clone()),
        created_at: existing.as_ref().map(|e| e.created_at.clone()).unwrap_or_else(|| now.clone()),
        updated_at: now,
    };
    worktrees::upsert_with_conn(conn, &record)
}

pub(super) fn apply_deleted(entity_id: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query("DELETE FROM workspace_session WHERE workspace_branch = ?")
            .bind(entity_id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    })?;
    worktrees::forget_with_conn(conn, entity_id)
}

pub(super) fn apply_tmux_assigned(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: TmuxAssignedPayload = serde_json::from_str(payload_json)?;
    let Some(mut existing) = worktrees::get_with_conn(conn, entity_id)? else {
        return Ok(());
    };
    existing.tmux_session_name = p.tmux_session_name;
    existing.updated_at = Utc::now().to_rfc3339();
    worktrees::upsert_with_conn(conn, &existing)
}

pub(super) fn apply_started(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: StartedPayload = serde_json::from_str(payload_json)?;
    let existing = worktrees::get_with_conn(conn, entity_id)?;
    let now = Utc::now().to_rfc3339();
    let record = WorktreeRecord {
        branch: entity_id.to_string(),
        id: existing.as_ref().map(|e| e.id.clone()).unwrap_or_else(|| entity_id.to_string()),
        path: Some(p.worktree_path),
        tmux_session_name: Some(p.tmux_session_name),
        created_at: existing.as_ref().map(|e| e.created_at.clone()).unwrap_or_else(|| now.clone()),
        updated_at: now,
    };
    worktrees::upsert_with_conn(conn, &record)
}
