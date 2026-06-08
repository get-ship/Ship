//! Git worktree resolution helpers for `default_worktree_resolver`.
//!
//! Relocated from the retired `project_api` module — these are the only
//! survivors of that REST grab-bag, kept because `default_worktree_resolver`
//! (in `init`) genuinely needs them to map a branch id → worktree path.

use std::path::PathBuf;

/// Read the current branch name at `root` via `git rev-parse`. None on any
/// failure (not a git repo, detached HEAD, git missing).
pub(crate) fn current_branch_at(root: &std::path::Path) -> Option<String> {
    let output = ship_substrate::spawn_bounded::Cmd::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() || name == "HEAD" { None } else { Some(name) }
}

/// Look up the worktree path for a branch via `git worktree list --porcelain`.
/// Returns None if the branch isn't a checked-out worktree.
pub(crate) fn worktree_path_for_branch(branch: &str) -> Option<PathBuf> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    let output = ship_substrate::spawn_bounded::Cmd::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for block in text.split("\n\n") {
        let mut path: Option<String> = None;
        let mut found_branch: Option<String> = None;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p.trim().to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                found_branch = Some(b.trim().to_string());
            }
        }
        if let (Some(p), Some(b)) = (path, found_branch)
            && b == branch
        {
            return Some(PathBuf::from(p));
        }
    }
    None
}
