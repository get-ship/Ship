//! Per-run git-level stats for the pipelines render layer.
//! Relocated from `shipd::services::pipelines::view_state` (L2 → L1).

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
    BranchStats { commits_ahead, dirty_files }
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
