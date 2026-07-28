//! The injected port bundle the `service.agents` actor drives.
//!
//! Mirrors the orchestrator's historic `OrchestratorContext`, but holds ONLY
//! the ports the bare agent-session domain needs — no pipeline-DAG bookkeeping
//! (done_flags / activity_txs / fanout maps stay with the S4 orchestrator).
//!
//! Every field is an L0 type:
//!   - the core dispatch ports come from
//!     [`agent_driver::dispatch_ports`] (isolation / executor /
//!     platform / registry / spawn-policy / projection-reader);
//!   - `event_dedup` is the L0 runner's [`EventDedup`].
//!
//! The concrete impls (`ConfigSpawnPolicy`, `ProjectionsReader`, the shipd
//! executor / isolation / platform) live in L1/host code and are injected here
//! at the composition root — agents names only the ports.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use agent_schema::dispatch_intent::DispatchRequestedPayload;
#[cfg(feature = "ship-code")]
use ship_substrate::dispatch_gate::EventDedup;
#[cfg(feature = "ship-code")]
use ship_code::dispatch_ports::{
    AgentPlatform, IsolationStrategy, ProcessRegistryPort, ProjectionReader, SessionExecutor,
    SpawnPolicy,
};
#[cfg(feature = "ship-code")]
use ship_code::LifecycleSink;

/// Async callback for native Yoke harness dispatch.
///
/// Ungated so `AgentPorts` carries it with or without `ship-code`. The
/// host (shipd) captures gateway/kernel; `ship-agents` never imports yoke.
/// Payload type is L1 (`agent-def`).
pub type YokeRunFn = Arc<
    dyn Fn(DispatchRequestedPayload) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

/// Ports for agent-session orchestration. Cloneable (all `Arc`) so the actor
/// can hand a snapshot to each detached dispatch task.
///
/// With `ship-code` OFF, only `yoke_run_fn` is present — the entire
/// harness-driver stack (executor/isolation/registry/etc.) is absent. Callers
/// that construct `AgentPorts` must wrap every harness field in
/// `#[cfg(feature = "ship-code")]` blocks.
#[derive(Clone)]
pub struct AgentPorts {
    /// Native Yoke dispatch. When `Some` and harness is `"yoke"` or `"floki"`,
    /// routes here and skips the ship-code harness-driver stack.
    pub yoke_run_fn: Option<YokeRunFn>,
    /// Prepares + tears down the per-session isolated worktree.
    #[cfg(feature = "ship-code")]
    pub isolation: Arc<dyn IsolationStrategy>,
    /// Spawns / stops the harness process.
    #[cfg(feature = "ship-code")]
    pub executor: Arc<dyn SessionExecutor>,
    /// Host observation port — kernel emission lives here, not in harness code.
    #[cfg(feature = "ship-code")]
    pub lifecycle: Arc<dyn LifecycleSink>,
    /// Host platform ops (worktrees dir, cleanup, token mint).
    #[cfg(feature = "ship-code")]
    pub platform: Arc<dyn AgentPlatform>,
    /// Live process-handle registry (register / take / tombstone).
    #[cfg(feature = "ship-code")]
    pub registry: Arc<dyn ProcessRegistryPort>,
    /// Spawn/op-invocation policy. The agent dispatch hands a domain-free
    /// [`agent_schema::Authority::Agent`] principal; the impl resolves the
    /// scoped spawn env (role + token mint). The concrete impl
    /// (`ConfigSpawnPolicy`) stays in L1 because it reads project config; this
    /// crate sees only the port.
    #[cfg(feature = "ship-code")]
    pub spawn_policy: Arc<dyn SpawnPolicy>,
    /// Primitive projection reads for the L0 router gate's terminal-state
    /// guard + slug_for_session_id lookup for the cancel handler. Agents never
    /// reads `crate::projections` directly.
    #[cfg(feature = "ship-code")]
    pub projections: Arc<dyn ProjectionReader>,
    /// Event-id dedup at the actor entry. Prevents a replayed envelope
    /// (mailbox redelivery, startup replay race, test replay) from
    /// double-admitting a spawn.
    #[cfg(feature = "ship-code")]
    pub event_dedup: Arc<EventDedup>,
}
impl AgentPorts {
    /// Construct a flag-off `AgentPorts` carrying ONLY the yoke-harness
    /// dispatch closure. The harness-driver fields (isolation / executor /
    /// platform / registry / spawn_policy / projections / event_dedup) are
    /// absent in this build (`#[cfg(feature = "ship-code")]`-gated);
    /// callers that need them must use the harness-on construction site
    /// (e.g. shipd's `dispatch::run_dispatch::subscribe_job_events`).
    ///
    /// Keeps native yoke routable when shipd is built without ship-code.
    ///
    /// Available only when `ship-code` is OFF: with the feature on,
    /// the struct literal needs every harness field set, and the canonical
    /// composition root (`dispatch::run_dispatch::subscribe_job_events`)
    /// owns that wiring.
    #[cfg(not(feature = "ship-code"))]
    pub fn yoke_only(yoke_run_fn: Option<YokeRunFn>) -> Self {
        Self { yoke_run_fn }
    }
}

#[cfg(all(test, not(feature = "ship-code")))]
mod tests {
    //! Tests for the flag-off `AgentPorts` shape. With `ship-code` off,
    //! only the yoke path is reachable; these guard the constructor and the
    //! ungated dispatch seam.
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// SPEC: when the ship-code feature is off, the composition root needs a
    /// way to construct `AgentPorts` carrying only the yoke-harness closure,
    /// and the SAME closure must be invocable through the port bundle — the
    /// dispatch seam calls `ports.yoke_run_fn` to fire a yoke session.
    /// Asserting closure identity by INVOKING it through the seam (not just
    /// `is_some()`, which a bare struct-literal constructor makes a
    /// tautology) pins the regression a future refactor could introduce by
    /// wrapping or swapping the closure.
    #[tokio::test]
    async fn yoke_only_closure_is_invocable_through_the_port_bundle() {
        let called = Arc::new(AtomicBool::new(false));
        let called_inner = called.clone();
        let run_fn: YokeRunFn = Arc::new(move |_payload: DispatchRequestedPayload| {
            let called = called_inner.clone();
            Box::pin(async move {
                called.store(true, Ordering::SeqCst);
            })
        });

        let ports = AgentPorts::yoke_only(Some(run_fn));
        let payload: DispatchRequestedPayload = serde_json::from_value(serde_json::json!({
            "session_id": "s-1",
            "slug": "yoke-seam",
            "agent_id": "dev",
            "workspace": "w",
            "branch": "job/yoke-seam",
            "open_tab": false,
            "invocation": "print",
            "keepalive": false,
        }))
        .expect("minimal dispatch payload");

        ports
            .yoke_run_fn
            .as_ref()
            .expect("yoke_only must preserve the closure")(payload)
        .await;
        assert!(
            called.load(Ordering::SeqCst),
            "invoking the port-bundle closure must run the wired closure"
        );
    }
}
