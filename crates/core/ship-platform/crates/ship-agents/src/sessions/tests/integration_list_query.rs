/// Integration tests — sessions list/query edge cases: limit enforcement,
/// multi-branch isolation, cross-branch listing, ordering, and no-active-session
/// queries.
///
/// These tests complement the unit coverage in `tests_session.rs` (which
/// covers the basic happy-path list) by exercising boundary conditions on
/// `list_sessions` and `get_active_session`.
use anyhow::Result;
use tempfile::tempdir;

use crate::sessions::{
    end_session, get_active_session, list_sessions, start_session, EndSessionRequest,
    SessionStatus,
};
use crate::testutil::init_project;

// ── get_active_session ────────────────────────────────────────────────────────

#[test]
fn sessions_list_query_get_active_returns_none_when_no_session() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let result = get_active_session(&ship_dir, "feature/empty")?;
    assert!(result.is_none(), "must return None when no session exists");
    Ok(())
}

#[test]
fn sessions_list_query_get_active_returns_none_after_end() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    start_session(&ship_dir, "feature/lq-ended", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-ended",
        EndSessionRequest::default(),
    )?;

    let result = get_active_session(&ship_dir, "feature/lq-ended")?;
    assert!(result.is_none(), "must return None after session is ended");
    Ok(())
}

#[test]
fn sessions_list_query_get_active_returns_session_while_active() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let started = start_session(&ship_dir, "feature/lq-active", None, None, None)?;
    let active = get_active_session(&ship_dir, "feature/lq-active")?
        .expect("must find active session");

    assert_eq!(active.id, started.id);
    assert_eq!(active.status, SessionStatus::Active);
    Ok(())
}

#[test]
fn sessions_list_query_get_active_rejects_empty_branch() {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path()).unwrap();

    let err = get_active_session(&ship_dir, "").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

// ── list_sessions — limit enforcement ────────────────────────────────────────

#[test]
fn sessions_list_query_list_respects_limit() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    // Create 5 sessions (start + end each so they don't collide on the
    // one-active-per-space rule).
    for i in 0..5 {
        start_session(
            &ship_dir,
            &format!("feature/lq-limit-{i}"),
            None,
            None,
            None,
        )?;
        end_session(
            &ship_dir,
            &format!("feature/lq-limit-{i}"),
            EndSessionRequest::default(),
        )?;
    }

    let limited = list_sessions(&ship_dir, None, 3)?;
    assert!(
        limited.len() <= 3,
        "list_sessions must respect limit=3; got {}",
        limited.len()
    );
    Ok(())
}

#[test]
fn sessions_list_query_list_limit_one_returns_at_most_one() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    // Create two sessions.
    start_session(&ship_dir, "feature/lq-limit-one-a", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-limit-one-a",
        EndSessionRequest::default(),
    )?;
    start_session(&ship_dir, "feature/lq-limit-one-b", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-limit-one-b",
        EndSessionRequest::default(),
    )?;

    let result = list_sessions(&ship_dir, None, 1)?;
    assert!(
        result.len() <= 1,
        "limit=1 must return at most 1 session; got {}",
        result.len()
    );
    Ok(())
}

// ── list_sessions — branch filtering ─────────────────────────────────────────

#[test]
fn sessions_list_query_list_with_branch_excludes_other_branches() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    start_session(&ship_dir, "feature/lq-branch-a", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-branch-a",
        EndSessionRequest::default(),
    )?;
    start_session(&ship_dir, "feature/lq-branch-b", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-branch-b",
        EndSessionRequest::default(),
    )?;

    let only_a = list_sessions(&ship_dir, Some("feature/lq-branch-a"), 10)?;
    assert!(
        only_a.iter().all(|s| s.branch == "feature/lq-branch-a"),
        "branch filter must only return sessions for that branch"
    );
    assert!(!only_a.is_empty(), "must return at least one session for branch-a");
    Ok(())
}

#[test]
fn sessions_list_query_list_no_branch_filter_includes_all() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    start_session(&ship_dir, "feature/lq-all-x", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-all-x",
        EndSessionRequest::default(),
    )?;
    start_session(&ship_dir, "feature/lq-all-y", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-all-y",
        EndSessionRequest::default(),
    )?;

    let all = list_sessions(&ship_dir, None, 10)?;
    let branches: Vec<&str> = all.iter().map(|s| s.branch.as_str()).collect();
    assert!(
        branches.contains(&"feature/lq-all-x"),
        "all-sessions list must include branch-x"
    );
    assert!(
        branches.contains(&"feature/lq-all-y"),
        "all-sessions list must include branch-y"
    );
    Ok(())
}

#[test]
fn sessions_list_query_list_unknown_branch_returns_empty() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let result = list_sessions(&ship_dir, Some("feature/nonexistent-xyz"), 10)?;
    assert!(
        result.is_empty(),
        "unknown branch must return empty list, not an error"
    );
    Ok(())
}

// ── list_sessions — session status in returned records ────────────────────────

#[test]
fn sessions_list_query_list_returns_ended_sessions() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/lq-status", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/lq-status",
        EndSessionRequest::default(),
    )?;

    let sessions = list_sessions(&ship_dir, Some("feature/lq-status"), 10)?;
    let found = sessions.iter().find(|s| s.id == session.id)
        .expect("ended session must appear in list");
    assert_eq!(found.status, SessionStatus::Ended);
    Ok(())
}

#[test]
fn sessions_list_query_list_includes_active_session() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/lq-active-list", None, None, None)?;

    let sessions = list_sessions(&ship_dir, Some("feature/lq-active-list"), 10)?;
    let found = sessions.iter().find(|s| s.id == session.id)
        .expect("active session must appear in list");
    assert_eq!(found.status, SessionStatus::Active);
    Ok(())
}
