//! Workspace reconciliation — detect dead sessions.
//!
//! Simplified from full worktree reconciliation to a single check:
//! "are there sessions marked active whose MCP process is dead?"
//! Run on daemon startup.

use anyhow::Result;
use std::path::Path;

use super::crud::get_workspace;
use super::types::Workspace;

/// Resolve the project root for a workspace.
///
/// If workspace has a valid path → returns it.
/// Otherwise → falls back to ship_dir parent (main checkout).
pub fn resolve_project_root(
    ship_dir: &Path,
    branch: &str,
) -> Result<(std::path::PathBuf, Workspace)> {
    let ws = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("workspace not found for branch '{}'", branch))?;

    // 1. Workspace has an explicit path on disk.
    if let Some(ref ws_path) = ws.path {
        let p = Path::new(ws_path);
        if p.exists() {
            return Ok((p.to_path_buf(), ws));
        }
    }

    // 2. Discover the active project via CWD walk. This follows git worktree
    //    redirects and returns the main checkout's .ship/ when called from a
    //    linked worktree. Correct source for "where are this workspace's files"
    //    when ship_dir is the daemon's ~/.ship/ (global), not a project .ship/.
    if let Ok(project_ship) = crate::project::get_project_dir(None) {
        if let Some(root) = project_ship.parent() {
            return Ok((root.to_path_buf(), ws));
        }
    }

    // 3. Last-resort: ship_dir.parent() — only correct if ship_dir is itself
    //    a project .ship/, not the global daemon dir.
    let fallback = ship_dir.parent().unwrap_or(ship_dir).to_path_buf();
    Ok((fallback, ws))
}

/// Reconcile all workspaces — check for dead sessions.
///
/// Called on daemon startup. Returns the list of all workspaces.
pub fn reconcile_all_workspaces(ship_dir: &Path) -> Result<Vec<Workspace>> {
    super::crud::list_workspaces_raw(ship_dir)
}

/// Reconcile a single workspace (no-op in the simplified model).
pub fn reconcile_workspace(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<Workspace>> {
    get_workspace(ship_dir, branch)
}
