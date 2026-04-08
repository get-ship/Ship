//! Tests for workspace lifecycle — create, activate, transition, agent, tmux.

use crate::workspace::*;
use anyhow::Result;
use tempfile::tempdir;

// ── create_workspace ────────────────────────────────────────────────────────

#[test]
fn create_workspace_defaults_to_active() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let ws = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/default-status".into(),
            ..Default::default()
        },
    )?;
    assert_eq!(ws.status, WorkspaceStatus::Active);
    Ok(())
}

#[test]
fn create_workspace_with_explicit_archived_status() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let ws = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/archived".into(),
            status: Some(WorkspaceStatus::Archived),
            ..Default::default()
        },
    )?;
    assert_eq!(ws.status, WorkspaceStatus::Archived);
    Ok(())
}

#[test]
fn create_workspace_rejects_empty_branch() {
    let tmp = tempdir().unwrap();
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();

    let err = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "".into(),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn create_workspace_rejects_worktree_path_without_is_worktree() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let err = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/bad-path".into(),
            is_worktree: Some(false),
            worktree_path: Some("/some/path".into()),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Worktree path can only be set"));
    Ok(())
}

#[test]
fn create_workspace_sets_last_activated_at_when_active() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let ws = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/activated".into(),
            status: Some(WorkspaceStatus::Active),
            ..Default::default()
        },
    )?;
    assert!(ws.last_activated_at.is_some());
    Ok(())
}

#[test]
fn create_workspace_generates_id_from_branch() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let ws = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/my-test".into(),
            ..Default::default()
        },
    )?;
    assert_eq!(ws.id, "feature-my-test");
    assert_eq!(ws.branch, "feature/my-test");
    Ok(())
}

#[test]
fn create_workspace_update_preserves_existing_fields() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/preserve".into(),
            is_worktree: Some(true),
            worktree_path: Some("/path/a".into()),
            ..Default::default()
        },
    )?;

    // Update without changing worktree settings — they should persist.
    let updated = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/preserve".into(),
            ..Default::default()
        },
    )?;
    assert!(updated.is_worktree);
    assert_eq!(updated.worktree_path.as_deref(), Some("/path/a"));
    Ok(())
}

// ── transition_workspace_status ─────────────────────────────────────────────

#[test]
fn transition_active_to_archived() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/trans-archive".into(),
            status: Some(WorkspaceStatus::Active),
            ..Default::default()
        },
    )?;

    let ws = transition_workspace_status(&ship_dir, "feature/trans-archive", WorkspaceStatus::Archived)?;
    assert_eq!(ws.status, WorkspaceStatus::Archived);
    Ok(())
}

#[test]
fn transition_archived_to_active() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/trans-reactivate".into(),
            status: Some(WorkspaceStatus::Archived),
            ..Default::default()
        },
    )?;

    let ws = transition_workspace_status(
        &ship_dir,
        "feature/trans-reactivate",
        WorkspaceStatus::Active,
    )?;
    assert_eq!(ws.status, WorkspaceStatus::Active);
    assert!(ws.last_activated_at.is_some());
    Ok(())
}

#[test]
fn transition_active_to_active_is_noop() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/trans-noop".into(),
            status: Some(WorkspaceStatus::Active),
            ..Default::default()
        },
    )?;

    let ws = transition_workspace_status(&ship_dir, "feature/trans-noop", WorkspaceStatus::Active)?;
    assert_eq!(ws.status, WorkspaceStatus::Active);
    Ok(())
}

#[test]
fn transition_missing_workspace_errors() {
    let tmp = tempdir().unwrap();
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();

    let err = transition_workspace_status(&ship_dir, "nonexistent", WorkspaceStatus::Active)
        .unwrap_err();
    assert!(err.to_string().contains("Workspace not found"));
}

// ── activate_workspace ──────────────────────────────────────────────────────

#[test]
fn activate_creates_workspace_if_missing() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let ws = activate_workspace(&ship_dir, "feature/auto-create")?;
    assert_eq!(ws.status, WorkspaceStatus::Active);
    assert_eq!(ws.branch, "feature/auto-create");
    Ok(())
}

#[test]
fn activate_reactivates_archived_workspace() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/reactivate".into(),
            status: Some(WorkspaceStatus::Archived),
            ..Default::default()
        },
    )?;

    let ws = activate_workspace(&ship_dir, "feature/reactivate")?;
    assert_eq!(ws.status, WorkspaceStatus::Active);
    Ok(())
}

// ── set_workspace_active_agent ──────────────────────────────────────────────

#[test]
fn set_active_agent_errors_for_missing_workspace() {
    let tmp = tempdir().unwrap();
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();

    let err = set_workspace_active_agent(&ship_dir, "nonexistent", Some("agent"))
        .unwrap_err();
    assert!(err.to_string().contains("Workspace not found"));
}

#[test]
fn set_active_agent_clear_succeeds() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/clear-agent".into(),
            ..Default::default()
        },
    )?;

    let ws = set_workspace_active_agent(&ship_dir, "feature/clear-agent", None)?;
    assert!(ws.active_agent.is_none());
    Ok(())
}

// ── set_workspace_started ───────────────────────────────────────────────────

#[test]
fn set_started_writes_worktree_and_tmux() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/started".into(),
            ..Default::default()
        },
    )?;

    set_workspace_started(
        &ship_dir,
        "feature/started",
        std::path::Path::new("/worktrees/started"),
        "tmux-session-1",
    )?;
    Ok(())
}

#[test]
fn set_started_errors_for_missing_workspace() {
    let tmp = tempdir().unwrap();
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();

    let err = set_workspace_started(
        &ship_dir,
        "nonexistent",
        std::path::Path::new("/path"),
        "tmux",
    )
    .unwrap_err();
    assert!(err.to_string().contains("Workspace not found"));
}

// ── seed_service_workspace ──────────────────────────────────────────────────

#[test]
fn seed_service_workspace_creates_ship_workspace() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    seed_service_workspace(&ship_dir)?;
    let workspaces = list_workspaces(&ship_dir)?;
    assert!(workspaces.iter().any(|w| w.branch == "ship"));
    Ok(())
}

#[test]
fn seed_service_workspace_is_idempotent() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    seed_service_workspace(&ship_dir)?;
    seed_service_workspace(&ship_dir)?;
    let count = list_workspaces(&ship_dir)?
        .iter()
        .filter(|w| w.branch == "ship")
        .count();
    assert_eq!(count, 1, "seed_service_workspace must be idempotent");
    Ok(())
}

// ── sync_workspace ──────────────────────────────────────────────────────────

#[test]
fn sync_workspace_delegates_to_activate() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let ws = sync_workspace(&ship_dir, "feature/sync-test")?;
    assert_eq!(ws.status, WorkspaceStatus::Active);
    assert_eq!(ws.branch, "feature/sync-test");
    Ok(())
}

// ── validate_workspace_transition ───────────────────────────────────────────

#[test]
fn validate_transition_same_status_allowed() {
    assert!(validate_workspace_transition(WorkspaceStatus::Active, WorkspaceStatus::Active).is_ok());
    assert!(validate_workspace_transition(WorkspaceStatus::Archived, WorkspaceStatus::Archived).is_ok());
}

#[test]
fn validate_transition_active_to_archived_allowed() {
    assert!(validate_workspace_transition(WorkspaceStatus::Active, WorkspaceStatus::Archived).is_ok());
}

#[test]
fn validate_transition_archived_to_active_allowed() {
    assert!(validate_workspace_transition(WorkspaceStatus::Archived, WorkspaceStatus::Active).is_ok());
}
