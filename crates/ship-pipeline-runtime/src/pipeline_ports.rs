//! Port traits for pipeline domain operations that require daemon-only state.
//!
//! The pipeline domain declares these traits; shipd implements them in
//! `pipeline_ports.rs` (a sibling to `agent_event_ports.rs`).
//! This follows the same port/adapter pattern established for ship-agents.

use async_trait::async_trait;
use ship_substrate::events::EventEnvelope;
use std::path::{Path, PathBuf};

/// Provides persistence + event routing and project-level paths.
/// Shipd implements this via `ApiState`.
#[async_trait]
pub trait PipelineDispatchPort: Send + Sync {
    /// Persist an event envelope and route it through the kernel.
    async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()>;

    /// Path to the active `.ship/` directory (used for projection reads).
    fn ship_dir(&self) -> &Path;
}

/// Signals the step monitor for a running pipeline job.
/// Shipd implements this by looking up the active dispatch handle's
/// `activity_txs` map and sending `StepActivity::TurnEnded`.
#[async_trait]
pub trait TurnSignalPort: Send + Sync {
    /// Attempt to send a TurnEnded signal to the step monitor for `slug`.
    /// Returns `true` if the signal was delivered (monitor is active),
    /// `false` if no monitor exists for this slug (caller should fall through
    /// to emitting `pipeline.run.completed` directly).
    async fn signal_turn_ended(&self, slug: &str) -> bool;
}

/// Resolves the worktrees directory path.
/// Shipd implements this via `ship_supervisor::helpers::worktrees_dir()`.
pub trait WorktreesDirPort: Send + Sync {
    fn worktrees_dir(&self) -> PathBuf;
}
