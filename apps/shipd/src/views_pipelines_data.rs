//! Per-run git-level stats for the pipelines view.
//!
//! The runtime's `JobRecord` projection does not currently surface
//! per-step commit or dirty-file counts (see `.artifacts/ui-proposal/
//! state-gaps.md`). Extending `StepRecord` is out of scope for this
//! view. What *is* cheap and honest today is a per-run (per-worktree)
//! snapshot of `commits ahead of main` and `uncommitted files` — those
//! numbers are retrievable by shelling out to `git` in the run's
//! worktree directory, which the kernel already records on dispatch.
//!
//! These helpers are intentionally separate from `views_git_data` so
//! that module stays focused on the git view's own reads; we only
//! borrow its `run_git` shell wrapper.

use crate::views_git_data::run_git;
use std::path::Path;

/// Branch-level git stats for a single pipeline run.
///
/// Both fields default to `0` when the worktree is missing, is on
/// `main`, or git returns an error — callers should render `0` as
/// "clean / up to date" rather than as a failure.
#[derive(serde::Serialize, Clone, Debug, Default)]
pub struct BranchStats {
    /// `git rev-list --count main..HEAD` — number of commits on this
    /// run's branch that are not on `main`.
    pub commits_ahead: u32,
    /// `git status --porcelain | wc -l` — count of non-empty lines.
    pub dirty_files: u32,
}

/// Run both git queries against `worktree_path`. Errors are swallowed
/// into zeros on purpose: a missing worktree (the run finished and the
/// directory was pruned) is an expected condition, not a view failure.
pub fn branch_stats(worktree_path: &Path) -> BranchStats {
    if !worktree_path.exists() {
        return BranchStats::default();
    }
    let commits_ahead = run_git(worktree_path, &["rev-list", "--count", "main..HEAD"])
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let dirty_files = run_git(worktree_path, &["status", "--porcelain"])
        .ok()
        .map(|s| s.lines().filter(|l| !l.is_empty()).count() as u32)
        .unwrap_or(0);
    BranchStats {
        commits_ahead,
        dirty_files,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn missing_worktree_returns_defaults() {
        let p = PathBuf::from("/tmp/ship-does-not-exist-xyz-1234567890");
        let s = branch_stats(&p);
        assert_eq!(s.commits_ahead, 0);
        assert_eq!(s.dirty_files, 0);
    }

    #[test]
    fn default_stats_are_zero() {
        let s = BranchStats::default();
        assert_eq!(s.commits_ahead, 0);
        assert_eq!(s.dirty_files, 0);
    }
}
