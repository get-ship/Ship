//! Ungated path helpers for shipd.
//!
//! Today: a single `repo_worktrees_dir()` helper used by the
//! `AgentDispatchPort::worktrees_dir()` adapter (harness-drivers ON) and
//! the `WorktreesDirPort` adapter (always). The implementation is the
//! honest one: read `SHIP_REPO_DIR`, derive the project name from the
//! file-name, and ask [`ship_config::worktrees::worktrees_dir`] for the
//! root. This used to be duplicated in `ship_code::helpers::worktrees_dir`
//! (harness-gated). The collapse is one definition surviving — both call
//! sites route through here, eliminating the mirror.
//!
//! Naming: `repo_worktrees_dir` (returns the worktrees ROOT for the SHIP
//! repo, not any "local paths" generic concept) makes the single-purpose
//! shape explicit. Kept as `pub fn worktrees_dir()` to match the
//! long-standing call-site name (`crate::ports::local_paths::worktrees_dir`).

use std::path::PathBuf;

/// Resolve the worktrees root for the daemon's repo.
///
/// Reads `Var::RepoDir` (or falls back to CWD), derives the project name
/// from the directory's file-name, and delegates to the L1
/// [`ship_config::worktrees::worktrees_dir`] resolver — the same path the
/// dispatch shell scripts + helmd's land-handler use, so all three
/// (shipd's harness path, helmd's dispatch, the legacy shell scripts)
/// land on the same root.
pub fn worktrees_dir() -> PathBuf {
    let repo = ship_substrate::env::read_raw(ship_substrate::env::Var::RepoDir)
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let project = repo
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("ship")
        .to_string();
    ship_config::worktrees::worktrees_dir(&project)
}
