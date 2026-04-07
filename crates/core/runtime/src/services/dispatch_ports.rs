//! Port types for job dispatch — isolation and execution abstractions.
//!
//! These types are always available (not gated behind `unstable`) because
//! the daemon's job dispatch orchestration depends on them for trait injection.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// Context for a job being dispatched. Carries everything an executor needs.
#[derive(Clone, Debug)]
pub struct JobContext {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    pub work_dir: PathBuf,
    pub env: HashMap<String, String>,
    /// How to launch the provider CLI for this job.
    pub launch: LaunchProfile,
}

/// Everything about *how* to run the provider process.
///
/// Separated from JobContext so launch configuration is composable —
/// provider defaults + pipeline step overrides + user preferences.
#[derive(Clone, Debug, Default)]
pub struct LaunchProfile {
    /// Provider CLI to use (e.g. "claude", "codex"). None = default (claude).
    pub provider: Option<String>,
    /// Model override for this run.
    pub model: Option<String>,
    /// Effort level (e.g. "low", "medium", "high", "max"). Provider-specific.
    pub effort: Option<String>,
    /// Resume a previous session by ID.
    pub resume_session: Option<String>,
    /// Display name for the session (shows in provider's session picker).
    pub session_name: Option<String>,
    /// Maximum spend in USD for this run.
    pub max_budget_usd: Option<f64>,
    /// Permission mode override (e.g. "plan", "auto", "bypassPermissions").
    pub permission_mode: Option<String>,
    /// Whether to persist the session (false = ephemeral, e.g. gate reviews).
    pub persist_session: bool,
    /// Request structured JSON output from the provider.
    pub structured_output: bool,
}

/// Handle returned by a `JobExecutor` after spawning an agent.
pub struct ExecutorHandle {
    pub pid: Option<u32>,
    /// Opaque adapter-specific state.
    pub inner: Box<dyn std::any::Any + Send + Sync>,
}

/// CLI command built by a `ProviderLauncher`.
#[derive(Debug, Clone)]
pub struct LaunchCommand {
    /// Binary name (e.g. "claude", "codex").
    pub bin: String,
    /// CLI arguments.
    pub args: Vec<String>,
    /// Extra environment variables to set.
    pub env: HashMap<String, String>,
}

impl LaunchCommand {
    /// Flatten to a single shell command string (for tmux send-keys).
    pub fn to_shell_string(&self) -> String {
        let mut parts = Vec::new();
        for (k, v) in &self.env {
            parts.push(format!("{k}={v}"));
        }
        parts.push(self.bin.clone());
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }
}

/// Builds provider-specific CLI launch commands.
///
/// Each provider (Claude, Codex, etc.) implements this to translate
/// a `LaunchProfile` into the right binary + flags for that provider's CLI.
pub trait ProviderLauncher: Send + Sync {
    /// Build the launch command for an interactive session (tmux).
    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand;

    /// Build the launch command for a non-interactive (headless) run.
    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand;
}

/// Port: prepares and cleans up the isolated working directory for a job.
#[async_trait]
pub trait IsolationStrategy: Send + Sync {
    /// Prepare an isolated working directory. Returns the path.
    async fn prepare(&self, job: &JobContext) -> Result<PathBuf>;
    /// Clean up after job completion or failure.
    async fn cleanup(&self, job: &JobContext) -> Result<()>;
}

/// Port: spawns and manages the agent process lifecycle.
#[async_trait]
pub trait JobExecutor: Send + Sync {
    /// Spawn the agent process in the prepared work directory.
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle>;
    /// Check if the agent process is still running.
    async fn is_alive(&self, handle: &ExecutorHandle) -> bool;
    /// Terminate the agent process.
    async fn stop(&self, handle: &ExecutorHandle) -> Result<()>;
    /// Inject a message into the running agent.
    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()>;
}
