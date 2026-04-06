//! Tests for event_upserts — each workspace lifecycle function emits the
//! correct event type via the transactional event → projection path.

use anyhow::Result;
use tempfile::tempdir;

use crate::db::events::list_all_events;
use crate::workspace::event_upserts::*;
use crate::workspace::helpers::workspace_id_from_branch;

fn init() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();
    (tmp, ship_dir)
}

fn last_event_type() -> String {
    let events = list_all_events().unwrap();
    events.last().unwrap().event_type.clone()
}

fn events_of_type(event_type: &str) -> Vec<crate::events::EventEnvelope> {
    list_all_events()
        .unwrap()
        .into_iter()
        .filter(|e| e.event_type == event_type)
        .collect()
}

// ── workspace.created ───────────────────────────────────────────────────────

#[test]
fn upsert_created_emits_workspace_created_event() -> Result<()> {
    let (_tmp, ship_dir) = init();

    upsert_workspace_on_created(&ship_dir, "feat/test", false, None, None, "active")?;
    assert_eq!(last_event_type(), "workspace.created");

    let events = events_of_type("workspace.created");
    assert!(!events.is_empty());
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["status"], "active");
    assert_eq!(payload["workspace_id"], workspace_id_from_branch("feat/test"));
    Ok(())
}

#[test]
fn upsert_created_with_worktree() -> Result<()> {
    let (_tmp, ship_dir) = init();

    upsert_workspace_on_created(
        &ship_dir,
        "feat/wt",
        true,
        Some("/worktrees/wt"),
        None,
        "active",
    )?;

    let events = events_of_type("workspace.created");
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["is_worktree"], true);
    assert_eq!(payload["worktree_path"], "/worktrees/wt");
    Ok(())
}

#[test]
fn upsert_created_with_agent() -> Result<()> {
    let (_tmp, ship_dir) = init();

    upsert_workspace_on_created(
        &ship_dir,
        "feat/agent",
        false,
        None,
        Some("my-agent"),
        "active",
    )?;

    let events = events_of_type("workspace.created");
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["active_agent"], "my-agent");
    Ok(())
}

// ── workspace.activated ─────────────────────────────────────────────────────

#[test]
fn upsert_activated_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    // Need a workspace record first.
    upsert_workspace_on_created(&ship_dir, "feat/act", false, None, None, "active")?;

    upsert_workspace_on_activate(&ship_dir, "feat/act", Some("my-agent"), &["claude".into()])?;
    assert_eq!(last_event_type(), "workspace.activated");

    let events = events_of_type("workspace.activated");
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["agent_id"], "my-agent");
    assert_eq!(payload["providers"][0], "claude");
    Ok(())
}

#[test]
fn upsert_activated_without_agent() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/no-agent", false, None, None, "active")?;

    upsert_workspace_on_activate(&ship_dir, "feat/no-agent", None, &[])?;

    let events = events_of_type("workspace.activated");
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert!(payload["agent_id"].is_null());
    Ok(())
}

// ── workspace.compiled ──────────────────────────────────────────────────────

#[test]
fn upsert_compiled_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/comp", false, None, None, "active")?;

    upsert_workspace_on_compiled(&ship_dir, "feat/comp", 3, 150)?;

    let events = events_of_type("workspace.compiled");
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["config_generation"], 3);
    assert_eq!(payload["duration_ms"], 150);
    Ok(())
}

// ── workspace.compile_failed ────────────────────────────────────────────────

#[test]
fn upsert_compile_failed_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/fail", false, None, None, "active")?;

    upsert_workspace_on_compile_failed(&ship_dir, "feat/fail", "missing provider")?;

    let events = events_of_type("workspace.compile_failed");
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["error"], "missing provider");
    Ok(())
}

// ── workspace.archived ──────────────────────────────────────────────────────

#[test]
fn upsert_archived_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/arch", false, None, None, "active")?;

    upsert_workspace_on_archived(&ship_dir, "feat/arch")?;
    assert_eq!(last_event_type(), "workspace.archived");
    Ok(())
}

// ── workspace.status_changed ────────────────────────────────────────────────

#[test]
fn upsert_status_changed_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/status", false, None, None, "active")?;

    upsert_workspace_on_status_changed(&ship_dir, "feat/status", "active", "archived")?;

    let events = events_of_type("workspace.status_changed");
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["old_status"], "active");
    assert_eq!(payload["new_status"], "archived");
    Ok(())
}

// ── workspace.deleted ───────────────────────────────────────────────────────

#[test]
fn upsert_deleted_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/del", false, None, None, "active")?;

    upsert_workspace_on_deleted(&ship_dir, "feat/del")?;

    let events = events_of_type("workspace.deleted");
    assert_eq!(events.len(), 1);
    Ok(())
}

// ── workspace.agent_changed ─────────────────────────────────────────────────

#[test]
fn emit_agent_changed_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/agent-ch", false, None, None, "active")?;

    emit_workspace_agent_changed_event(&ship_dir, "feat/agent-ch", Some("new-agent"))?;

    let events = events_of_type("workspace.agent_changed");
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["agent_id"], "new-agent");
    Ok(())
}

#[test]
fn emit_agent_changed_with_clear() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/agent-clr", false, None, None, "active")?;

    emit_workspace_agent_changed_event(&ship_dir, "feat/agent-clr", None)?;

    let events = events_of_type("workspace.agent_changed");
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert!(payload["agent_id"].is_null());
    Ok(())
}

// ── workspace.reconciled ────────────────────────────────────────────────────

#[test]
fn upsert_reconciled_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/rec", false, None, None, "active")?;

    crate::workspace::event_upserts::upsert_workspace_on_reconciled(
        &ship_dir,
        "feat/rec",
        true,
        Some("/new/path"),
        "path changed",
    )?;

    let events = events_of_type("workspace.reconciled");
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["is_worktree"], true);
    assert_eq!(payload["worktree_path"], "/new/path");
    assert_eq!(payload["reason"], "path changed");
    Ok(())
}

// ── workspace.tmux_assigned ─────────────────────────────────────────────────

#[test]
fn upsert_tmux_assigned_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/tmux", false, None, None, "active")?;

    crate::workspace::event_upserts::upsert_workspace_on_tmux_assigned(
        &ship_dir,
        "feat/tmux",
        Some("my-session"),
    )?;

    let events = events_of_type("workspace.tmux_assigned");
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["tmux_session_name"], "my-session");
    Ok(())
}

#[test]
fn upsert_tmux_assigned_clear() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/tmux-clr", false, None, None, "active")?;

    crate::workspace::event_upserts::upsert_workspace_on_tmux_assigned(
        &ship_dir,
        "feat/tmux-clr",
        None,
    )?;

    let events = events_of_type("workspace.tmux_assigned");
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert!(payload["tmux_session_name"].is_null());
    Ok(())
}

// ── workspace.started ───────────────────────────────────────────────────────

#[test]
fn upsert_started_emits_event() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/start", false, None, None, "active")?;

    crate::workspace::event_upserts::upsert_workspace_on_started(
        &ship_dir,
        "feat/start",
        "/worktrees/start",
        "tmux-start",
    )?;

    let events = events_of_type("workspace.started");
    assert_eq!(events.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&events[0].payload_json)?;
    assert_eq!(payload["worktree_path"], "/worktrees/start");
    assert_eq!(payload["tmux_session_name"], "tmux-start");
    Ok(())
}

// ── Events are elevated ─────────────────────────────────────────────────────

#[test]
fn workspace_events_are_elevated() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/elev", false, None, None, "active")?;

    let events = events_of_type("workspace.created");
    assert!(events[0].elevated, "workspace events must be elevated");
    Ok(())
}

#[test]
fn workspace_events_carry_workspace_context() -> Result<()> {
    let (_tmp, ship_dir) = init();
    upsert_workspace_on_created(&ship_dir, "feat/ctx", false, None, None, "active")?;

    let events = events_of_type("workspace.created");
    assert_eq!(events[0].workspace_id.as_deref(), Some("feat/ctx"));
    assert_eq!(events[0].actor_id.as_deref(), Some("feat/ctx"));
    Ok(())
}
