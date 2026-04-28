//! Tests for session drain primitives — drain, resume, cleanup, tool count.

use anyhow::Result;
use tempfile::tempdir;

use crate::db::session_drain::*;
use crate::db::session_events::insert_session_with_started_event;
use crate::events::types::SessionStarted;

fn init() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();
    (tmp, ship_dir)
}

fn create_test_session(_ship_dir: &std::path::Path, session_id: &str, workspace_id: &str) {
    let branch = workspace_id;
    let payload = SessionStarted {
        goal: Some("test".into()),
        workspace_id: workspace_id.into(),
        branch: branch.into(),
        ..Default::default()
    };
    insert_session_with_started_event(session_id, workspace_id, &payload).unwrap();
}

// ── find_draining_session ───────────────────────────────────────────────────

#[test]
fn find_draining_session_returns_none_when_no_draining() -> Result<()> {
    let (_tmp, ship_dir) = init();
    create_test_session(&ship_dir, "sess-1", "ws-find-none");

    let result = find_draining_session("ws-find-none", "agent-1")?;
    assert!(result.is_none());
    Ok(())
}

// ── drain_session ───────────────────────────────────────────────────────────

#[test]
fn drain_session_transitions_active_session() -> Result<()> {
    let (_tmp, ship_dir) = init();
    create_test_session(&ship_dir, "sess-drain", "ws-drain");

    drain_session("sess-drain")?;
    Ok(())
}

#[test]
fn drain_session_errors_for_missing_session() {
    let (_tmp, _ship_dir) = init();

    let err = drain_session("nonexistent").unwrap_err();
    assert!(err.to_string().contains("not found"));
}

// ── resume_session ──────────────────────────────────────────────────────────

#[test]
fn resume_session_errors_for_missing_session() {
    let (_tmp, _ship_dir) = init();

    let err = resume_session("nonexistent").unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn resume_after_drain_succeeds() -> Result<()> {
    let (_tmp, ship_dir) = init();
    create_test_session(&ship_dir, "sess-resume", "ws-resume");

    drain_session("sess-resume")?;
    resume_session("sess-resume")?;
    Ok(())
}

// ── increment_tool_count ────────────────────────────────────────────────────

#[test]
fn increment_tool_count_succeeds() -> Result<()> {
    let (_tmp, ship_dir) = init();
    create_test_session(&ship_dir, "sess-tool", "ws-tool");

    increment_tool_count("sess-tool")?;
    Ok(())
}

#[test]
fn increment_tool_count_multiple_times() -> Result<()> {
    let (_tmp, ship_dir) = init();
    create_test_session(&ship_dir, "sess-multi", "ws-multi");

    increment_tool_count("sess-multi")?;
    increment_tool_count("sess-multi")?;
    increment_tool_count("sess-multi")?;
    Ok(())
}

#[test]
fn increment_tool_count_errors_for_missing_session() {
    let (_tmp, _ship_dir) = init();

    let err = increment_tool_count("nonexistent").unwrap_err();
    assert!(err.to_string().contains("not found"));
}

// ── cleanup_stale_draining ──────────────────────────────────────────────────

#[test]
fn cleanup_stale_draining_returns_zero_with_no_stale() -> Result<()> {
    let (_tmp, _ship_dir) = init();

    let count = cleanup_stale_draining(60)?;
    assert_eq!(count, 0);
    Ok(())
}

#[test]
fn cleanup_stale_draining_does_not_affect_active_sessions() -> Result<()> {
    let (_tmp, ship_dir) = init();
    create_test_session(&ship_dir, "sess-active", "ws-active");

    let count = cleanup_stale_draining(0)?;
    assert_eq!(count, 0, "active sessions should not be cleaned up");
    Ok(())
}
