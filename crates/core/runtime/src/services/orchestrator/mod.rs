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

pub mod emit;
pub mod gate_retry;
pub mod gate_rules;
pub mod handlers;
pub mod pipeline;
pub mod pipeline_loader;
pub mod pipeline_vars;
pub mod pending;
pub mod pipeline_specs;
pub mod prompt_composer;
pub mod replay;
pub mod router;
pub mod run_step;
pub mod step_monitor;

pub mod dispatch;

use crate::services::dispatch_ports::{
    AgentPlatform, IsolationStrategy, JobExecutor, ProcessRegistryPort,
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
    /// Maps `slug` → done_declared flag for active step monitors.
    pub done_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    /// Maps `slug` → activity sender for active step monitors.
    /// External signals (Stop hook, etc.) send here instead of emitting job.completed.
    pub activity_txs: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<StepActivity>>>>,
    /// Per-job advancement lock. Prevents two concurrent events from
    /// double-dispatching the same next step.
    pub job_locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}
