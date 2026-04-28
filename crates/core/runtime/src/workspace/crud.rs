use crate::db::workspace_state::{
    get_workspace_by_id_db, get_workspace_db, has_active_session_db, list_workspaces_db,
};
use crate::errors::WorkspaceError;
use super::event_upserts::upsert_workspace_on_deleted;
use anyhow::Result;
use std::path::Path;

use super::helpers::*;
use super::types::*;
use super::types_session::*;

// ---- Get / list / delete ---------------------------------------------------

/// Look up a workspace by its id (or branch as fallback).
pub fn get_workspace_by_id(_ship_dir: &Path, id: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_by_id_db(id)?;
    let Some((branch, (ws_id, path, created_at))) = row else {
        return Ok(None);
    };

    Ok(Some(Workspace {
        id: ws_id,
        name: branch,
        path,
        created_at: created_at
            .map(|s| parse_datetime(&s))
            .unwrap_or_else(chrono::Utc::now),
    }))
}

pub fn get_workspace(_ship_dir: &Path, branch: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_db(branch)?;
    let Some((id, path, created_at)) = row else {
        return Ok(None);
    };

    Ok(Some(Workspace {
        id,
        name: branch.to_string(),
        path,
        created_at: created_at
            .map(|s| parse_datetime(&s))
            .unwrap_or_else(chrono::Utc::now),
    }))
}

/// List all workspaces.
pub fn list_workspaces(ship_dir: &Path) -> Result<Vec<Workspace>> {
    list_workspaces_raw(ship_dir)
}

/// List workspaces directly from DB.
pub(crate) fn list_workspaces_raw(_ship_dir: &Path) -> Result<Vec<Workspace>> {
    let rows = list_workspaces_db()?;
    let mut workspaces = Vec::with_capacity(rows.len());
    for (branch, id, path, created_at) in rows {
        workspaces.push(Workspace {
            id,
            name: branch,
            path,
            created_at: created_at
                .map(|s| parse_datetime(&s))
                .unwrap_or_else(chrono::Utc::now),
        });
    }
    Ok(workspaces)
}

pub fn delete_workspace(ship_dir: &Path, id_or_branch: &str) -> Result<(), WorkspaceError> {
    let id_or_branch = ensure_branch_key(id_or_branch)?;
    let branch = match get_workspace_by_id_db(id_or_branch)? {
        Some((resolved_branch, _)) => resolved_branch,
        None => id_or_branch.to_string(),
    };
    upsert_workspace_on_deleted(ship_dir, &branch)?;
    Ok(())
}

// ---- Workspace status (computed from sessions) -----------------------------

/// Compute workspace status from session state.
pub fn workspace_status(workspace: &Workspace) -> WorkspaceStatus {
    match has_active_session_db(&workspace.id) {
        Ok(true) => WorkspaceStatus::Active,
        _ => WorkspaceStatus::Idle,
    }
}

// ---- Provider matrix / repair (public) -------------------------------------

pub fn get_workspace_provider_matrix(
    ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<WorkspaceProviderMatrix, WorkspaceError> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| WorkspaceError::NotFound { branch: branch.to_string() })?;
    Ok(super::compile::build_workspace_provider_matrix(ship_dir, &workspace, agent_id)?)
}

pub fn repair_workspace(
    ship_dir: &Path,
    branch: &str,
    dry_run: bool,
) -> Result<WorkspaceRepairReport, WorkspaceError> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| WorkspaceError::NotFound { branch: branch.to_string() })?;
    let mut matrix = get_workspace_provider_matrix(ship_dir, branch, None)?;

    let mut actions = Vec::new();
    let mut missing_provider_configs =
        super::compile::missing_provider_configs_for_workspace(
            &super::compile::resolve_workspace_context_root(ship_dir, &workspace),
            &matrix.allowed_providers,
        );
    if !missing_provider_configs.is_empty() {
        actions.push(format!(
            "missing provider configs: {}",
            missing_provider_configs.join(",")
        ));
    }

    let mut reapplied_compile = false;
    let needs_recompile = !missing_provider_configs.is_empty();

    if !dry_run && needs_recompile && matrix.resolution_error.is_none() {
        super::compile::compile_workspace_context(ship_dir, &workspace, None)?;
        matrix = get_workspace_provider_matrix(ship_dir, branch, None)?;
        let context_root = super::compile::resolve_workspace_context_root(ship_dir, &workspace);
        missing_provider_configs =
            super::compile::missing_provider_configs_for_workspace(&context_root, &matrix.allowed_providers);
        reapplied_compile = true;
        actions.push("recompiled workspace context".to_string());
    }

    Ok(WorkspaceRepairReport {
        workspace_branch: workspace.name.clone(),
        dry_run,
        agent_id: None,
        providers_expected: matrix.allowed_providers,
        needs_recompile: !missing_provider_configs.is_empty(),
        missing_provider_configs,
        reapplied_compile,
        resolution_error: matrix.resolution_error,
        actions,
    })
}
