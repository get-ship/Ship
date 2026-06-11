//! Job orchestration — pipeline advancement, step dispatch, event routing.
//!
//! This module contains the business logic for job lifecycle management:
//! pipeline DAG traversal, step dispatch decisions, event emission, and
//! startup reconciliation. It delegates process management to the host
//! via the `AgentPlatform` and `ProcessRegistryPort` traits defined in
//! `dispatch_ports`.
//!
//! # Architecture
//!
//! The orchestrator is transport-agnostic. It receives `EventEnvelope`s
//! and calls back to the host through injected trait objects. This makes
//! the orchestration logic testable without a real supervisor (tmux, git
//! worktree, etc.).

// Runner MECHANISM: emit (kernel-emit primitives) stays in L0 ship-runner;
// dedup/exit_handler/router/observe/spawn_env moved to L1 ship_code::runner.
// Re-exported at the historic `orchestrator::{dedup, emit, exit_handler}` paths
// so the L1 policy below and shipd resolve them unchanged.
pub use ship_runner::emit;
pub use ship_code::runner::{dedup, exit_handler};
pub mod coordinator;
pub mod coordinator_programs;
pub mod isolation;
pub mod fanout;
pub mod fanout_aggregate;
pub mod fanout_cancel;
pub mod fanout_driver;
pub mod fanout_recover;
pub mod git_helpers;
pub mod review_retry;
pub mod gate_rules;
pub mod handlers;
pub mod pipeline;
pub mod pipeline_loader;
#[cfg(test)]
mod pipeline_loader_tests;
pub mod pending;
pub mod pipeline_specs;
pub mod projection_reader;
pub mod prompt_composer;
pub mod replay;
pub mod spawn_policy;
pub mod router;
pub mod run_step;
pub mod step_executor;
pub mod step_io;
pub mod step_monitor;
pub mod sweep;

pub mod dispatch;

#[cfg(test)]
mod tests_fakes;
#[cfg(test)]
mod tests_coordinator;
#[cfg(test)]
mod tests_cancel;
#[cfg(test)]
mod tests_prompt_composer;
#[cfg(test)]
mod tests_dispatch_harness;
#[cfg(test)]
mod tests_run_step_journal;
#[cfg(test)]
mod tests_agent_step_journal;
#[cfg(test)]
mod tests_spawn_failed;
#[cfg(test)]
mod tests_replay;
#[cfg(test)]
mod tests_sweep;
#[cfg(test)]
mod tests_terminal_guard;
#[cfg(test)]
mod tests_heartbeat;
#[cfg(test)]
mod tests_step_io;
#[cfg(test)]
mod tests_pipeline_load;
#[cfg(test)]
mod tests_fanout;
#[cfg(test)]
mod tests_fanout_journal;
#[cfg(test)]
mod tests_fanout_harness;
#[cfg(test)]
mod tests_fanout_integration;
#[cfg(test)]
mod tests_fanout_cancel;
#[cfg(test)]
mod tests_fanout_recover;
#[cfg(test)]
mod tests_merge_step;
#[cfg(test)]
mod tests_review_retry;
#[cfg(test)]
mod tests_runner_ports;

use durable_execution::JournalStore;
use ship_code::dispatch_ports::{
    AgentPlatform, IsolationStrategy, ProcessRegistryPort, ProcessTreeProbe, ProjectionReader,
    SessionExecutor, SpawnPolicy,
};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Activity signal sent to a step monitor from external sources (hooks, API).
#[derive(Debug, Clone)]
pub enum StepActivity {
    /// A harness Stop hook fired — the agent finished a response turn.
    /// In interactive mode this does NOT mean the task is done.
    TurnEnded,
}

// `LaneOutcome` moved to `ship-pipelines::fanout` per modular-runtime-01.
// Re-exported here so existing call sites compile unchanged.
pub use ship_pipelines::fanout::LaneOutcome;

/// Reference that links a lane slug back to its parent fanout step.
#[derive(Debug, Clone)]
pub struct FanoutLaneRef {
    pub pipeline_run_id: String,
    pub parent_slug: String,
    pub fanout_step_id: String,
    pub fanout_step_index: usize,
    pub lane_id: String,
}

/// Bundles the injected strategies for job orchestration.
///
/// Created by the host process (e.g. shipd) and passed to the orchestrator
/// router. All fields are trait objects so the orchestration logic is
/// decoupled from the concrete supervisor implementation.
///
/// The fields divide into two groups that the agents-arc keystone split
/// names explicitly (see `dispatch_ports::RunnerPorts` / `RunnerDomainState`):
///   - **core ports** the runner mechanism needs (`isolation`, `executor`,
///     `platform`, `registry`, `probe`, `spawn_policy`, `projections`,
///     `event_dedup`, `run_locks`) — the movable half;
///   - **per-domain state** kept by the L1 caller (`done_flags`,
///     `activity_txs`, `fanout_lane_txs`, `fanout_lane_refs`).
/// Held as one struct this slice (no crate move); the next slice relocates the
/// runner against `RunnerPorts` alone.
#[derive(Clone)]
pub struct OrchestratorContext {
    pub isolation: Arc<dyn IsolationStrategy>,
    pub executor: Arc<dyn SessionExecutor>,
    pub platform: Arc<dyn AgentPlatform>,
    pub registry: Arc<dyn ProcessRegistryPort>,
    /// Subprocess-tree heartbeat probe used by `StepMonitor` to distinguish
    /// "agent is burning CPU on a tool call" from "agent is stuck."
    pub probe: Arc<dyn ProcessTreeProbe>,
    /// Spawn/op-invocation policy. The runner hands a domain-free `Authority`
    /// principal; this resolves the scoped spawn env (role + token mint).
    pub spawn_policy: Arc<dyn SpawnPolicy>,
    /// Projection reads in primitive shapes (orphan-slug union, terminal-run
    /// guard) so the runner doesn't reach into `crate::projections` directly.
    pub projections: Arc<dyn ProjectionReader>,
    /// Maps `slug` → done_declared flag for active step monitors.
    pub done_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    /// Maps `slug` → activity sender for active step monitors.
    /// External signals (Stop hook, etc.) send here instead of emitting job.completed.
    pub activity_txs: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<StepActivity>>>>,
    /// Per-job advancement lock. Prevents two concurrent events from
    /// double-dispatching the same next step.
    pub run_locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
    /// Event-id dedup at the router entry point. Prevents a replayed
    /// envelope (mailbox redelivery, startup replay race, test replay)
    /// from advancing the pipeline twice.
    pub event_dedup: Arc<dedup::EventDedup>,
    /// Maps lane_slug → oneshot sender. When a lane's step.observed fires,
    /// the router sends the lane outcome here instead of calling advance_pipeline.
    pub fanout_lane_txs: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<LaneOutcome>>>>,
    /// Maps lane_slug → FanoutLaneRef so the router can identify fanout lanes.
    pub fanout_lane_refs: Arc<Mutex<HashMap<String, FanoutLaneRef>>>,
    /// Durable-execution journal store for pipeline-run dispatch journaling.
    /// Wired to SqliteJournalStore in production (shipd); InMemoryJournalStore in tests.
    /// Used by C2+ to save post-hoc shadow journals; read by C3+ reconcile for restart confirmation.
    pub journal_store: Arc<dyn JournalStore>,
}
