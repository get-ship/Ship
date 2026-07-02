//! Worktree-directory resolution for dev tooling (spawn-agent / daemon cleanup).
//!
//! `worktrees.dir` is dev-infra, not a user setting: it resolves from the
//! `SHIP_WORKTREE_DIR` env override, else a per-project default. The dispatch
//! shell scripts resolve it the same way.

use std::path::PathBuf;

/// Expand a leading `~/` to `$HOME`. Leaves other paths untouched.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

/// Resolve the base directory for git worktrees: the `SHIP_WORKTREE_DIR` env
/// override, else `~/dev/<project_name>-worktrees/`.
pub fn worktrees_dir(project_name: &str) -> PathBuf {
    if let Ok(dir) = ship_substrate::env::read_raw(ship_substrate::env::Var::WorktreeDir)
        && !dir.is_empty()
    {
        return expand_tilde(&dir);
    }
    let home = home::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join("dev").join(format!("{project_name}-worktrees"))
}
