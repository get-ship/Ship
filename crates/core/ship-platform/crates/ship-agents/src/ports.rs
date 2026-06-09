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

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use ship_substrate::events::agent_session::AgentSessionStartedPayload;
#[cfg(feature = "harness-drivers")]
use ship_code::runner::dedup::EventDedup;
#[cfg(feature = "harness-drivers")]
use ship_code::dispatch_ports::{
    AgentPlatform, IsolationStrategy, ProcessRegistryPort, ProjectionReader, SessionExecutor,
    SpawnPolicy,
};

/// Async callback type for floki-harness dispatch.
///
/// Defined here (ungated) so `AgentPorts` carries it regardless of the
/// `harness-drivers` feature. The closure captures its own floki + gateway
/// state; `ship-agents` never imports floki directly. The `AgentSessionStartedPayload`
/// type is L0 (`ship_substrate` only) — no harness crate enters this signature.
pub type FlokiRunFn = Arc<
    dyn Fn(AgentSessionStartedPayload) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

/// Ports for agent-session orchestration. Cloneable (all `Arc`) so the actor
/// can hand a snapshot to each detached dispatch task.
///
/// With `harness-drivers` OFF, only `floki_run_fn` is present — the entire
/// harness-driver stack (executor/isolation/registry/etc.) is absent. Callers
/// that construct `AgentPorts` must wrap every harness field in
/// `#[cfg(feature = "harness-drivers")]` blocks.
#[derive(Clone)]
pub struct AgentPorts {
    /// Floki-harness dispatch closure. When `Some` and `payload.harness == "floki"`,
    /// dispatch routes here and skips the harness-driver stack entirely.
    /// Not behind any cfg gate — the floki path must always compile.
    pub floki_run_fn: Option<FlokiRunFn>,
    /// Prepares + tears down the per-session isolated worktree.
    #[cfg(feature = "harness-drivers")]
    pub isolation: Arc<dyn IsolationStrategy>,
    /// Spawns / stops the harness process.
    #[cfg(feature = "harness-drivers")]
    pub executor: Arc<dyn SessionExecutor>,
    /// Host platform ops (worktrees dir, cleanup, token mint).
    #[cfg(feature = "harness-drivers")]
    pub platform: Arc<dyn AgentPlatform>,
    /// Live process-handle registry (register / take / tombstone).
    #[cfg(feature = "harness-drivers")]
    pub registry: Arc<dyn ProcessRegistryPort>,
    /// Spawn/op-invocation policy. The agent dispatch hands a domain-free
    /// [`ship_types::Authority::Agent`] principal; the impl resolves the
    /// scoped spawn env (role + token mint). The concrete impl
    /// (`ConfigSpawnPolicy`) stays in L1 because it reads project config; this
    /// crate sees only the port.
    #[cfg(feature = "harness-drivers")]
    pub spawn_policy: Arc<dyn SpawnPolicy>,
    /// Primitive projection reads for the L0 router gate's terminal-state
    /// guard. Agents never reads `crate::projections` directly.
    #[cfg(feature = "harness-drivers")]
    pub projections: Arc<dyn ProjectionReader>,
    /// Event-id dedup at the actor entry. Prevents a replayed envelope
    /// (mailbox redelivery, startup replay race, test replay) from
    /// double-admitting a spawn.
    #[cfg(feature = "harness-drivers")]
    pub event_dedup: Arc<EventDedup>,
}
