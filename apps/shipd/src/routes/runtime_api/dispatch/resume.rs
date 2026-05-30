//! Resume resolution — finding and validating a prior session for `resume: true`.

use axum::http::StatusCode;

/// Snapshot of a prior session that the dispatch handler reuses when
/// `resume: true`. Carries the worktree slug/branch plus the
/// `.ship/session.json` record needed to build the harness-specific
/// resume command downstream.
pub(super) struct ResumeOutcome {
    pub slug: String,
    pub branch: String,
    pub session: ship_platform::services::harnesses::session_file::SessionFile,
}

#[derive(Debug)]
pub(super) enum ResumeResolveError {
    Status {
        status: StatusCode,
        message: String,
    },
}

/// Locate the prior session for `(agent, workspace)`:
/// 1. Use the sessions DB to find the slug + branch (most-recent match).
/// 2. Read `<worktree>/.ship/session.json` for the harness-native id.
/// 3. Verify the requested harness (when provided) matches the session.
/// 4. Verify the resolved harness adapter actually supports `build_resume_args`.
///
/// Every failure path returns a typed error the HTTP handler renders as
/// a 400 (the spec forbids silent fresh launches).
pub(super) fn resolve_resume(
    agent: &str,
    workspace: &str,
    requested_harness: Option<&str>,
    worktrees_root: &std::path::Path,
) -> Result<ResumeOutcome, ResumeResolveError> {
    let sessions = ship_substrate::db::session::list_sessions_db(Some(workspace), 100)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("resume lookup failed: {e}"),
        })?;
    let hit = sessions
        .into_iter()
        .find(|s| s.agent_id.as_deref() == Some(agent))
        .ok_or_else(|| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "resume: no existing session for agent `{agent}` in workspace `{workspace}`"
            ),
        })?;

    let work_dir = worktrees_root.join(&hit.id);
    let session_file = load_and_validate_session_file(&work_dir, &hit.id, requested_harness)?;

    Ok(ResumeOutcome {
        slug: hit.id,
        branch: hit.branch,
        session: session_file,
    })
}

/// Worktree-local part of the resume resolver: reads `.ship/session.json`,
/// checks the harness matches `requested_harness` (when supplied), and
/// confirms the adapter's `build_resume_args` produces a valid command for
/// the recorded session id. Returns the parsed session file on success.
///
/// Split out so unit tests can exercise the file-/adapter-validation
/// branches without seeding the sessions DB.
pub(super) fn load_and_validate_session_file(
    work_dir: &std::path::Path,
    slug: &str,
    requested_harness: Option<&str>,
) -> Result<ship_platform::services::harnesses::session_file::SessionFile, ResumeResolveError> {
    let session_file = ship_platform::services::harnesses::session_file::read_session_file(work_dir)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!("resume: failed to read session file: {e}"),
        })?
        .ok_or_else(|| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "resume: no .ship/session.json in worktree `{}` — the prior run did not capture a harness session id",
                work_dir.display()
            ),
        })?;

    if let Some(req_h) = requested_harness
        && req_h != session_file.harness
    {
        return Err(ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!(
                "resume: harness mismatch — request asked for `{req_h}` but the session was opened with `{}`",
                session_file.harness
            ),
        });
    }

    // Verify the adapter actually supports resume before promising the
    // caller it'll work. Cursor will fail here; the dispatch path
    // then never hands off to the executor.
    let adapter = ship_platform::services::harness_adapter::adapter_for(&session_file.harness)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!("resume: {e}"),
        })?;
    let profile = ship_platform::services::dispatch_ports::LaunchProfile {
        harness: Some(session_file.harness.clone()),
        persist_session: true,
        ..Default::default()
    };
    adapter
        .build_resume_args(slug, &session_file.session_id, &profile)
        .map_err(|e| ResumeResolveError::Status {
            status: StatusCode::BAD_REQUEST,
            message: format!("resume: {e}"),
        })?;

    Ok(session_file)
}
