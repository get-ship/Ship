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
impl AgentPorts {
    /// Construct a flag-off `AgentPorts` carrying ONLY the floki-harness
    /// dispatch closure. The harness-driver fields (isolation / executor /
    /// platform / registry / spawn_policy / projections / event_dedup) are
    /// absent in this build (`#[cfg(feature = "harness-drivers")]`-gated);
    /// callers that need them must use the harness-on construction site
    /// (e.g. shipd's `dispatch::run_dispatch::subscribe_job_events`).
    ///
    /// Wiring the agent-sessions actor with this constructor is what keeps
    /// floki routable when shipd is built `--no-default-features`. The actor
    /// is alive, `agent.session.started` is delivered, and the floki path in
    /// [`crate::dispatch::handle_agent_session_started`] dispatches to the
    /// closure exactly as in the harness-on build.
    ///
    /// Available only when `harness-drivers` is OFF: with the feature on,
    /// the struct literal needs every harness field set, and the canonical
    /// composition root (`dispatch::run_dispatch::subscribe_job_events`)
    /// owns that wiring.
    #[cfg(not(feature = "harness-drivers"))]
    pub fn floki_only(floki_run_fn: Option<FlokiRunFn>) -> Self {
        Self { floki_run_fn }
    }
}

#[cfg(all(test, not(feature = "harness-drivers")))]
mod tests {
    //! Tests for the flag-off `AgentPorts` shape. With `harness-drivers` off,
    //! only the floki path is reachable; these guard the constructor and the
    //! ungated dispatch seam. (The full floki→executor-skip routing test
    //! lives in `crates/core/ship-platform/crates/ship-agents/tests/behavior.rs`
    //! under `cfg(feature = "harness-drivers")` — it asserts the harness path
    //! is skipped, which the floki branch in `handle_agent_session_started`
    //! does regardless of feature state.)
    use super::*;
    use ship_substrate::events::agent_session::AgentSessionStartedPayload;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// SPEC: when the ship-code feature is off, the composition root needs a
    /// way to construct `AgentPorts` carrying only the floki-harness closure.
    /// `floki_only` is that constructor. The struct is otherwise empty (no
    /// harness fields to set), but the closure must round-trip through the
    /// port bundle untouched.
    #[test]
    fn floki_only_round_trips_closure() {
        let called = Arc::new(AtomicBool::new(false));
        let called_inner = called.clone();
        let run_fn: FlokiRunFn = Arc::new(move |_payload: AgentSessionStartedPayload| {
            let called = called_inner.clone();
            Box::pin(async move {
                called.store(true, Ordering::SeqCst);
            })
        });

        let ports = AgentPorts::floki_only(Some(run_fn));
        assert!(
            ports.floki_run_fn.is_some(),
            "floki_only must preserve the closure",
        );
    }

    /// SPEC: `floki_only(None)` produces a valid port bundle with NO floki
    /// closure wired. With the harness stack absent, the agent-sessions
    /// actor becomes a no-op for floki payloads (the dispatched event is
    /// persisted, but no run fires). This is the documented "feature-off
    /// floki no-op" mode that callers opting out of both feature and
    /// floki wiring get.
    #[test]
    fn floki_only_none_is_noop() {
        let ports = AgentPorts::floki_only(None);
        assert!(
            ports.floki_run_fn.is_none(),
            "floki_only(None) must produce a no-op port bundle",
        );
    }
}
