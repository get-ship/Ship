//! Shared git helpers for the orchestrator.
//!
//! Small, stateless wrappers around `git` invocations used across the
//! dispatch → step → gate chain. Kept as a single helper module so the
//! semantics of "what SHA do we capture when" are defined in one place.

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
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(work_dir)
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
}

/// Pick the SHA that defines the "from" end of a gate's diff range.
///
/// Prefer the job-level baseline captured at dispatch so the gate sees
/// commits made by any earlier pipeline step. Fall back to the gating
/// step's own start SHA for legacy events that predate `baseline_sha`.
/// Returns an empty string when neither is present — the caller uses
/// that to short-circuit to `<no baseline>..HEAD` with an empty diff.
pub fn gate_baseline_sha(
    job_baseline: Option<&str>,
    step_start: Option<&str>,
) -> String {
    job_baseline
        .map(str::to_string)
        .or_else(|| step_start.map(str::to_string))
        .unwrap_or_default()
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
