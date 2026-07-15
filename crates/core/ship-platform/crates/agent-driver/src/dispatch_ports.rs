//! Minimal dispatch-port value types kept in L0 ship-substrate.
//!
//! Only `LaunchCommand` (+ its `to_shell_string` impl) and `LaunchProfile`
//! remain here. They must stay in L0 because the L0 dispatch emit
//! helpers (in `ship_substrate::events::emit`) import them, and that L0
//! must not depend on the L1 `ship-code` crate.
//!
//! All port traits (`SessionExecutor`, `IsolationStrategy`, etc.), `SessionContext`,
//! `ExitSignal`, `ExecutorHandle`, and the runner-ports types have moved to
//! `ship_code::dispatch_ports`.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;

pub use ship_types::Authority;


/// CLI command built by a `HarnessLauncher`.
#[derive(Debug, Clone)]
pub struct LaunchCommand {
    /// Binary name (e.g. "claude", "codex").
    pub bin: String,
    /// CLI arguments.
    pub args: Vec<String>,
    /// Extra environment variables to set.
    pub env: HashMap<String, String>,
}

// `LaunchCommand::to_shell_string` + the `shell_quote` helper live in the
// `shell_quote` submodule (size-cap split). The type stays here as a port type.
mod shell_quote;

/// Everything about *how* to run the harness process.
#[derive(Clone, Debug, Default)]
pub struct LaunchProfile {
    pub harness: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub resume_session: Option<String>,
    pub session_name: Option<String>,
    pub max_budget_usd: Option<f64>,
    pub permission_mode: Option<String>,
    pub persist_session: bool,
    pub structured_output: bool,
    pub autostart_prompt: Option<String>,
    pub append_system_prompt: Option<String>,
}

/// Port: platform-specific agent operations.
#[async_trait]
pub trait AgentPlatform: Send + Sync {
    fn worktrees_dir(&self) -> Result<PathBuf>;
    fn compile_agent_config(&self, worktree_path: &std::path::Path, agent_id: &str) -> Result<()>;
    async fn graceful_agent_exit(&self, slug: &str, harness: Option<&str>);
    async fn cleanup_job(&self, slug: &str, branch: &str);
    fn git_merge(&self, branch: &str) -> Result<bool>;
    async fn cleanup_orphaned_worktrees(&self);
    async fn mint_session_token(&self, _agent_id: &str) -> Option<String> {
        None
    }
    fn daemon_url(&self) -> Option<String> {
        None
    }
}

/// Port: spawn/op-invocation policy.
#[async_trait]
pub trait SpawnPolicy: Send + Sync {
    async fn spawn_env(&self, principal: &Authority, actor_id: &str) -> HashMap<String, String>;
}

/// Port: projection reads, returned in primitive / substrate shapes.
#[async_trait]
pub trait ProjectionReader: Send + Sync {
    async fn known_dispatch_slugs(&self) -> Vec<String>;
    async fn is_run_terminal(&self, entity_id: &str) -> bool;
    /// Resolve the slug for a session given its session_id. Used by the
    /// cancel handler to look up the process registry keyed by slug.
    /// Returns `None` when the session is not found in the canonical sessions
    /// projection (e.g. the session was never started or the projection is stale).
    async fn slug_for_session_id(&self, _session_id: &str) -> Option<String> {
        None
    }
}
