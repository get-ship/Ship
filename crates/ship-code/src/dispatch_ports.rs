//! Port types for job dispatch — moved from L0 ship-substrate to L1 ship-code.
//!
//! `LaunchCommand` and `LaunchProfile` remain in `ship-substrate` (they are used
//! by `agent_driver` (the L1 contract leaf)). Everything else that was
//! in `agent_driver::dispatch_ports` lives here.
//!
//! `LaunchCommand` and `LaunchProfile` are re-exported here so callers can use
//! `ship_code::dispatch_ports::*` for all dispatch-port types.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use std::sync::Arc;

// Re-export the harness value and shared port types from the agent-driver leaf.
pub use agent_schema::InvocationMode;
pub use agent_driver::dispatch_ports::{
    AgentPlatform, Authority, LaunchCommand, LaunchProfile, ProjectionReader, SpawnPolicy,
};
pub use agent_driver::server_session::ServerSession;

// ── Value types ────────────────────────────────────────────────────────────────

/// Why an executor is signalling exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCause {
    /// The agent process actually exited.
    AgentExited,
    /// The executor's session/window was destroyed externally.
    SessionDestroyed,
}

/// Signal sent from an executor monitor when an agent's run ends.
#[derive(Debug)]
pub struct ExitSignal {
    pub slug: String,
    pub session_id: String,
    pub exit_code: Option<i32>,
    pub cause: ExitCause,
}

/// Per-step metadata threaded to executors.
#[derive(Clone, Debug)]
pub struct StepInfo {
    pub index: usize,
    pub id: String,
}

/// Context for a job being dispatched.
#[derive(Clone, Debug)]
pub struct SessionContext {
    /// ULID. The session id is the routing key for the session's
    /// canonical `session.*` events (started/completed/failed).
    pub session_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    pub spec_content: Option<String>,
    pub work_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub launch: LaunchProfile,
    pub exit_tx: Option<UnboundedSender<ExitSignal>>,
    pub step: Option<StepInfo>,
    pub invocation: InvocationMode,
    pub keepalive: bool,
    /// Absolute path to the repo working tree the job's worktree forks
    /// from. Per-call; consumed in-call by `GitWorktreeIsolation::prepare`.
    pub repo_path: Option<PathBuf>,
    /// Ref the job's worktree branches from. Per-call; same lifetime
    /// rules as `repo_path`.
    pub base_ref: Option<String>,
}

/// Handle returned by a `SessionExecutor` after spawning an agent.
pub struct ExecutorHandle {
    pub pid: Option<u32>,
    pub inner: Box<dyn std::any::Any + Send + Sync>,
}

// ── Port traits ─────────────────────────────────────────────────────────────────

/// Port: prepares and cleans up the isolated working directory for a job.
#[async_trait]
pub trait IsolationStrategy: Send + Sync {
    async fn prepare(&self, job: &SessionContext) -> Result<PathBuf>;
    async fn cleanup(&self, job: &SessionContext) -> Result<()>;
}

/// Port: spawns and manages the agent process lifecycle.
#[async_trait]
pub trait SessionExecutor: Send + Sync {
    async fn spawn(&self, ctx: &SessionContext) -> Result<ExecutorHandle>;
    async fn is_alive(&self, handle: &ExecutorHandle) -> bool;
    async fn stop(&self, handle: &ExecutorHandle) -> Result<()>;
    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()>;
    async fn resume_handle(
        &self,
        _slug: &str,
        _harness: Option<&str>,
        _current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        None
    }
}


/// Port: process handle registry for tracking spawned agents.
#[async_trait]
pub trait ProcessRegistryPort: Send + Sync {
    async fn register(&self, slug: &str, handle: ExecutorHandle);
    async fn unregister(&self, slug: &str);
    async fn take(&self, _slug: &str) -> Option<ExecutorHandle> { None }
    async fn live_slugs(&self) -> Vec<String> { vec![] }
    async fn get_pid(&self, _slug: &str) -> Option<u32> { None }
    async fn drain_dead(&self, _executor: &dyn SessionExecutor) -> Vec<String> { vec![] }
    async fn mark_cancelled(&self, _slug: &str) {}
    async fn take_if_cancelled(&self, _slug: &str) -> bool { false }
}

/// Sample of the liveness state of a dispatched process tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSample {
    pub cpu_ticks: u64,
    pub process_count: u32,
}

/// Port: observes liveness of a dispatched process tree.
#[async_trait]
pub trait ProcessTreeProbe: Send + Sync {
    async fn sample(&self, root_pid: u32) -> Option<TreeSample>;
}

// ── Runner ports ─────────────────────────────────────────────────────────────


/// Core ports the runner mechanism needs to sequence and dispatch steps.
#[derive(Clone)]
pub struct RunnerPorts {
    pub isolation: Arc<dyn IsolationStrategy>,
    pub executor: Arc<dyn SessionExecutor>,
    pub platform: Arc<dyn AgentPlatform>,
    pub registry: Arc<dyn ProcessRegistryPort>,
    pub probe: Arc<dyn ProcessTreeProbe>,
    pub spawn_policy: Arc<dyn SpawnPolicy>,
    pub projections: Arc<dyn ProjectionReader>,
}

use std::sync::atomic::AtomicBool;
use tokio::sync::oneshot;

/// Per-domain bookkeeping kept by the L1 caller.
pub struct RunnerDomainState<Activity, LaneOutcome, LaneRef> {
    pub done_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    pub activity_txs: Arc<Mutex<HashMap<String, UnboundedSender<Activity>>>>,
    pub fanout_lane_txs: Arc<Mutex<HashMap<String, oneshot::Sender<LaneOutcome>>>>,
    pub fanout_lane_refs: Arc<Mutex<HashMap<String, LaneRef>>>,
}
