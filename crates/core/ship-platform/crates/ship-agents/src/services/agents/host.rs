//! `AgentsHost` — the shipd-side wiring point for `AgentsWire`.
//!
//! `AgentsWire` is the crate-owned ConnectRPC impl of `AgentsService`. To
//! mount it, shipd historically built six async closures + four port adapters
//! from its `ApiState`. The trait + `mount` fn here let ship-agents own the
//! shape of that wiring; shipd just implements `AgentsHost` and calls
//! `mount(state, gate, router)`.
//!
//! The concrete port adapters (`AgentDispatchAdapter`, `FeedbackAuthAdapter`,
//! `ViewBroadcastAdapter`, `CollectionPersistAdapter`) stay in shipd — they
//! close over shipd's `AuthContext`, kernel, and collection service. The
//! abstract port trait objects (`FeedbackAuthPort`, `AgentDispatchPort`,
//! `ViewBroadcastPort`, `CollectionPersistPort`) are re-exposed here as the
//! `Host`-returning types so the host bundle reads as a single surface.
//!
//! No behavior change: every closure + port is identical to the prior
//! `apps/shipd/src/rpc/ops/agents_wire.rs` body — only the location moved.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Router as ConnectRouter};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::services::agents::annotation::CollectionPersistPort;
use crate::services::agents::dispatch_core::AgentDispatchPort;
use crate::services::agents::feedback::ViewBroadcastPort;
use crate::services::agents::rpc_wire::{
    AgentsWire, AgentsWireConfig, FeedbackAuthPort, GateFn, GrantFn, NotifySavedFn, SessionsFn,
    SteerFn, StopFn,
};
use crate::services::agents::service::AgentsService;
use ship_reactive::ReactiveRegistry;
use ship_substrate::events::KernelRouter;
use ship_substrate::ports::KernelLog;

/// Future returned by host async methods.
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Shipd-derived handle to the dispatch process registry. The shipd impl
/// (under `#[cfg(feature = "ship-code")]`) wraps the daemon's live
/// `DispatchHandle`; the trait object lets the stop / steer closures in this
/// crate act on the registry without importing shipd types.
pub trait DispatchHandlePort: Send + Sync {
    /// Stop the running agent process for `slug`.
    fn stop(&self, slug: &str) -> BoxFuture<'_, Result<Value, ConnectError>>;
    /// Send `message` to the running agent process for `slug`.
    fn steer(&self, slug: &str, message: &str) -> BoxFuture<'_, Result<Value, ConnectError>>;
}

/// Shipd-side accessor surface used by `mount`.
///
/// `mount` reads these accessors to build the six async closures + four port
/// adapters that `AgentsWire::new(AgentsWireConfig { … })` consumes. The trait
/// is `Clone + Send + Sync + 'static` because every closure and adapter is
/// `Arc`-cloned across the wire.
pub trait AgentsHost: Clone + Send + Sync + 'static {
    /// The agent domain service — `AgentsWire::svc`.
    fn agents_arc(&self) -> Arc<dyn AgentsService>;

    /// The user-scope ship directory — `AgentsWire::ship_dir`.
    fn ship_dir(&self) -> PathBuf;

    /// The shared kernel router the Live-Watch `KernelLog` wraps.
    /// `AgentsWire::kernel` is a fresh `KernelLog::new(...)` over this arc.
    fn kernel_router(&self) -> Arc<Mutex<KernelRouter>>;

    /// Attached project workspace (for the project-scope notifier branch).
    fn workspace_dir(&self) -> Option<PathBuf>;

    /// The shared `agent_def_kernel` (`OnceLock`-mounted) when the agent
    /// reactive registry is up. `None` for a space-less boot.
    fn agent_def_kernel(&self) -> Option<Arc<KernelLog>>;

    /// The agent `Watch` reactive registry — `None` when space-less.
    fn agent_reactive(&self) -> Option<Arc<ReactiveRegistry>>;

    /// Per-process session registry — `agent.grant` mutates grants here.
    fn session_registry(&self) -> ship_auth::scopes::SessionRegistry;

    /// The dispatch port — agent dispatch writes events through this.
    fn dispatch_port(&self) -> Arc<dyn AgentDispatchPort>;

    /// Feedback auth port — extracts the emit context + agent id from the
    /// request context.
    fn feedback_auth(&self) -> Arc<dyn FeedbackAuthPort>;

    /// View broadcast port — feedback / annotation live broadcast.
    fn view_broadcast(&self) -> Arc<dyn ViewBroadcastPort>;

    /// Collection persist port — annotation writes land here.
    fn collection(&self) -> Arc<dyn CollectionPersistPort>;

    /// Extract the verified approver (`agent_id`) from the auth context for
    /// `agent.grant`. Returns `Err(unauthenticated)` when the auth extension
    /// is missing; `Ok(None)` for anonymous / daemon-token requests.
    fn extract_approver(&self, ctx: &RequestContext) -> Result<Option<String>, ConnectError>;

    /// Dispatch handle — `Some` only when ship-code built the process
    /// registry. When `None`, stop / steer return `"dispatch not available"`.
    /// Ship-agents always treats this as `None` on non-`harness-drivers`
    /// builds; shipd swaps in the live registry when ship-code is on.
    fn dispatch_handle(&self) -> Option<Arc<dyn DispatchHandlePort>> {
        None
    }
}

// ── mount ────────────────────────────────────────────────────────────────────

/// Mount `AgentsWire` onto `router`. `host` supplies every closure captured
/// in the wire config; `gate` is the C0/C2/C3 gate supplied by shipd from the
/// proto descriptor.
///
/// Behavior is identical to the prior `apps/shipd/src/rpc/ops/agents_wire.rs::mount`.
pub fn mount<H: AgentsHost>(host: &H, gate: GateFn, router: ConnectRouter) -> ConnectRouter {
    // The shared Live-Watch kernel — same `KernelLog` the agent reactive
    // registry subscribes to.
    let agents_kernel = Arc::new(KernelLog::new(host.kernel_router()));

    // SessionsFn returns the raw items list as a Value — the wire wraps it
    // in {"items": ...}. Reads from canonical_sessions (runtime.db) —
    // canonical-only posture (Floki sessions only until #20 migrates
    // ship-code emitters to canonical session.* events).
    let agents_sessions_fn: SessionsFn = Arc::new(move || {
        let sessions =
            crate::services::sessions::canonical_projection::load_sessions().unwrap_or_default();
        serde_json::to_value(sessions).unwrap_or(serde_json::Value::Array(vec![]))
    });

    // notify_saved_fn: post a profile write fan-out so the reactive index
    // resolves fresh state. Resolves project vs user scope from the wire's
    // scope string and forwards to ship-config's `notify_agent_def_saved`.
    let notify_saved_host = host.clone();
    let notify_saved_fn: NotifySavedFn = Arc::new(move |id: String, scope_str: String| {
        let host = notify_saved_host.clone();
        Box::pin(async move {
            use ship_files::files::Scope;
            use ship_substrate::ShipHome;
            use ship_substrate::ports::{KernelLog, LocalFsBroker};
            use ship_types::SpaceId;
            let scope = match scope_str.as_str() {
                "project" => Scope::Project,
                _ => Scope::User,
            };
            let (ship_dir, space) = match scope {
                Scope::Project => {
                    let Some(ws) = host.workspace_dir() else {
                        return;
                    };
                    let project_ship_dir = ShipHome::project_ship_dir(&ws);
                    // Use the CONTENT-ID (ship.jsonc#id) as the space-id
                    // so agent-def events route to the same
                    // spaces/<content-id>/space.db every other path uses.
                    let space = ship_substrate::project::ensure_space_id(&project_ship_dir)
                        .ok()
                        .map(SpaceId::new);
                    (project_ship_dir, space)
                }
                Scope::User => (host.ship_dir(), None),
            };
            let kernel = match host.agent_def_kernel() {
                Some(shared) => shared,
                None => Arc::new(KernelLog::new(host.kernel_router())),
            };
            let fs = LocalFsBroker::new();
            ship_config::agents_write::notify_agent_def_saved(&kernel, &fs, &ship_dir, &id, space)
                .await;
        })
    });

    // stop_fn: kill a running agent via the dispatch process registry.
    // Gated: `host.dispatch_handle()` only exists when ship-code is on (the
    // shipd impl wires it under its own `#[cfg(feature = "ship-code")]`).
    // When absent, the closure returns the same `"dispatch not available"`
    // error as the original.
    let stop_host = host.clone();
    let stop_fn: StopFn = Arc::new(move |slug: String| {
        let host = stop_host.clone();
        Box::pin(async move {
            let dh = host
                .dispatch_handle()
                .ok_or_else(|| ConnectError::internal("dispatch not available"))?;
            dh.stop(&slug).await
        })
    });

    // steer_fn: send a message to a running agent via the dispatch process
    // registry. Gated identically to stop_fn.
    let steer_host = host.clone();
    let steer_fn: SteerFn = Arc::new(move |slug: String, message: String| {
        let host = steer_host.clone();
        Box::pin(async move {
            let dh = host
                .dispatch_handle()
                .ok_or_else(|| ConnectError::internal("dispatch not available"))?;
            dh.steer(&slug, &message).await
        })
    });

    // grant_fn: live-session upgrade + baseline grants persistence. The C3
    // gate is enforced by `gate` before this runs; this closure carries
    // only the two-effect domain logic.
    let grant_host = host.clone();
    let grant_fn: GrantFn = Arc::new(
        move |agent: String, new_grants: Vec<String>, ctx: RequestContext| {
            let host = grant_host.clone();
            Box::pin(async move {
                // C3 gate already enforced by the AgentsWire gate closure before
                // this runs. Extract approver agent_id from the auth context
                // injected by the middleware.
                let approver = host.extract_approver(&ctx)?;
                let mut baseline: std::collections::BTreeSet<String> =
                    crate::services::permissions::projection::provisioned_grants_for(&agent)
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                        .into_iter()
                        .collect();
                let mut upgraded = 0usize;
                {
                    let registry = host.session_registry();
                    let mut reg = registry.lock().await;
                    for session in reg.values_mut() {
                        if session.agent_id == agent {
                            baseline.extend(session.grants.iter().cloned());
                            for g in &new_grants {
                                if !session.grants.contains(g) {
                                    session.grants.push(g.clone());
                                }
                            }
                            upgraded += 1;
                        }
                    }
                }
                baseline.extend(new_grants.iter().cloned());
                let provisioned: Vec<String> = baseline.into_iter().collect();
                ship_auth::scopes::provision_grants(
                    &agent,
                    &provisioned,
                    "operator",
                    approver.as_deref(),
                )
                .map_err(|e| ConnectError::internal(format!("failed to persist grants: {e}")))?;
                Ok(serde_json::json!({
                    "agent": agent,
                    "grants": new_grants,
                    "provisioned": provisioned,
                    "sessions_upgraded": upgraded,
                }))
            })
        },
    );

    let agents_wire = Arc::new(AgentsWire::new(AgentsWireConfig {
        svc: host.agents_arc(),
        ship_dir: host.ship_dir(),
        kernel: agents_kernel,
        gate,
        sessions_fn: agents_sessions_fn,
        notify_saved: notify_saved_fn,
        dispatch_port: host.dispatch_port(),
        stop_fn,
        steer_fn,
        grant_fn,
        agent_reactive: host.agent_reactive(),
        feedback_auth: host.feedback_auth(),
        view_broadcast: host.view_broadcast(),
        collection: host.collection(),
    }));
    ship_ops_rpc::AgentsServiceExt::register(agents_wire, router)
}
