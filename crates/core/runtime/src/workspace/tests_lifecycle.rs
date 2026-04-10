//! Tests for workspace lifecycle — create, agent, tmux, seed.

use crate::workspace::*;
use anyhow::Result;
use tempfile::tempdir;

// ── create_workspace ────────────────────────────────────────────────────────

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
    assert_eq!(ws.name, "feature/my-test");
    Ok(())
}

#[test]
fn create_workspace_with_path() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    let ws = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/with-path".into(),
            path: Some("/some/path".into()),
            ..Default::default()
        },
    )?;
    assert_eq!(ws.path.as_deref(), Some("/some/path"));
    Ok(())
}

#[test]
fn create_workspace_update_preserves_existing_path() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

    create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/preserve".into(),
            path: Some("/path/a".into()),
            ..Default::default()
        },
    )?;

    // Update without changing path — it should persist.
    let updated = create_workspace(
        &ship_dir,
        CreateWorkspaceRequest {
            branch: "feature/preserve".into(),
            ..Default::default()
        },
    )?;
    assert_eq!(updated.path.as_deref(), Some("/path/a"));
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
    assert!(workspaces.iter().any(|w| w.name == "ship"));
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
        .filter(|w| w.name == "ship")
        .count();
    assert_eq!(count, 1, "seed_service_workspace must be idempotent");
    Ok(())
}
