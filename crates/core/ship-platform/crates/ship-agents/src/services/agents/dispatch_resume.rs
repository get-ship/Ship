//! Resume resolution — finding and validating a prior session for `resume: true`.

/// Snapshot of a prior session that the dispatch handler reuses when
/// `resume: true`. Carries the worktree slug/branch plus the
/// `.ship/session.json` record needed to build the harness-specific
/// resume command downstream.
pub struct ResumeOutcome {
    /// Session id (ULID). Used as slug in the canonical schema (no slug field).
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
/// 1. Derive `space_id` from the workspace path via the `projects` projection.
/// 2. Query `canonical_sessions` for the most recent resumable session for
///    `(agent_id, space_id)`.
/// 3. Scan `worktrees_root` for a worktree whose `.ship/session.json` matches
///    the found `session_id`.
/// 4. Derive the current branch via `git rev-parse --abbrev-ref HEAD`.
/// 5. Return `ResumeOutcome { slug: session_id, branch, session }`.
///
/// Every failure path returns a typed error the caller renders as
/// a 400 (the spec forbids silent fresh launches).
pub fn resolve_resume(
    agent: &str,
    workspace: &str,
    requested_harness: Option<&str>,
    worktrees_root: &std::path::Path,
) -> Result<ResumeOutcome, ResumeResolveError> {
    // 1. Resolve space_id from the workspace path.
    let space_record =
        ship_substrate::db::projects::find_by_path(workspace).map_err(|e| {
            ResumeResolveError::Status {
                status: 500,
                message: format!("resume: projects lookup failed: {e}"),
            }
        })?;
    let space_id = space_record
        .ok_or_else(|| ResumeResolveError::Status {
            status: 400,
            message: "resume: workspace is not a registered Ship project".to_string(),
        })?
        .id;

    // 2. Query canonical_sessions for the most recent resumable session.
    let canonical_session =
        crate::services::sessions::canonical_projection::load_latest_resumable_session(
            agent, &space_id,
        )
        .map_err(|e| ResumeResolveError::Status {
            status: 500,
            message: format!("resume: canonical sessions query failed: {e}"),
        })?
        .ok_or_else(|| ResumeResolveError::Status {
            status: 400,
            message: format!(
                "resume: no active or suspended session found for agent `{agent}` in this workspace"
            ),
        })?;

    let session_id = &canonical_session.session_id;

    // 3. Scan worktrees_root for a worktree whose .ship/session.json matches.
    let worktree_dir = find_worktree_for_session(session_id, worktrees_root)?;

    // 4. Load + validate the session file (harness check + build_resume_args).
    // slug = session_id in the canonical schema (no separate slug field).
    let session_file =
        load_and_validate_session_file(&worktree_dir, session_id, requested_harness)?;

    // 5. Derive branch.
    let branch = derive_branch(&worktree_dir)?;

    Ok(ResumeOutcome {
        slug: session_id.clone(),
        branch,
        session: session_file,
    })
}

/// Scan immediate subdirectories of `worktrees_root` for one whose
/// `.ship/session.json` records `session_id`.
///
/// O(worktrees) — acceptable for the cold resume path. A session_id-indexed
/// worktree registry should be added as a follow-up to avoid scan latency on
/// installations with many orphan worktrees.
fn find_worktree_for_session(
    session_id: &str,
    worktrees_root: &std::path::Path,
) -> Result<std::path::PathBuf, ResumeResolveError> {
    let entries = std::fs::read_dir(worktrees_root).map_err(|e| ResumeResolveError::Status {
        status: 500,
        message: format!(
            "resume: cannot read worktrees root `{}`: {e}",
            worktrees_root.display()
        ),
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Try to read .ship/session.json; skip any dir that lacks it.
        let sf = ship_harnesses::harnesses::session_file::read_session_file(&path)
            .unwrap_or(None);
        if let Some(sf) = sf {
            if sf.session_id == session_id {
                return Ok(path);
            }
        }
    }

    Err(ResumeResolveError::Status {
        status: 400,
        message: format!(
            "resume: session `{session_id}` found in projection but no matching \
             .ship/session.json under `{}`",
            worktrees_root.display()
        ),
    })
}

/// Run `git rev-parse --abbrev-ref HEAD` in `worktree_dir` to get the branch.
fn derive_branch(worktree_dir: &std::path::Path) -> Result<String, ResumeResolveError> {
    let output = std::process::Command::new("git")
        .args(["-C", worktree_dir.to_string_lossy().as_ref(), "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| ResumeResolveError::Status {
            status: 500,
            message: format!("resume: git rev-parse failed: {e}"),
        })?;

    if !output.status.success() {
        return Err(ResumeResolveError::Status {
            status: 500,
            message: format!(
                "resume: git rev-parse exited non-zero in `{}`: {}",
                worktree_dir.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
    let profile = harness_adapter::dispatch_ports::LaunchProfile {
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
