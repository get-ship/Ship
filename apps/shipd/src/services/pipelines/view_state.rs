//! Per-run git-level stats for the pipelines render layer.
//!
//! The runtime's `PipelineRun` projection does not surface per-step
//! commit or dirty-file counts. What's cheap + honest today is a
//! per-run (per-worktree) snapshot of commits-ahead-of-main and
//! uncommitted-files counts — one `git rev-list --count` + one
//! `git status --porcelain` per run. We shell out rather than pulling
//! in libgit2 because worktree paths here are pipeline-owned and may
//! be pruned or mid-clone; subprocess failures just become zero.
//!
//! Lives alongside the pipelines service state because it's the
//! view-composition half of the same domain — keeping it as a sibling
//! module (`state` + `view_state`) preserves the 300-line cap while
//! honoring "one domain, one state home."

use std::path::Path;

fn run_git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = ship_substrate::spawn_bounded::Cmd::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git spawn: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Branch-level git stats for a single pipeline run.
///
/// Both fields default to `0` when the worktree is missing, on `main`,
/// or git returns an error — callers render `0` as "clean / up to date"
/// rather than a failure.
#[derive(serde::Serialize, Clone, Debug, Default)]
pub struct BranchStats {
    pub commits_ahead: u32,
    pub dirty_files: u32,
}

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
