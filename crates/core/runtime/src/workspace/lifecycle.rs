use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use crate::errors::WorkspaceError;
use super::crud::get_workspace;
use super::event_upserts::upsert_workspace_on_created;
use super::helpers::*;
use super::types::*;
use super::types_session::*;

/// Create or update a workspace record.
pub fn create_workspace(ship_dir: &Path, request: CreateWorkspaceRequest) -> Result<Workspace, WorkspaceError> {
    let branch = ensure_branch_key(&request.branch)?.to_string();
    let now = Utc::now();

    let existing = get_workspace(ship_dir, &branch)?;
    let mut workspace = existing.unwrap_or_else(|| new_workspace(&branch, now));

    if let Some(path) = request.path {
        let trimmed = path.trim();
        workspace.path = if trimmed.is_empty() { None } else { Some(trimmed.to_string()) };
    }

    workspace.id = workspace_id_from_branch(&branch);
    workspace.name = branch;

    upsert_workspace_on_created(
        ship_dir,
        &workspace.name,
        workspace.path.as_deref(),
    )?;
    Ok(workspace)
}

/// Set (or clear) the tmux session name for a workspace.
pub fn set_workspace_tmux_session(
    ship_dir: &Path,
    branch: &str,
    session_name: Option<&str>,
) -> Result<Workspace, WorkspaceError> {
    let branch = ensure_branch_key(branch)?;
    get_workspace(ship_dir, branch)?
        .ok_or_else(|| WorkspaceError::NotFound { branch: branch.to_string() })?;
    super::event_upserts::upsert_workspace_on_tmux_assigned(ship_dir, branch, session_name)?;
    get_workspace(ship_dir, branch)?
        .ok_or_else(|| WorkspaceError::NotFound { branch: branch.to_string() })
}

/// Write path and tmux_session_name to the workspace record.
/// Idempotent — safe to call multiple times with the same values.
pub fn set_workspace_started(
    ship_dir: &Path,
    branch: &str,
    worktree_path: &Path,
    tmux_session: &str,
) -> Result<(), WorkspaceError> {
    let branch = ensure_branch_key(branch)?;
    get_workspace(ship_dir, branch)?
        .ok_or_else(|| WorkspaceError::NotFound { branch: branch.to_string() })?;
    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree_path contains non-UTF-8 characters"))?;
    super::event_upserts::upsert_workspace_on_started(ship_dir, branch, path_str, tmux_session)?;
    Ok(())
}

