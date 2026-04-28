//! Workspace state reads — kv-backed worktree registry queries.
//!
//! The runtime DB no longer projects workspaces to a table; storage moved
//! to `kv_state` namespace `worktree` (slice 05 of workspace-retirement).
//! These functions adapt the registry shape to the legacy
//! `WorkspaceDbRow` / `WorkspaceDbListRow` tuples consumed by `workspace::crud`.

use anyhow::Result;

use super::types::{WorkspaceDbListRow, WorkspaceDbRow};
use super::{block_on, open_db};
use crate::worktrees;

/// Retrieve the workspace record by id (or branch as fallback), or None.
pub fn get_workspace_by_id_db(id: &str) -> Result<Option<(String, WorkspaceDbRow)>> {
    let Some(rec) = worktrees::get_by_id_or_branch(id)? else {
        return Ok(None);
    };
    Ok(Some((
        rec.branch,
        (rec.id, rec.path, Some(rec.created_at)),
    )))
}

/// Retrieve the workspace record for the given branch, or None if none exists.
pub fn get_workspace_db(branch: &str) -> Result<Option<WorkspaceDbRow>> {
    let Some(rec) = worktrees::get(branch)? else {
        return Ok(None);
    };
    Ok(Some((rec.id, rec.path, Some(rec.created_at))))
}

pub fn list_workspaces_db() -> Result<Vec<WorkspaceDbListRow>> {
    let rows = worktrees::list()?;
    Ok(rows
        .into_iter()
        .map(|r| (r.branch, r.id, r.path, Some(r.created_at)))
        .collect())
}

/// Retrieve the tmux_session_name for a workspace by id or branch.
pub fn get_workspace_tmux_session_db(id: &str) -> Result<Option<String>> {
    Ok(worktrees::get_by_id_or_branch(id)?.and_then(|r| r.tmux_session_name))
}

/// Check if a workspace has an active session.
pub fn has_active_session_db(workspace_id: &str) -> Result<bool> {
    let mut conn = open_db()?;
    let count: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM workspace_session \
             WHERE workspace_id = ? AND status = 'active' LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_one(&mut conn)
        .await
    })?;
    Ok(count > 0)
}
