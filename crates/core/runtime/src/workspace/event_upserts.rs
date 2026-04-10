//! Lifecycle-event-aware workspace functions.
//!
//! Each function emits a typed workspace event. The event append and
//! projection update happen atomically in a single SQLite transaction
//! inside `workspace_events::run_tx`.

use crate::db::workspace_events::{
    emit_workspace_agent_changed, emit_workspace_compiled,
    emit_workspace_compile_failed, emit_workspace_created,
    emit_workspace_deleted, emit_workspace_started,
    emit_workspace_tmux_assigned,
};
use crate::events::types::{
    WorkspaceAgentChanged, WorkspaceCompileFailed,
    WorkspaceCompiled, WorkspaceCreated, WorkspaceStarted,
    WorkspaceTmuxAssigned,
};
use anyhow::Result;
use std::path::Path;

use super::helpers::workspace_id_from_branch;

// ── public event-emitting functions ──────────────────────────────────────────

pub fn upsert_workspace_on_compiled(
    _ship_dir: &Path,
    branch: &str,
    config_generation: u32,
    duration_ms: u64,
) -> Result<()> {
    let payload = WorkspaceCompiled {
        config_generation,
        duration_ms,
    };
    emit_workspace_compiled(branch, &payload)?;
    Ok(())
}

pub fn upsert_workspace_on_compile_failed(
    _ship_dir: &Path,
    branch: &str,
    error: &str,
) -> Result<()> {
    let payload = WorkspaceCompileFailed { error: error.to_string() };
    emit_workspace_compile_failed(branch, &payload)?;
    Ok(())
}

pub fn upsert_workspace_on_created(
    _ship_dir: &Path,
    branch: &str,
    path: Option<&str>,
    active_agent: Option<&str>,
) -> Result<()> {
    let workspace_id = workspace_id_from_branch(branch);
    let payload = WorkspaceCreated {
        workspace_id,
        workspace_type: "feature".to_string(),
        active_agent: active_agent.map(str::to_string),
        providers: Vec::new(),
        mcp_servers: Vec::new(),
        skills: Vec::new(),
        path: path.map(str::to_string),
    };
    emit_workspace_created(branch, &payload)?;
    Ok(())
}

pub fn upsert_workspace_on_deleted(_ship_dir: &Path, branch: &str) -> Result<()> {
    emit_workspace_deleted(branch)?;
    Ok(())
}

pub fn emit_workspace_agent_changed_event(
    _ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<()> {
    let payload = WorkspaceAgentChanged { agent_id: agent_id.map(str::to_string) };
    emit_workspace_agent_changed(branch, &payload)?;
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
