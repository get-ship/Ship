//! Workspace state reads — projection table queries.
//!
//! All writes go through events and projections. These functions read from the
//! `workspace` projection table in platform.db.

use anyhow::Result;
use sqlx::Row;

use super::types::{WorkspaceDbListRow, WorkspaceDbRow};
use super::{block_on, open_db};

/// Retrieve the workspace record by id (or branch as fallback), or None if none exists.
pub fn get_workspace_by_id_db(id: &str) -> Result<Option<(String, WorkspaceDbRow)>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT branch, COALESCE(id, branch), path, \
             active_agent, created_at \
             FROM workspace WHERE id = ? OR branch = ? LIMIT 1",
        )
        .bind(id)
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        let branch: String = row.get(0);
        let rec_id: String = row.get(1);
        let path: Option<String> = row.get(2);
        let active_agent: Option<String> = row.get(3);
        let created_at: Option<String> = row.get(4);
        Ok(Some((
            branch,
            (rec_id, path, active_agent, created_at),
        )))
    } else {
        Ok(None)
    }
}

/// Retrieve the workspace record for the given branch, or None if none exists.
pub fn get_workspace_db(branch: &str) -> Result<Option<WorkspaceDbRow>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT COALESCE(id, branch), path, \
             active_agent, created_at \
             FROM workspace WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        let id: String = row.get(0);
        let path: Option<String> = row.get(1);
        let active_agent: Option<String> = row.get(2);
        let created_at: Option<String> = row.get(3);
        Ok(Some((id, path, active_agent, created_at)))
    } else {
        Ok(None)
    }
}

pub fn list_workspaces_db() -> Result<Vec<WorkspaceDbListRow>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT branch, COALESCE(id, branch), path, \
             active_agent, created_at \
             FROM workspace \
             ORDER BY created_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let branch: String = row.get(0);
        let id: String = row.get(1);
        let path: Option<String> = row.get(2);
        let active_agent: Option<String> = row.get(3);
        let created_at: Option<String> = row.get(4);

        result.push((branch, id, path, active_agent, created_at));
    }
    Ok(result)
}

/// Retrieve the tmux_session_name for a workspace by id or branch.
pub fn get_workspace_tmux_session_db(id: &str) -> Result<Option<String>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT tmux_session_name FROM workspace WHERE id = ? OR branch = ? LIMIT 1",
        )
        .bind(id)
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row_opt.flatten())
}

/// Retrieve the current config_generation for a workspace by id or branch.
pub fn get_workspace_config_generation_db(id: &str) -> Result<Option<i64>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query_scalar::<_, i64>(
            "SELECT config_generation FROM workspace WHERE id = ? OR branch = ? LIMIT 1",
        )
        .bind(id)
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row_opt)
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
