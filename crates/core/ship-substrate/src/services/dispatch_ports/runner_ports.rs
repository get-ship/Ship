//! Domain-free runner ports (agents-arc keystone).
//!
//! The orchestrator-runner historically reached UP into L1 domain modules for
//! three concerns: spawn/op-invocation policy (`config` + agent role + token
//! mint), projection reads (`crate::projections`), and a `Context` that mixed
//! core ports with per-domain bookkeeping. These three ports sever those
//! dependencies so the runner mechanism compiles against substrate alone.
//!
//! EVERY signature below uses only substrate / primitive types — `String`, ids,
//! [`Authority`], `HashMap` — and never an `AgentConfig` / `PipelineStep` /
//! `ProjectConfig` / agent-or-pipeline domain struct. That is the whole point:
//! a domain type that wants to cross is redesigned to pass a primitive.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

use super::{AgentPlatform, IsolationStrategy, ProcessRegistryPort, ProcessTreeProbe, SessionExecutor};

pub use ship_types::Authority;

/// Port: spawn/op-invocation policy.
///
/// Shaped as *invoke-an-op-with-a-principal*, not "spawn an agent": the
/// caller hands a domain-free [`Authority`] principal and the target actor id,
/// and the impl resolves whatever scoped environment that principal needs to
/// authenticate the launched process (e.g. `SHIP_ACTOR_ID`, a scoped
/// `SHIP_TOKEN`, `SHIP_DAEMON_URL`). A human caller, a future pipeline-token,
/// and an agent all flow through the same `Authority` seam.
///
/// The agent-specific role/token resolution (config lookup, scope mapping,
/// daemon session mint) lives entirely inside the impl — the runner never
/// names config / auth / agent types, only the principal it acts as.
#[async_trait]
pub trait SpawnPolicy: Send + Sync {
    /// Resolve the environment to inject into a process spawned on behalf of
    /// `principal`, targeting `actor_id`. The returned map is merged verbatim
    /// into the launched process env — keys/values are opaque to the runner.
    async fn spawn_env(&self, principal: &Authority, actor_id: &str) -> HashMap<String, String>;
}

/// Port: projection reads, returned in primitive / substrate shapes.
///
/// Pre-loaded state handed to the runner rather than the runner reaching into
/// domain projections. Every method returns a primitive (`Vec<String>`,
/// `bool`) so no projection/domain struct (`PipelineRun`, `AgentSession`, …)
/// crosses the seam.
#[async_trait]
pub trait ProjectionReader: Send + Sync {
    /// Every slug owned by a known intentional dispatch — across pipeline
    /// runs, bare agent sessions, and live actors (fanout lanes, chat
    /// actors, shell sessions). The coherence sweep treats a registered
    /// slug absent from this set as a true orphan. Returned as a flat slug
    /// list: the union is computed inside the impl, against the domain
    /// projections, so the runner sees only `String`s.
    async fn known_dispatch_slugs(&self) -> Vec<String>;

    /// True iff the run identified by `entity_id` has reached a terminal
    /// status (failed / merged / cancelled). The terminal-state guard uses
    /// this to drop late pipeline events for dead runs. Returns `false` when
    /// the run is unknown or non-terminal — callers treat that as "proceed".
    async fn is_run_terminal(&self, entity_id: &str) -> bool;
}

/// Core ports the runner mechanism needs to sequence and dispatch steps.
///
/// This is the *movable* half of the historic `OrchestratorContext`: once the
/// runner is relocated to L0 (a later slice) it compiles against exactly this
/// bundle — substrate ports only, no L1 per-domain state. Composed by the L1
/// caller, which keeps [`RunnerDomainState`] beside it.
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

/// Per-domain bookkeeping kept by the L1 caller, NOT by the runner mechanism.
///
/// The maps here are L1 concerns (step monitors, fanout lane wiring) that the
/// relocated runner does not depend on. Named as a struct so the split is
/// explicit and the next slice can move [`RunnerPorts`] out from under it.
/// Generic over the L1 activity / lane-outcome / lane-ref types so substrate
/// carries no domain shapes — the L1 caller pins the concrete types.
pub struct RunnerDomainState<Activity, LaneOutcome, LaneRef> {
    /// `slug` → done_declared flag for active step monitors.
    pub done_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    /// `slug` → activity sender for active step monitors.
    pub activity_txs: Arc<Mutex<HashMap<String, UnboundedSender<Activity>>>>,
    /// `lane_slug` → oneshot sender for fanout lane outcomes.
    pub fanout_lane_txs: Arc<Mutex<HashMap<String, oneshot::Sender<LaneOutcome>>>>,
    /// `lane_slug` → fanout lane reference.
    pub fanout_lane_refs: Arc<Mutex<HashMap<String, LaneRef>>>,
}
