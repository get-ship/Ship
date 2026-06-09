//! Resume resolution — finding and validating a prior session for `resume: true`.

/// Snapshot of a prior session that the dispatch handler reuses when
/// `resume: true`. Carries the worktree slug/branch plus the
/// `.ship/session.json` record needed to build the harness-specific
/// resume command downstream.
pub struct ResumeOutcome {
    pub slug: String,
    pub branch: String,
    pub session: ship_harnesses::harnesses::session_file::SessionFile,
}

#[derive(Debug)]
pub enum ResumeResolveError {
    /// Carries an HTTP-status hint (as u16) and operator-readable message.
    Status {
        /// HTTP status code for the caller to render (400 or 500).
        status: u16,
        message: String,
    },
}

/// Locate the prior session for `(agent, workspace)`:
/// 1. Use the sessions DB to find the slug + branch (most-recent match).
/// 2. Read `<worktree>/.ship/session.json` for the harness-native id.
/// 3. Verify the requested harness (when provided) matches the session.
/// 4. Verify the resolved harness adapter actually supports `build_resume_args`.
///
/// Every failure path returns a typed error the caller renders as
/// a 400 (the spec forbids silent fresh launches).
pub fn resolve_resume(
    agent: &str,
    workspace: &str,
    requested_harness: Option<&str>,
    worktrees_root: &std::path::Path,
) -> Result<ResumeOutcome, ResumeResolveError> {
    // Workbench session DB is gone; canonical session projection does not yet
    // expose a resume-lookup query. Resume is unavailable until the canonical
    // read-model is wired — return 400 so callers fail fast rather than silently.
    let _ = (agent, workspace, requested_harness, worktrees_root);
    Err(ResumeResolveError::Status {
        status: 400,
        message: "resume: session lookup unavailable (workbench session DB removed)".to_string(),
    })
}

/// Worktree-local part of the resume resolver: reads `.ship/session.json`,
/// checks the harness matches `requested_harness` (when supplied), and
/// confirms the adapter's `build_resume_args` produces a valid command for
/// the recorded session id. Returns the parsed session file on success.
///
/// Split out so unit tests can exercise the file-/adapter-validation
/// branches without seeding the sessions DB.
pub fn load_and_validate_session_file(
    work_dir: &std::path::Path,
    slug: &str,
    requested_harness: Option<&str>,
) -> Result<ship_harnesses::harnesses::session_file::SessionFile, ResumeResolveError> {
    let session_file = ship_harnesses::harnesses::session_file::read_session_file(work_dir)
        .map_err(|e| ResumeResolveError::Status {
            status: 400,
            message: format!("resume: failed to read session file: {e}"),
        })?
        .ok_or_else(|| ResumeResolveError::Status {
            status: 400,
            message: format!(
                "resume: no .ship/session.json in worktree `{}` — the prior run did not capture a harness session id",
                work_dir.display()
            ),
        })?;

    if let Some(req_h) = requested_harness
        && req_h != session_file.harness
    {
        return Err(ResumeResolveError::Status {
            status: 400,
            message: format!(
                "resume: harness mismatch — request asked for `{req_h}` but the session was opened with `{}`",
                session_file.harness
            ),
        });
    }

    // Verify the adapter actually supports resume before promising the
    // caller it'll work. Cursor will fail here; the dispatch path
    // then never hands off to the executor.
    let adapter = ship_harnesses::harness_adapter::adapter_for(&session_file.harness)
        .map_err(|e| ResumeResolveError::Status {
            status: 400,
            message: format!("resume: {e}"),
        })?;
    let profile = ship_substrate::services::dispatch_ports::LaunchProfile {
        harness: Some(session_file.harness.clone()),
        persist_session: true,
        ..Default::default()
    };
    adapter
        .build_resume_args(slug, &session_file.session_id, &profile)
        .map_err(|e| ResumeResolveError::Status {
            status: 400,
            message: format!("resume: {e}"),
        })?;

    Ok(session_file)
}
