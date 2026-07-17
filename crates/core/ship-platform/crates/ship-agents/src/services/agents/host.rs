//! `AgentsHost` — the shipd-side wiring point for `AgentsWire`.
//!
//! `AgentsWire` is the crate-owned ConnectRPC impl of `AgentsService`. To
//! mount it, shipd historically built six async closures + four port adapters
//! from its `ApiState`. The trait + `mount` fn here let ship-agents own the
//! shape of that wiring; shipd just implements `AgentsHost` and calls
//! `mount(state, gate, router)`.
//! The concrete `AgentDispatchAdapter` stays in shipd — it closes over shipd's
//! runtime state. The abstract `AgentDispatchPort` trait is re-exposed here as
//! the `Host`-returning type so the host bundle reads as a single surface.
//!
//! Capability split:
//!  * `AgentsHost` — the *core* trait that exposes only ship-agents /
//!    ship-substrate types. Implementing it does NOT require ship-auth or
//!    ship-reactive in scope.
//!  * [`AgentsGrantHost`] — opt-in capability for the `agent.grant`
//!    path. Depends on `ship-auth` because the grant closure calls
//!    `ship_auth::scopes::provision_grants` and reads
//!    `ship_auth::scopes::SessionRegistry`.
//!  * [`AgentsWatchHost`] — opt-in capability for the `agent.*.Watch`
//!    stream. Depends on `ship-reactive` because it returns
//!    `Option<Arc<ReactiveRegistry>>`.
//!
//! Two mount fns:
//!  * [`mount`] — the *lean* mount a read-only consumer (e.g. helmd) calls.
//!    Requires only `AgentsHost`. The grant path returns
//!    `ConnectError::unimplemented` and the watch path sees
//!    `agent_reactive: None` (which `AgentsWire::watch` already translates
//!    into `ConnectError::unavailable`).
//!  * [`mount_full`] — the *full* mount shipd calls. Requires
//!    `AgentsHost + AgentsGrantHost + AgentsWatchHost` and wires the real
//!    closures identical to the prior single-mount body.
//!
//! No behavior change on the shipd (full) path: every closure + port is
//! identical to the prior `apps/shipd/src/rpc/ops/agents_wire.rs::mount` —
//! only the location and trait split changed.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Router as ConnectRouter};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::services::agents::dispatch_core::AgentDispatchPort;
use crate::services::agents::rpc_wire::{
    AgentsWire, AgentsWireConfig, GateFn, GrantFn, NotifySavedFn, SessionsFn, SteerFn, StopFn,
};
use crate::services::agents::service::AgentsService;
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

/// Lean, ship-agents/ship-substrate-only accessor surface used by [`mount`].
///
/// Every implementor of `AgentsHost` can be wired into [`mount`] without
/// pulling in `ship-auth` or `ship-reactive` in scope. To also handle
/// `agent.grant` or `agent.*.Watch`, opt in to [`AgentsGrantHost`] and/or
/// [`AgentsWatchHost`] and use [`mount_full`].
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

    /// The live agent-def index (project scope) when mounted. When present, the
    /// profile READ handlers serve list/detail from its DataStore snapshot
    /// instead of re-reading `.ship/agents/*.jsonc`. `None` for a space-less
    /// boot or a host that does not run the index.
    fn agent_def_index(
        &self,
    ) -> Option<Arc<ship_config::agents_index::AgentDefIndex>> {
        None
    }

    /// The dispatch port — agent dispatch writes events through this.
    fn dispatch_port(&self) -> Arc<dyn AgentDispatchPort>;

    /// Extract the verified approver (`agent_id`) from the auth context for
    /// `agent.grant`. Returns `Err(unauthenticated)` when the auth extension
    /// is missing; `Ok(None)` for anonymous / daemon-token requests.
    ///
    /// This is ship-auth-*free*: it operates on the ConnectRPC
    /// `RequestContext`, not on any ship-auth type. It is part of the core
    /// trait because the lean `grant_fn` closure still extracts an approver
    /// (or bails the same way) so wire contract is uniform across mounts.
    fn extract_approver(&self, ctx: &RequestContext) -> Result<Option<String>, ConnectError>;

    /// Dispatch handle — `Some` only when ship-code built the process
    /// registry. When `None`, stop / steer return `"dispatch not available"`.
    /// Ship-agents always treats this as `None` on non-`ship-code`
    /// builds; shipd swaps in the live registry when ship-code is on.
    fn dispatch_handle(&self) -> Option<Arc<dyn DispatchHandlePort>> {
        None
    }
}

/// Capability trait for the `agent.grant` path. Brings `ship-auth` types into
/// scope on the host implementor. [`mount_full`] requires this.
pub trait AgentsGrantHost: AgentsHost {
    /// Per-process session registry — `agent.grant` mutates grants here.
    fn session_registry(&self) -> ship_auth::scopes::SessionRegistry;
}

/// Capability trait for the `agent.*.Watch` stream. Brings `ship-reactive`
/// types into scope on the host implementor. [`mount_full`] requires this.
pub trait AgentsWatchHost: AgentsHost {
    /// The agent `Watch` reactive registry — `None` when space-less.
    fn agent_reactive(&self) -> Option<Arc<ship_reactive::ReactiveRegistry>>;
}

// ── mount ────────────────────────────────────────────────────────────────────

/// The non-capability half of an `AgentsWireConfig` — holds the shared
/// closures + readers that neither mount depends on `ship-auth` or
/// `ship-reactive` for. The two capability-shaped fields (`grant_fn`,
/// `agent_reactive`) are filled in by the caller: [`mount`] substitutes a
/// lean stub + `None`; [`mount_full`] uses the live values.
struct AgentsWireCoreConfig {
    svc:              Arc<dyn AgentsService>,
    ship_dir:         PathBuf,
    kernel:           Arc<KernelLog>,
    gate:             GateFn,
    sessions_fn:      SessionsFn,
    notify_saved:     NotifySavedFn,
    dispatch_port:    Arc<dyn AgentDispatchPort>,
    stop_fn:          StopFn,
    steer_fn:         SteerFn,
}

/// Build the ship-agents + ship-substrate portion of an `AgentsWireConfig`.
/// Pulls every accessor / closure that does NOT require `ship-auth` or
/// `ship-reactive` — i.e. the closure-half of `AgentsHost`. The two
/// capability-shaped closures / fields (`grant_fn`, `agent_reactive`) are
/// filled in by the caller. [`mount`] substitutes lean stubs; [`mount_full`]
/// uses the real values.
fn build_core<H: AgentsHost>(host: &H, gate: GateFn) -> AgentsWireCoreConfig {
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
                    // Marker cutover: agents/ is a domain dir at the STACK
                    // ROOT — the workspace root itself anchors the reader.
                    // The CONTENT-ID from the registry pointer is the
                    // space-id, so agent-def events route to the same
                    // spaces/<content-id>/space.db every other path uses.
                    let space = ship_substrate::project::resolve_active_space(&ws)
                        .map(SpaceId::new);
                    (ws.clone(), space)
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

    AgentsWireCoreConfig {
        svc: host.agents_arc(),
        ship_dir: host.ship_dir(),
        kernel: agents_kernel,
        gate,
        sessions_fn: agents_sessions_fn,
        notify_saved: notify_saved_fn,
        dispatch_port: host.dispatch_port(),
        stop_fn,
        steer_fn,
    }
}
/// Lean stub `grant_fn`: every call returns `unimplemented`. Lets a consumer
/// that has no `ship-auth` capability still mount the full `AgentsWire`
/// surface; calls to `agent.grant` reject with a well-typed error.
fn lean_grant_fn() -> GrantFn {
    Arc::new(
        move |_agent: String, _new_grants: Vec<String>, _ctx: RequestContext| {
            Box::pin(async move {
                Err(ConnectError::unimplemented(
                    "agent.grant not available on this consumer",
                ))
            })
        },
    )
}

/// Mount `AgentsWire` onto `router` — the LEAN mount.
///
/// `host` supplies every ship-agents/ship-substrate accessor (core
/// `AgentsHost`). The grant path is stubbed to
/// `ConnectError::unimplemented("agent.grant not available on this consumer")`
/// and the watch path is wired with `agent_reactive: None` (which
/// `AgentsWire::watch` already translates into
/// `ConnectError::unavailable("agent watch unavailable: no active space")`).
/// A lean consumer like helmd can implement just `AgentsHost` and mount
/// without importing `ship-auth` or `ship-reactive`.
pub fn mount<H: AgentsHost>(host: &H, gate: GateFn, router: ConnectRouter) -> ConnectRouter {
    let core = build_core(host, gate);
    let agents_wire = Arc::new(AgentsWire::new(AgentsWireConfig {
        svc:              core.svc,
        ship_dir:         core.ship_dir,
        kernel:           core.kernel,
        gate:             core.gate,
        sessions_fn:      core.sessions_fn,
        notify_saved:     core.notify_saved,
        dispatch_port:    core.dispatch_port,
        stop_fn:          core.stop_fn,
        steer_fn:         core.steer_fn,
        grant_fn:         lean_grant_fn(),
        agent_reactive:   None,
        def_index:        host.agent_def_index(),
    }));
    ship_ops_rpc::AgentsServiceExt::register(agents_wire, router)
}

/// Mount `AgentsWire` onto `router` — the FULL mount.
///
/// `host` must implement the core trait *plus* [`AgentsGrantHost`] (for
/// `agent.grant`) and [`AgentsWatchHost`] (for the `Watch` stream). This is
/// what shipd uses; behavior is byte-for-byte identical to the prior single
/// `mount` body.
pub fn mount_full<H>(
    host: &H,
    gate: GateFn,
    router: ConnectRouter,
) -> ConnectRouter
where
    H: AgentsHost + AgentsGrantHost + AgentsWatchHost,
{
    let core = build_core(host, gate);

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
                // Slice 3 close-out: `ship_agents::services::permissions`
                // was removed (the runtime CRUD service had no live UI
                // consumer; the named-preset layer moved to
                // `defineConfig.agents.permissions.presets` in
                // `stack.config.ts`). `provisioned_grants_for` (the
                // granted-capability baseline) is the OTHER
                // `provisioned_grants_for` — it lives in
                // `ship_auth::grants_projection` (re-exported here
                // so the host stays independent of the deleted
                // services/permissions module).
                let mut baseline: std::collections::BTreeSet<String> =
                    ship_auth::grants_projection::provisioned_grants_for(&agent)
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
        svc:              core.svc,
        ship_dir:         core.ship_dir,
        kernel:           core.kernel,
        gate:             core.gate,
        sessions_fn:      core.sessions_fn,
        notify_saved:     core.notify_saved,
        dispatch_port:    core.dispatch_port,
        stop_fn:          core.stop_fn,
        steer_fn:         core.steer_fn,
        grant_fn,
        agent_reactive:   host.agent_reactive(),
        def_index:        host.agent_def_index(),
    }));
    ship_ops_rpc::AgentsServiceExt::register(agents_wire, router)
}
