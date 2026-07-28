//! ConnectRPC op surface — the daemon's implementation of `ship.ops.v1`
//! (`ship-ops-rpc`), mounted additively beside the REST/SSE routes. Proto owns
//! op shapes + policy; this owns the handlers and (later) gate enforcement. The
//! first op, `Meta` (C0), proves the proto → trait → mount pipeline end to end.
//! See docs/ship-architecture/references/docs/op-surface.md.
//!
//! `pipelines_wire`, `agents_wire`, `ui_wire`, `terminal_wire` — each a `mount(router, …)`
//! that registers one crate-owned wire with its shipd closures. `policy`
//! carries the proto-policy introspection map.

mod agents_wire;
// ship.code.v1.CodeService — the coding-agent capability surface
// (mount split 2026-07-10). Mounted at router assembly under
// `--features bridge` (the single conditional). With the feature
// off, no route is registered — the path is absent (Connect discovers
// the service by path presence; absence is structural, not a refused
// call). The mount reuses the same `AgentDispatchAdapter` the in-process
// `AgentsService.Dispatch` wire uses (one kernel subscription serves
// both wires).
#[cfg(feature = "bridge")]
mod code_wire;
mod pipelines_wire;
pub mod policy;
// ship.ops.v1.TerminalService — the PTY/harness Connect adapter, part of
// the ship-code surface. Compiled and mounted only under
// `--features bridge`, mirroring `code_wire`: with the feature off no
// route is registered — the path is absent (structural, not a refused
// call). The underlying sessions Registry is NOT gated — it serves agent
// sessions too; only this adapter is.
#[cfg(feature = "bridge")]
mod terminal_wire;
mod ui_wire;
pub use policy::{OP_POLICY, OpPolicy, policy_for};

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_ops_rpc::{MetaResponse, OpsService, OpsServiceExt, OwnedMetaRequestView};

use crate::ship_runtime::ApiState;

/// Daemon-side `OpsService`. Holds `ApiState` so ported ops can reach runtime state.
pub struct OpsImpl {
    state: ApiState,
}

impl OpsImpl {
    /// Borrow the runtime — for handlers implemented in sibling modules
    /// (e.g. `rpc::runtime`, `rpc::events`) that need `ApiState`.
    pub(crate) fn state(&self) -> &ApiState {
        &self.state
    }
}

impl OpsService for OpsImpl {
    // The trait declares `-> impl Encodable<MetaResponse>`; the documented impl
    // pattern is a plain `async fn` returning the concrete type, which refines
    // the trait's return. Intentional and benign.
    #[allow(refining_impl_trait)]
    async fn meta(
        &self,
        _ctx: RequestContext,
        _request: OwnedMetaRequestView,
    ) -> ServiceResult<MetaResponse> {
        Ok(Response::new(MetaResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            // BUILD_GIT_SHA is emitted by build-info's emit_build_env when wired;
            // "dev" is the honest fallback for an unwired local build.
            build: option_env!("BUILD_GIT_SHA").unwrap_or("dev").to_string(),
            ..Default::default()
        }))
    }
}

/// Extract the kernel-attested pid from a request context.
/// `PeerPid` is inserted only over the Unix host socket (SO_PEERCRED); TCP and
/// non-Unix builds always return `None`, which the device crate refuses at the
/// `register_window` call site (a window token is meaningless without a pid).
fn pid_from_ctx(ctx: &connectrpc::RequestContext) -> Option<u32> {
    #[cfg(unix)]
    {
        ctx.extensions.get::<crate::auth::peer_socket::PeerPid>().and_then(|p| p.0)
    }
    #[cfg(not(unix))]
    {
        let _ = ctx;
        None
    }
}

/// The ConnectRPC op surface as an `axum::Router`, merged into the daemon's
/// authenticated routes so `auth_middleware` wraps it (injecting `AuthContext`).
/// Wire paths: `/ship.ops.v1.<Service>/<Method>`.
///
/// No transport-level gate: enforcement lives at the dispatch chokepoint
/// (`auth::gate::decide`, invoked by the `GateFn` closure built here), so
/// commands are gated identically on every transport. Reads are C0
/// (authenticated at the edge). This is the "one gate" of
/// `.artifacts/auth-gate-synthesis.md`.
pub fn router(state: ApiState) -> axum::Router {
    let svc = Arc::new(OpsImpl { state: state.clone() });
    let router = OpsServiceExt::register(svc.clone(), connectrpc::Router::new());
    let router = ship_ops_rpc::RuntimeServiceExt::register(svc.clone(), router);

    // window.* — the live-data multiplex (ADR 0001). The Connect wire is
    // deleted; the multiplex now lives in deck-axum (#276), mounted in
    // `apps/shipd/src/router.rs` inside the auth-gated group with
    // `MultiplexState` extension carrying `ShipDataPort` + registry +
    // `ShipChangeSource` impls.
    // ── Gate closure shared by all crate-owned wires ────────────────────────
    // Reads the blast from the proto descriptor (the ONE policy source) and runs
    // gate::decide — the one capability enforcement seam, now at the transport
    // handler for crate-owned wires (#94).
    let gate: ship_agents::services::rules::wire::GateFn = Arc::new({
        move |ctx: &RequestContext, op: &str| {
            let auth = ctx
                .extensions
                .get::<crate::auth::AuthContext>()
                .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
            let pol = ship_ops_rpc::policy::policy_for(op)
                .ok_or_else(|| ConnectError::internal(format!("no policy for {op}")))?;
            let caller = crate::auth::gate::caller_from_auth(auth);
            match crate::auth::gate::decide(
                pol.blast,
                crate::auth::gate::service_of(op),
                &caller,
            ) {
                crate::auth::gate::Decision::Deny(m) => {
                    Err(ConnectError::permission_denied(m.to_string()))
                }
                _ => Ok(()),
            }
        }
    });

    // rules.* — crate-owned RulesWire (ship-agents). Replaces impl RulesService for OpsImpl.
    let rules_wire = Arc::new(ship_agents::services::rules::RulesWire::new(
        state.rules_arc(),
        gate.clone(),
    ));
    let router = ship_ops_rpc::RulesServiceExt::register(rules_wire, router);
    // artifacts.* is served exclusively by the typed `ship-artifacts` wire mounted
    // below (active-space bound). No OpsImpl ArtifactsService mount — the wire is
    // the sole handler. collections.* is likewise served by the typed CollectionWire.
    // permissions.* — Slice 3 close-out: the runtime CRUD service
    // (`ship_agents::services::permissions::PermissionsWire` +
    // `PermissionsServiceImpl`) was removed — no live UI consumer
    // invokes it, and the named-preset layer is now sourced from
    // `defineConfig.agents.permissions.presets` in `stack.config.ts`
    // (the resolver's `merge_ts_declarations` populates
    // `ExtensionLibrary::permissions_presets`). The proto remains
    // in `ship-contracts/proto/ship/ops/v1/permissions.proto` —
    // the surface is reserved for the environment-design / space-
    // model work (operator-enabled runtime overrides) that will
    // re-introduce the CRUD path when runtime permissions become
    // a real thing. For now the wire is not mounted: any
    // `permissions.*` Connect-RPC call 404s, the same posture the
    // REST surface had after its 0d037516d deletion.
    // mcp.* — external MCP server config CRUD.
    // Crate-owned McpWire (ship-agents). Replaces impl McpService for OpsImpl.
    let mcp_wire = Arc::new(ship_agents::services::mcp::McpWire::new(
        state.mcp_arc(),
        gate.clone(),
    ));
    let router = ship_ops_rpc::McpServiceExt::register(mcp_wire, router);
    // skills.* (restricted ship-authoring writes + view reads + live streams).
    // Crate-owned SkillsWire (ship-agents). Replaces impl SkillsService for OpsImpl.
    let skills_kernel = Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone()));
    let skills_wire = Arc::new(ship_agents::services::skills::SkillsWire::new(
        state.skills_arc(),
        state.ship_dir().to_path_buf(),
        skills_kernel,
        gate.clone(),
    ));
    let router = ship_ops_rpc::SkillsServiceExt::register(skills_wire, router);
    // analytics.* — crate-owned AnalyticsWire (ship-analytics). All four ops are
    // C0 reads; the wire reaches analytics data through ship-analytics' service
    // module (no raw DB in this file — SF-4 preserved).
    let analytics_wire = Arc::new(ship_analytics::wire::AnalyticsWire);
    let router = ship_ops_rpc::AnalyticsServiceExt::register(analytics_wire, router);
    // isolation.* — crate-owned IsolationWire (ship-isolation). All ops are C0
    // reads; the wire calls ship_isolation::decision / decision_ops.
    let isolation_wire = Arc::new(ship_isolation::wire::IsolationWire);
    let router = ship_ops_rpc::IsolationServiceExt::register(isolation_wire, router);
    // device.* — crate-owned DeviceWire (ship-device).
    // C0 reads + C3 writes (operator-only); ports injected from shipd.
    let device_gate = gate.clone();
    let device_sessions: std::sync::Arc<dyn ship_device::WindowSessions> =
        Arc::new(crate::ports::device::OwnedRegistryWindowSessions::new(state.session_registry().clone()));
    let device_space_reg: std::sync::Arc<dyn ship_device::SpaceRegistry> =
        Arc::new(crate::ports::device::LiveSpaceRegistry::new(state.clone()));
    let device_pid: ship_device::wire::PidFn = Arc::new(pid_from_ctx);
    let device_space_fn: ship_device::wire::SpaceFn = Arc::new(|ctx: &connectrpc::RequestContext| {
        ctx.extensions
            .get::<crate::auth::AuthContext>()
            .and_then(|a| a.space_id.clone())
    });
    let device_wire = Arc::new(ship_device::wire::DeviceWire::new(
        device_gate,
        device_sessions,
        device_space_reg,
        device_pid,
        device_space_fn,
    ));
    let router = ship_ops_rpc::DeviceServiceExt::register(device_wire, router);
    let router = agents_wire::mount(router, &state, gate.clone());
    // ship.code.v1.CodeService — coding-class dispatch surface; gated
    // behind `--features bridge` (mount split 2026-07-10). With the
    // feature off, this compiles to nothing and the route is absent.
    #[cfg(feature = "bridge")]
    let router = code_wire::mount(router, &state);
    // account.* — typed AccountService (account.proto), served by the
    // `ship-account` crate (`AccountWire`). Process-global identity: token at
    // rest under `<daemon_dir>/account.jsonc`, `account.*` + `daemon.owner.*`
    // audit ride a fresh `KernelLog` over the shared kernel (Account-scoped →
    // account.db). If the gateway client fails to build, the mount is skipped
    // and the four account RPCs return Unimplemented.
    let router = match crate::rpc::account::build_account_wire(
        std::sync::Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone())),
        state.daemon_dir().join("account.jsonc"),
        &std::env::var("SHIP_GATEWAY_URL").unwrap_or_else(|_| "https://api.getship.dev".to_string()),
    ) {
        Some(account_wire) => {
            ship_ops_rpc::AccountServiceExt::register(account_wire, router)
        }
        None => router,
    };
    let router = ship_ops_rpc::EventsServiceExt::register(svc.clone(), router);
    // ui.* — the unified authoring service (compiler analysis + view authoring
    // + component discovery). Crate-owned `UiWire` (ship-ui); closures in
    // `ui_wire`. Replaces the retired `sui.*` and `view.*` services.
    let router = ui_wire::mount(router, &state);
    // pipelines.* — crate-owned PipelinesWire (ship-pipeline-runtime): definition
    // CRUD (List/Get/Put/Patch/Delete) over the engine-shaped surface.
    let router = pipelines_wire::mount(router, &state, &gate);
    // terminal.* — the L2 Connect adapter over the unified live-session
    // registry (ship_agents::services::sessions::Registry). All eight
    // methods: Spawn/Stop/List/Get/SendInput/Resize/SessionBytes/SessionEvents.
    // ship-code surface: gated like code_wire — feature off ⇒ route absent.
    #[cfg(feature = "bridge")]
    let router = terminal_wire::mount(router, &state);
    // search — ShipdSearchWire (rpc::search) branches on the request scope:
    // THIS_SPACE runs the active-space ShipdSearchContext (the historical
    // single-context path); GLOBAL fans out across reachable spaces via
    // ship_search::run_global. Composition + ranking + merge stay in the
    // scope-blind ship-search crate.
    let search_wire =
        std::sync::Arc::new(crate::rpc::search::ShipdSearchWire::new(state.clone()));
    let router = ship_ops_rpc::SearchServiceExt::register(search_wire, router);
    // collection.* / config.* / artifacts.* bind to the active space's per-scope
    // `space.db`. In space-less mode (no active space) there is no context to bind
    // and no `global` pseudo-space to fall back onto (S2 home-space kill), so the
    // three space-scoped wires are skipped — their RPCs then return Unimplemented
    // rather than fabricate a partition. Mounted only when a space is active.
    //
    // collections.* — the SOLE CollectionsService (collections.proto), the typed
    // crate-owned guts. Mounted with the active-space SpaceContext + the ONE shared
    // schema registry (C-2 §3.1 / UD-7) so a wire-created schema is visible to the
    // internal trigger/view callers.
    let router = match state.collection_ctx() {
        Some(collection_ctx) => {
            let collection_kernel = collection_ctx.kernel.clone();
            let collection_schemas = state.collection_schemas();
            let collection_wire = std::sync::Arc::new(ship_collection::CollectionWire::with_schemas(
                collection_kernel,
                collection_schemas,
                move |_ctx: &connectrpc::RequestContext| collection_ctx.clone(),
            ));
            ship_ops_rpc::CollectionsServiceExt::register(collection_wire, router)
        }
        None => router,
    };
    // settings.* — SettingsService (settings.proto): the unified, scope-
    // discriminated settings keystone (SPACE / ACCOUNT / RUNTIME) served by the
    // ship-config crate (`SettingsWire`). Replaces the retired ConfigService.
    // Mounted unconditionally — the wire resolves its own per-request scope
    // context (account.db + the active-space space.db + the operator verdict).
    // SPACE ops error cleanly when no active space; ACCOUNT/RUNTIME always serve.
    // The legacy `SpaceService.SetSetting` stays mounted beside `Set(SPACE)`
    // until S4 repoints callers.
    let settings_wire = crate::rpc::settings::settings_wire(state.clone());
    let router = ship_ops_rpc::SettingsServiceExt::register(settings_wire, router);
    // artifacts.* v2 — ship-artifacts (DataStore + blob backend), overrides
    // OpsImpl's ArtifactsService registration for all 5 RPCs including Watch.
    let router = match state.artifact_ctx() {
        Some(artifact_ctx) => {
            let artifact_kernel = artifact_ctx.kernel.clone();
            let global_dir = state.global_blob_dir().to_path_buf();
            let artifacts_wire = std::sync::Arc::new(ship_artifacts::ArtifactsWire::new(
                artifact_kernel,
                global_dir,
                move |_ctx: &connectrpc::RequestContext| artifact_ctx.clone(),
            ));
            ship_ops_rpc::ArtifactsServiceExt::register(artifacts_wire, router)
        }
        None => router,
    };
    // space.* — SpaceService (ship-space crate): typed space identity + config
    // overlay + registry + Watch. The SOLE space service (the legacy SpaceService
    // — JsonResponse/ProjectRecord vestiges, *Live twins, window RPCs — was
    // retired in the space cutover; windows moved to DeviceService above).
    let space_wire = crate::rpc::space::space_wire(state.clone(), gate.clone());
    let router = ship_ops_rpc::SpaceServiceExt::register(space_wire, router);
    router.into_axum_router()
}
