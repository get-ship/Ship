//! Workspace state reads — inert as of phase 2 of workspace removal.
//!
//! The workspace primitive is being retired; there is no projection
//! storage to read. These functions return None / empty for compatibility
//! with the legacy `runtime::workspace::*` API surface, which deletes
//! in phase 5.

use anyhow::Result;

use super::types::{WorkspaceDbListRow, WorkspaceDbRow};
use super::{block_on, open_db};

pub fn get_workspace_by_id_db(_id: &str) -> Result<Option<(String, WorkspaceDbRow)>> {
    Ok(None)
}

pub fn get_workspace_db(_branch: &str) -> Result<Option<WorkspaceDbRow>> {
    Ok(None)
}

pub fn list_workspaces_db() -> Result<Vec<WorkspaceDbListRow>> {
    Ok(Vec::new())
}

pub fn get_workspace_tmux_session_db(_id: &str) -> Result<Option<String>> {
    Ok(None)
}

/// Check if a workspace has an active session.
///
/// Reads `workspace_session` directly — that table is still alive (it's
/// the analytics table renamed to `sessions` in phase 6).
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
