//! Ungated path helpers for shipd.
//!
//! Provides `worktrees_dir()` without depending on `ship-code` so callers
//! (`agent_event_ports`, `pipeline_ports`) compile with harness-drivers OFF.
//!
//! This replicates the logic of `ship_code::helpers::worktrees_dir()`:
//!   1. Read `SHIP_REPO_DIR` env var; fall back to CWD.
//!   2. Take the directory's file-name as the project name.
//!   3. Call `ship_config::local::worktrees_dir(project_name)`.
//!
//! Behaviour is identical to the ship_code helper since both delegate to the
//! same `ship_config::local::worktrees_dir` call.

use std::path::PathBuf;

/// Absolute path to the per-project worktrees root.
///
/// Derived from the daemon's repo root (via `SHIP_REPO_DIR` or CWD).
/// Matches the path produced by `ship_code::helpers::worktrees_dir()`.
pub fn worktrees_dir() -> PathBuf {
    let repo = ship_substrate::env::read_raw(ship_substrate::env::Var::RepoDir)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
    let project = repo
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("ship")
        .to_string();
    ship_config::local::worktrees_dir(&project)
}
