//! Tests for session lifecycle — start, end, attach, harness resolution.

use crate::sessions::*;
use anyhow::Result;
use tempfile::tempdir;

fn init_project_with_harness(tmp: &tempfile::TempDir) -> Result<std::path::PathBuf> {
    let ship_dir = crate::testutil::init_project(tmp.path())?;
    let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
    config.harnesses = vec!["claude".to_string()];
    crate::config::save_config(&config, Some(ship_dir.clone()))?;
    Ok(ship_dir)
}

// ── start_session ─────────────────────────────────────────────────

#[test]
fn start_session_auto_creates_workspace() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    let session = start_session(
        &ship_dir,
        "feature/auto-ws",
        Some("test goal".into()),
        None,
        None,
    )?;
    assert_eq!(session.branch, "feature/auto-ws");
    assert_eq!(session.status, SessionStatus::Active);
    assert_eq!(session.goal.as_deref(), Some("test goal"));
    Ok(())
}

#[test]
fn start_session_on_existing_workspace() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    let session = start_session(
        &ship_dir,
        "feature/existing-session",
        None,
        None,
        None,
    )?;
    assert_eq!(session.status, SessionStatus::Active);
    Ok(())
}

#[test]
fn start_session_returns_existing_active_session() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    let first = start_session(
        &ship_dir,
        "feature/attach",
        Some("first".into()),
        None,
        None,
    )?;

    let second = start_session(
        &ship_dir,
        "feature/attach",
        Some("second".into()),
        None,
        None,
    )?;

    assert_eq!(first.id, second.id, "should return existing active session");
    Ok(())
}

#[test]
fn start_session_rejects_empty_branch() {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project_with_harness(&tmp).unwrap();

    let err = start_session(&ship_dir, "", None, None, None).unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn start_session_rejects_empty_harness() {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project_with_harness(&tmp).unwrap();

    let err = start_session(
        &ship_dir,
        "feature/empty-harness",
        None,
        None,
        Some("".into()),
    )
    .unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn start_session_rejects_disallowed_harness() {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project_with_harness(&tmp).unwrap();

    let err = start_session(
        &ship_dir,
        "feature/bad-harness",
        None,
        None,
        Some("codex".into()),
    )
    .unwrap_err();
    assert!(err.to_string().contains("not allowed"));
}

#[test]
fn start_session_with_explicit_harness() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    let session = start_session(
        &ship_dir,
        "feature/explicit-harness",
        None,
        None,
        Some("claude".into()),
    )?;
    assert_eq!(session.primary_harness.as_deref(), Some("claude"));
    Ok(())
}

#[test]
fn start_session_assigns_default_harness() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    let session = start_session(
        &ship_dir,
        "feature/default-harness",
        None,
        None,
        None,
    )?;
    assert!(session.primary_harness.is_some());
    Ok(())
}

// ── end_session ───────────────────────────────────────────────────

#[test]
fn end_session_errors_for_missing_workspace() {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project_with_harness(&tmp).unwrap();

    let err = end_session(
        &ship_dir,
        "nonexistent",
        EndSessionRequest::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("No active session"));
}

#[test]
fn end_session_errors_when_no_active_session() {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project_with_harness(&tmp).unwrap();

    let err = end_session(
        &ship_dir,
        "feature/no-session",
        EndSessionRequest::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("No active session"));
}

#[test]
fn end_session_completes_active_session() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    start_session(
        &ship_dir,
        "feature/end-test",
        Some("goal".into()),
        None,
        None,
    )?;

    let ended = end_session(
        &ship_dir,
        "feature/end-test",
        EndSessionRequest {
            summary: Some("done".into()),
            ..Default::default()
        },
    )?;
    assert_eq!(ended.status, SessionStatus::Ended);
    assert_eq!(ended.summary.as_deref(), Some("done"));
    assert!(ended.ended_at.is_some());
    Ok(())
}

#[test]
fn end_session_creates_session_record() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    start_session(
        &ship_dir,
        "feature/record-test",
        None,
        None,
        None,
    )?;

    let ended = end_session(
        &ship_dir,
        "feature/record-test",
        EndSessionRequest {
            summary: Some("summary".into()),
            model: Some("claude-sonnet".into()),
            files_changed: Some(5),
            ..Default::default()
        },
    )?;
    // Telemetry now lives on the session itself (no separate record table).
    assert_eq!(ended.model.as_deref(), Some("claude-sonnet"));
    assert_eq!(ended.files_changed, Some(5));
    Ok(())
}

#[test]
fn end_session_with_no_summary() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    start_session(
        &ship_dir,
        "feature/no-summary",
        None,
        None,
        None,
    )?;

    let ended = end_session(
        &ship_dir,
        "feature/no-summary",
        EndSessionRequest::default(),
    )?;
    assert_eq!(ended.status, SessionStatus::Ended);
    assert!(ended.summary.is_none());
    Ok(())
}

#[test]
fn start_new_session_after_ending_previous() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project_with_harness(&tmp)?;

    let first = start_session(
        &ship_dir,
        "feature/sequential",
        Some("first".into()),
        None,
        None,
    )?;

    end_session(
        &ship_dir,
        "feature/sequential",
        EndSessionRequest::default(),
    )?;

    let second = start_session(
        &ship_dir,
        "feature/sequential",
        Some("second".into()),
        None,
        None,
    )?;

    assert_ne!(first.id, second.id, "new session should have new id");
    assert_eq!(second.goal.as_deref(), Some("second"));
    Ok(())
}
