//! The injected port bundle the `service.agents` actor drives.
//!
//! Mirrors the orchestrator's historic `OrchestratorContext`, but holds ONLY
//! the ports the bare agent-session domain needs — no pipeline-DAG bookkeeping
//! (done_flags / activity_txs / fanout maps stay with the S4 orchestrator).
//!
//! Every field is an L0 type:
//!   - the core dispatch ports come from
//!     [`ship_substrate::services::dispatch_ports`] (isolation / executor /
//!     platform / registry / spawn-policy / projection-reader);
//!   - `event_dedup` is the L0 runner's [`EventDedup`].
//!
//! The concrete impls (`ConfigSpawnPolicy`, `ProjectionsReader`, the shipd
//! executor / isolation / platform) live in L1/host code and are injected here
//! at the composition root — agents names only the ports.

use std::sync::Arc;

use ship_runner::dedup::EventDedup;
use ship_substrate::services::dispatch_ports::{
    AgentPlatform, IsolationStrategy, ProcessRegistryPort, ProjectionReader, SessionExecutor,
    SpawnPolicy,
};

/// Ports for agent-session orchestration. Cloneable (all `Arc`) so the actor
/// can hand a snapshot to each detached dispatch task.
#[derive(Clone)]
pub struct AgentPorts {
    /// Prepares + tears down the per-session isolated worktree.
    pub isolation: Arc<dyn IsolationStrategy>,
    /// Spawns / stops the harness process.
    pub executor: Arc<dyn SessionExecutor>,
    /// Host platform ops (worktrees dir, cleanup, token mint).
    pub platform: Arc<dyn AgentPlatform>,
    /// Live process-handle registry (register / take / tombstone).
    pub registry: Arc<dyn ProcessRegistryPort>,
    /// Spawn/op-invocation policy. The agent dispatch hands a domain-free
    /// [`ship_types::Authority::Agent`] principal; the impl resolves the
    /// scoped spawn env (role + token mint). The concrete impl
    /// (`ConfigSpawnPolicy`) stays in L1 because it reads project config; this
    /// crate sees only the port.
    pub spawn_policy: Arc<dyn SpawnPolicy>,
    /// Primitive projection reads for the L0 router gate's terminal-state
    /// guard. Agents never reads `crate::projections` directly.
    pub projections: Arc<dyn ProjectionReader>,
    /// Event-id dedup at the actor entry. Prevents a replayed envelope
    /// (mailbox redelivery, startup replay race, test replay) from
    /// double-admitting a spawn.
    pub event_dedup: Arc<EventDedup>,
}
