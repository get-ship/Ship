//! Port traits for pipeline-domain helpers that the runtime exposes.
//!
//! Only the always-needed surface survives here. The orchestrator-era
//! `PipelineDispatchPort` and `TurnSignalPort` were retired with the
//! engine (2026-07-10 cut); the bare agent-session wiring doesn't need
//! them. The remaining port is [`WorktreesDirPort`] — the live-log tail
//! (`log_stream::resolve_log_path`) and any future code-mode read paths
//! thread through it for the worktrees directory.
//!
//! Follows the same port/adapter pattern as the agents leaf: the runtime
//! declares the trait; shipd implements it.

use std::path::PathBuf;

/// Resolves the worktrees directory path.
///
/// Shipd implements this via `ship_agents::runtime::worktrees_dir()`.
pub trait WorktreesDirPort: Send + Sync {
    fn worktrees_dir(&self) -> PathBuf;
}
