//! Port types for job dispatch — isolation and execution abstractions.
//!
//! These types are always available (not gated behind `unstable`) because
//! the daemon's job dispatch orchestration depends on them for trait injection.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

/// Signal sent from an executor monitor when the agent process exits.
///
/// Executors send this on `JobContext::exit_tx` so the dispatch handler can
/// emit `job.completed` or `job.failed` without waiting for the SessionEnd hook.
#[derive(Debug)]
pub struct ExitSignal {
    pub slug: String,
    pub job_id: String,
    /// `None` means the process was killed or the exit code was unavailable
    /// (e.g. tmux session closed). `Some(0)` = clean exit, non-zero = failure.
    pub exit_code: Option<i32>,
}

/// Context for a job being dispatched. Carries everything an executor needs.
#[derive(Clone, Debug)]
pub struct JobContext {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    /// Inline spec content. When `Some`, the isolation strategy writes this to
    /// `{work_dir}/{spec_path}` during `prepare`. `None` = fall back to copying
    /// from the project root (legacy manual-dispatch path).
    pub spec_content: Option<String>,
    pub work_dir: PathBuf,
    pub env: HashMap<String, String>,
    /// How to launch the provider CLI for this job.
    pub launch: LaunchProfile,
    /// Exit notification channel. Set by `dispatch_job_internal` so the executor's
    /// background monitor can signal exit without curling the daemon API.
    /// `None` for pipeline re-dispatches — the liveness sweep handles those.
    pub exit_tx: Option<UnboundedSender<ExitSignal>>,
    /// Step metadata used by the PtyExecutor to pick the tmux window name and
    /// per-step log path. `None` for single-step jobs — the executor falls back
    /// to using `slug` as the window name.
    pub step: Option<StepInfo>,
    /// When true, dispatch forces the headless ProcessExecutor path even if
    /// the executor in `OrchestratorContext` is PtyExecutor. Surfaced through
    /// to the executor so it can fall back without its own flag check.
    pub headless: bool,
}

/// Per-step metadata threaded to executors.
#[derive(Clone, Debug)]
pub struct StepInfo {
    /// 0-based index within the pipeline.
    pub index: usize,
    /// Stable step identifier (e.g. "test-writer").
    pub id: String,
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

// ── Orchestrator port traits ────────────────────────────────────────────────

/// Port: platform-specific agent operations that the orchestrator delegates
/// to the host process (e.g. shipd).
///
/// These methods abstract over tmux, git worktree, and shell execution so
/// the orchestration logic in runtime is testable without a real supervisor.
#[async_trait]
pub trait AgentPlatform: Send + Sync {
    /// Base directory where job worktrees are created.
    fn worktrees_dir(&self) -> PathBuf;

    /// Compile agent configuration via `ship use <agent_id>`.
    fn compile_agent_config(&self, worktree_path: &std::path::Path, agent_id: &str) -> Result<()>;

    /// Gracefully exit the current agent in a tmux session.
    async fn graceful_agent_exit(&self, slug: &str, provider: Option<&str>);

    /// Clean up job resources (tmux session, worktree, branch).
    async fn cleanup_job(&self, slug: &str, branch: &str);

    /// Execute `git merge <branch>` in the repo directory. Returns true on success.
    fn git_merge(&self, branch: &str) -> Result<bool>;

    /// Start a single-step exit signal handler (non-pipeline jobs).
    fn start_exit_handler(
        &self,
        kernel: std::sync::Arc<tokio::sync::Mutex<crate::events::KernelRouter>>,
        exit_rx: tokio::sync::mpsc::UnboundedReceiver<ExitSignal>,
    );

    /// Start periodic liveness monitoring.
    fn start_liveness_sweep(
        &self,
        kernel: std::sync::Arc<tokio::sync::Mutex<crate::events::KernelRouter>>,
    );

    /// Clean up orphaned worktrees from previous daemon runs.
    async fn cleanup_orphaned_worktrees(&self);
}

/// Port: process handle registry for tracking spawned agents.
///
/// The orchestrator registers/unregisters handles as jobs are dispatched
/// and completed. The host process owns the actual registry data structure.
#[async_trait]
pub trait ProcessRegistryPort: Send + Sync {
    /// Store an executor handle for a dispatched job.
    async fn register(&self, slug: &str, handle: ExecutorHandle);
    /// Remove a handle (e.g. when a job completes).
    async fn unregister(&self, slug: &str);
}
