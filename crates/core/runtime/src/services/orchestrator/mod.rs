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

pub mod dedup;
pub mod emit;
pub mod fanout;
pub mod fanout_aggregate;
pub mod git_helpers;
pub mod review_retry;
pub mod gate_rules;
pub mod handlers;
pub mod pipeline;
pub mod pipeline_loader;
#[cfg(test)]
mod pipeline_loader_tests;
pub mod pipeline_vars;
pub mod pending;
pub mod pipeline_specs;
pub mod prompt_composer;
pub mod replay;
pub mod router;
pub mod run_step;
pub mod step_executor;
pub mod step_io;
pub mod step_monitor;
pub mod sweep;

pub mod dispatch;

#[cfg(test)]
mod tests_cancel;
#[cfg(test)]
mod tests_prompt_composer;
#[cfg(test)]
mod tests_dispatch_provider;
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
mod tests_yaml_load;
#[cfg(test)]
mod tests_fanout;
#[cfg(test)]
mod tests_fanout_harness;
#[cfg(test)]
mod tests_fanout_integration;

use crate::services::dispatch_ports::{
    AgentPlatform, IsolationStrategy, JobExecutor, ProcessRegistryPort, ProcessTreeProbe,
};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Activity signal sent to a step monitor from external sources (hooks, API).
#[derive(Debug, Clone)]
pub enum StepActivity {
    /// A provider Stop hook fired — the agent finished a response turn.
    /// In interactive mode this does NOT mean the task is done.
    TurnEnded,
}

/// Outcome from a single fanout lane after its step monitor fires.
#[derive(Debug, Clone)]
pub struct LaneOutcome {
    pub lane_id: String,
    pub success: bool,
    pub outputs: std::collections::BTreeMap<String, String>,
}

/// Reference that links a lane slug back to its parent fanout step.
#[derive(Debug, Clone)]
pub struct FanoutLaneRef {
    pub job_id: String,
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
#[derive(Clone)]
pub struct OrchestratorContext {
    pub isolation: Arc<dyn IsolationStrategy>,
    pub executor: Arc<dyn JobExecutor>,
    pub platform: Arc<dyn AgentPlatform>,
    pub registry: Arc<dyn ProcessRegistryPort>,
    /// Subprocess-tree heartbeat probe used by `StepMonitor` to distinguish
    /// "agent is burning CPU on a tool call" from "agent is stuck."
    pub probe: Arc<dyn ProcessTreeProbe>,
    /// Maps `slug` → done_declared flag for active step monitors.
    pub done_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    /// Maps `slug` → activity sender for active step monitors.
    /// External signals (Stop hook, etc.) send here instead of emitting job.completed.
    pub activity_txs: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<StepActivity>>>>,
    /// Per-job advancement lock. Prevents two concurrent events from
    /// double-dispatching the same next step.
    pub job_locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
    /// Event-id dedup at the router entry point. Prevents a replayed
    /// envelope (mailbox redelivery, startup replay race, test replay)
    /// from advancing the pipeline twice.
    pub event_dedup: Arc<dedup::EventDedup>,
    /// Maps lane_slug → oneshot sender. When a lane's step.observed fires,
    /// the router sends the lane outcome here instead of calling advance_pipeline.
    pub fanout_lane_txs: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<LaneOutcome>>>>,
    /// Maps lane_slug → FanoutLaneRef so the router can identify fanout lanes.
    pub fanout_lane_refs: Arc<Mutex<HashMap<String, FanoutLaneRef>>>,
}
