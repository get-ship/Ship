//! Lifecycle-event-aware workspace functions.
//!
//! Each function emits a typed workspace event. The event append and
//! projection update happen atomically in a single SQLite transaction
//! inside `workspace_events::run_tx`.

use crate::db::workspace_events::{
    emit_workspace_created, emit_workspace_deleted, emit_workspace_started,
    emit_workspace_tmux_assigned,
};
use crate::events::types::{
    WorkspaceCreated, WorkspaceStarted, WorkspaceTmuxAssigned,
};
use anyhow::Result;
use std::path::Path;

use super::helpers::workspace_id_from_branch;

// ── public event-emitting functions ──────────────────────────────────────────

pub fn upsert_workspace_on_created(
    _ship_dir: &Path,
    branch: &str,
    path: Option<&str>,
) -> Result<()> {
    let workspace_id = workspace_id_from_branch(branch);
    let payload = WorkspaceCreated {
        workspace_id,
        path: path.map(str::to_string),
    };
    emit_workspace_created(branch, &payload)?;
    Ok(())
}

pub fn upsert_workspace_on_deleted(_ship_dir: &Path, branch: &str) -> Result<()> {
    emit_workspace_deleted(branch)?;
    Ok(())
}

pub fn upsert_workspace_on_tmux_assigned(
    _ship_dir: &Path,
    branch: &str,
    tmux_session_name: Option<&str>,
) -> Result<()> {
    let payload = WorkspaceTmuxAssigned {
        tmux_session_name: tmux_session_name.map(str::to_string),
    };
    emit_workspace_tmux_assigned(branch, &payload)?;
    Ok(())
}

pub fn upsert_workspace_on_started(
    _ship_dir: &Path,
    branch: &str,
    worktree_path: &str,
    tmux_session_name: &str,
) -> Result<()> {
    let payload = WorkspaceStarted {
        worktree_path: worktree_path.to_string(),
        tmux_session_name: tmux_session_name.to_string(),
    };
    emit_workspace_started(branch, &payload)?;
    Ok(())
}
