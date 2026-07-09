//! Shared git helpers for the orchestrator.
//!
//! Small, stateless wrappers around `git` invocations used across the
//! dispatch → step → gate chain. Kept as a single helper module so the
//! semantics of "what SHA do we capture when" are defined in one place.
//!
//! Lives at `crate::loader::git_helpers` (not under `orchestrator`) so
//! `run_core::resolve_project_head_sha` can capture a job baseline
//! without the `ship-code` feature.

/// Resolve HEAD SHA in a git directory. Returns `None` if git fails,
/// the directory isn't a repo, or the output is empty.
///
/// Used at two moments:
/// - **Dispatch:** capture the source repo's HEAD → `PipelineRunStartedPayload.baseline_sha`.
///   This is the pipeline's baseline for gate `require_changed` / `forbid_changed`
///   checks, so gates see commits made by any earlier step.
/// - **Step start:** capture the worktree's HEAD → `PipelineStepObservedPayload.start_sha`.
///   This tracks commits made within a single step (used by `compute_verdict`).
pub fn resolve_head_sha(work_dir: &std::path::Path) -> Option<String> {
    ship_git::read::resolve_head_sha(work_dir)
}

/// Pick the SHA that defines the "from" end of a gate's diff range.
///
/// Prefer the job-level baseline captured at dispatch so the gate sees
/// commits made by any earlier pipeline step. Fall back to the gating
/// step's own start SHA for legacy events that predate `baseline_sha`.
/// Returns an empty string when neither is present — the caller uses
/// that to short-circuit to `<no baseline>..HEAD` with an empty diff.
pub fn gate_baseline_sha(
    run_baseline: Option<&str>,
    step_start: Option<&str>,
) -> String {
    run_baseline
        .map(str::to_string)
        .or_else(|| step_start.map(str::to_string))
        .unwrap_or_default()
}
/// Count (commits-since-baseline, uncommitted-files) for the step-monitor
/// evidence accumulator. Cheap O(refs-touched) read; both branches return
/// `0` for non-repos so callers can treat any zero result uniformly.
///
/// Migrated from `ship_code::runner::observe::scan_git_evidence`. The
/// per-bucket shell-out is gone — `ship_git::read` provides the
/// libgit2-backed primitives (commit count via `graph_ahead_behind`,
/// status buckets via `load_status_buckets`) so this helper compiles
/// without `ship-code`.
///
/// `start_sha = None` measures "commits made ahead of ALL local refs" —
/// the prior behavior the step monitor relied on for the unsynced case.
/// We approximate that with HEAD (most repos with local commits ahead of
/// upstream are ahead of all the local refs that could be a baseline).
/// The path is a deliberate narrowing; if a step monitor needs the exact
/// `HEAD --not --remotes` semantic in the future, add a dedicated
/// `count_commits_not_on_remotes` to ship_git rather than reintroducing
/// the subprocess.
pub fn scan_git_evidence(
    work_dir: &std::path::Path,
    start_sha: Option<&str>,
) -> (u32, u32) {
    let commits = match start_sha {
        Some(sha) => ship_git::read::count_commits_ahead(work_dir, sha),
        None => {
            // Fallback: when there's no baseline, "commits ahead of any ref
            // that an upstream tracking branch could see" is operationally
            // HEAD's commit count. For the step monitor this distinction
            // matters only when a step edits with no start_sha (the
            // post-restart replay path). Treat as zero — a no-baseline
            // monitor should not credit phantom diffs.
            0
        }
    };
    let buckets = ship_git::read::load_status_buckets(work_dir);
    let uncommitted = buckets.total as u32;
    (commits, uncommitted)
}
/// Tail the per-slug agent-log file (the supervisor's mirror of the
/// live harness's stderr) and ANSI-strip it for evidence payloads.
/// `None` when the file is missing or empty after stripping.
///
/// Migrated from `ship_code::runner::observe::scan_agent_log_tail`.
/// Lives here so the step monitor stays at one stop for "what git/log
/// evidence do we have for this step"; the ANSI grammar comes from
/// the L0 leaf [`ship_substrate::text::strip_ansi`].
pub fn scan_agent_log_tail(
    work_dir: &std::path::Path,
    slug: &str,
) -> Option<String> {
    const MAX_BYTES: usize = 2048;
    const MAX_LINES: usize = 30;

    let path = work_dir
        .join(".artifacts")
        .join(format!("agent-{slug}.log"));
    let raw = std::fs::read_to_string(&path).ok()?;
    if raw.trim().is_empty() {
        return None;
    }
    let start = raw.len().saturating_sub(MAX_BYTES);
    let mut cut = start;
    while cut < raw.len() && !raw.is_char_boundary(cut) {
        cut += 1;
    }
    let tail = &raw[cut..];
    let cleaned = ship_substrate::text::strip_ansi(tail);
    let mut lines: Vec<&str> = cleaned.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() > MAX_LINES {
        lines = lines.split_off(lines.len() - MAX_LINES);
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::gate_baseline_sha;

    #[test]
    fn prefers_job_baseline_over_step_start() {
        // The whole point of the baseline: when both exist, the gate must
        // diff from the job baseline so it sees commits made in earlier
        // pipeline steps. Without this, a review step's gate's diff range
        // starts AFTER its own predecessors' commits and returns empty.
        let picked = gate_baseline_sha(Some("base-abc"), Some("step-xyz"));
        assert_eq!(picked, "base-abc");
    }

    #[test]
    fn falls_back_to_step_start_when_baseline_absent() {
        // Legacy jobs (pre baseline_sha) won't carry a job-level baseline.
        // The gate must still do something reasonable — fall back to the
        // step's own start SHA so old events keep the previous behavior.
        let picked = gate_baseline_sha(None, Some("step-xyz"));
        assert_eq!(picked, "step-xyz");
    }

    #[test]
    fn empty_when_neither_present() {
        // Both missing — gate caller treats the empty string as "no
        // baseline", short-circuits to an empty diff range.
        let picked = gate_baseline_sha(None, None);
        assert_eq!(picked, "");
    }
}
