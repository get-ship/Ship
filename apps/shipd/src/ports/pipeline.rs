//! Shipd implementations of the surviving `ship-pipeline-runtime` port
//! trait.
//!
//! The orchestrator-era `PipelineDispatchPort` and `TurnSignalPort` are
//! gone (2026-07-10 engine cut). The only remaining port from the
//! pipeline runtime is [`WorktreesDirPort`] — used by the live agent-log
//! tail (`log_stream::resolve_log_path`).
//!
//! Follows the same pattern as `agent_event_ports.rs` for the agents
//! runtime.

use std::path::{Path, PathBuf};

use ship_pipeline_runtime::pipeline_ports::WorktreesDirPort;

/// Adapter for [`WorktreesDirPort`] backed by shipd's local paths.
pub struct WorktreesDirAdapter;

impl WorktreesDirPort for WorktreesDirAdapter {
    fn worktrees_dir(&self) -> PathBuf {
        crate::ports::local_paths::worktrees_dir()
    }
}

// `Path` is referenced by the trait method receivers; keep the import
// live so the file's `use` block mirrors the other ports modules.
#[allow(dead_code)]
fn _path_anchor(_: &Path) {}
