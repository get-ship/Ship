//! ConnectRPC wire layer for `AgentsService` — the keystone service.
//!
//! `AgentsWire` holds all the state the handler needs and implements
//! `ship_proto::ship::ops::v1::AgentsService` so shipd can mount it via
//! `AgentsServiceExt::register`, replacing the `impl AgentsService for OpsImpl`
//! that lived in `apps/shipd/src/agent_rpc.rs` + `agent_rpc_helpers.rs`.
//!
//! Three planes, each with injected ports:
//!   * Profile reads (C0) — `ship_dir` + domain functions directly.
//!   * Runtime reads (C0) — `SessionsFn` (pty) + kernel for live stream.
//!   * Profile writes (C2) — `Arc<dyn AgentsService>` + `NotifySavedFn`.
//!   * Orchestration control (C2/C3) — `DispatchFn`/`StopFn`/`SteerFn`/`GrantFn`.
//!   * Watch stream (agents-arc E5) — `Arc<ReactiveRegistry>` + ship_dir.
//!
//! `agents.grant` preserves its C3 operator-authority check: the `gate` closure
//! (supplied by shipd from the proto descriptor) enforces C3 before any handler
//! body runs — unchanged from the `OpsImpl` path. The `GrantFn` carries the
//! two-effect domain logic (live-session upgrade + baseline persistence).
// FILE_CAP_OVERRIDE: AgentsService has 23 RPC methods across 4 planes; one impl block.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{Value, json};
use ship_settings::agents_index::AgentDefIndex;
use ship_proto::ship::ops::v1::{AgentCatalogsResponse, AgentDetail, AgentDoc, AgentIndexResponse, AgentSettingsResponse, AgentTemplatesResponse, AgentWireResponse, AgentsService as AgentsServiceTrait, AgentsSummariesResponse, DispatchResult, GrantResult, JsonResponse, PermissionPresetsResponse, ScopedMutation, SessionsListResponse, SteerResult, StopResult, WatchEvent};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::ports::KernelLog;

use crate::services::agents::dispatch_core::{
    AgentDispatchPort, DispatchCoreError, dispatch_agent_session_core,
};
use crate::services::agents::dispatch_request::DispatchRequest;
use crate::services::agents::service::AgentsService;
use crate::services::agents::watch as agent_watch;
use crate::services::agents::{state, view_state};

// ── Closure type aliases ──────────────────────────────────────────────────────

/// Gate closure: (request_context, op_name) → Ok or PermissionDenied.
/// Shipd supplies this from `gate::decide` + proto descriptor.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Returns a JSON Value of current pty session items for the sessions read.
pub type SessionsFn = Arc<dyn Fn() -> Value + Send + Sync>;

/// Notify the agent-def reactive index after a profile write.
/// Signature: (id: String, scope_str: String) where scope_str is "project" | "user".
pub type NotifySavedFn = Arc<dyn Fn(String, String) -> BoxFuture<()> + Send + Sync>;

/// Async stop closure: (slug) → Result<StopResult JSON, ConnectError>.
pub type StopFn = Arc<dyn Fn(String) -> BoxFuture<Result<Value, ConnectError>> + Send + Sync>;

/// Async steer closure: (slug, message) → Result<SteerResult JSON, ConnectError>.
pub type SteerFn =
    Arc<dyn Fn(String, String) -> BoxFuture<Result<Value, ConnectError>> + Send + Sync>;

/// Async grant closure: (agent_id, grants) → Result<GrantResult JSON, ConnectError>.
/// Captures auth state (session registry) from shipd. The C3 gate is enforced by
/// the `gate` closure before this runs; this fn carries only domain logic.
pub type GrantFn = Arc<
    dyn Fn(String, Vec<String>, RequestContext) -> BoxFuture<Result<Value, ConnectError>>
        + Send
        + Sync,
>;

// ── Wire struct ───────────────────────────────────────────────────────────────

/// ConnectRPC wire for `AgentsService`.
pub struct AgentsWire {
    svc: Arc<dyn AgentsService>,
    ship_dir: PathBuf,
    kernel: Arc<KernelLog>,
    gate: GateFn,
    sessions_fn: SessionsFn,
    notify_saved: NotifySavedFn,
    dispatch_port: Arc<dyn AgentDispatchPort>,
    stop_fn: StopFn,
    steer_fn: SteerFn,
    grant_fn: GrantFn,
    agent_reactive: Option<Arc<ReactiveRegistry>>,
    /// The live agent-def index (project scope). When present, the profile
    /// READ handlers serve list/detail from the DataStore snapshot instead of
    /// re-reading `.ship/agents/*.jsonc`. `None` in space-less / lean mounts.
    def_index: Option<Arc<AgentDefIndex>>,
}

/// Named-field constructor input for [`AgentsWire`] — replaces the former
/// 14-positional-argument `new` (every sibling wire takes a config or a
/// handful of args; a positional list this long invited transposition bugs).
pub struct AgentsWireConfig {
    pub svc: Arc<dyn AgentsService>,
    pub ship_dir: PathBuf,
    pub kernel: Arc<KernelLog>,
    pub gate: GateFn,
    pub sessions_fn: SessionsFn,
    pub notify_saved: NotifySavedFn,
    pub dispatch_port: Arc<dyn AgentDispatchPort>,
    pub stop_fn: StopFn,
    pub steer_fn: SteerFn,
    pub grant_fn: GrantFn,
    pub agent_reactive: Option<Arc<ReactiveRegistry>>,
    pub def_index: Option<Arc<AgentDefIndex>>,
}

impl AgentsWire {
    pub fn new(config: AgentsWireConfig) -> Self {
        let AgentsWireConfig {
            svc,
            ship_dir,
            kernel,
            gate,
            sessions_fn,
            notify_saved,
            dispatch_port,
            stop_fn,
            steer_fn,
            grant_fn,
            agent_reactive,
            def_index,
        } = config;
        Self {
            svc,
            ship_dir,
            kernel,
            gate,
            sessions_fn,
            notify_saved,
            dispatch_port,
            stop_fn,
            steer_fn,
            grant_fn,
            agent_reactive,
            def_index,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_scope_opt(scope: &str) -> Result<Option<ship_files::files::Scope>, ConnectError> {
    match scope {
        "" => Ok(None),
        "project" => Ok(Some(ship_files::files::Scope::Project)),
        "user" => Ok(Some(ship_files::files::Scope::User)),
        other => Err(ConnectError::invalid_argument(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn parse_scope_required(scope: &str) -> Result<ship_files::files::Scope, ConnectError> {
    match scope {
        "project" => Ok(ship_files::files::Scope::Project),
        "user" => Ok(ship_files::files::Scope::User),
        other => Err(ConnectError::invalid_argument(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn scope_str(s: ship_files::files::Scope) -> String {
    match s {
        ship_files::files::Scope::Project => "project".to_string(),
        ship_files::files::Scope::User => "user".to_string(),
    }
}

fn agents_err(e: state::Error) -> ConnectError {
    match e {
        state::Error::NotFound(m) => ConnectError::not_found(m),
        state::Error::AlreadyExists(m) | state::Error::BadId(m) => {
            ConnectError::invalid_argument(m)
        }
        other => ConnectError::internal(other.to_string()),
    }
}

fn scoped(id: &str, resolved: ship_files::files::Scope) -> ScopedMutation {
    ScopedMutation {
        id: id.to_string(),
        scope: scope_str(resolved),
        ..Default::default()
    }
}

// ── Snapshot helpers for Watch ─────────────────────────────────────────────────

fn all_scope_snapshot(ship_dir: &Path) -> Value {
    json!({
        "summaries": summaries_value(ship_dir),
        "catalogs": catalogs_value(ship_dir),
    })
}

fn agent_scope_snapshot(ship_dir: &Path, id: &str) -> Value {
    json!({
        "detail": detail_value(ship_dir, id),
        "wire": wire_canvas_value(ship_dir, id),
        "settings": settings_value(ship_dir, id),
    })
}

fn summaries_value(ship_dir: &Path) -> Value {
    let items = view_state::list_agents(ship_dir);
    serde_json::to_value(json!({ "items": items })).unwrap_or(Value::Null)
}

fn catalogs_value(ship_dir: &Path) -> Value {
    serde_json::to_value(view_state::list_agent_catalogs(ship_dir)).unwrap_or(Value::Null)
}

fn detail_value(ship_dir: &Path, id: &str) -> Value {
    view_state::load_agent(ship_dir, id)
        .and_then(|d| serde_json::to_value(d).ok())
        .unwrap_or(Value::Null)
}

fn wire_canvas_value(ship_dir: &Path, id: &str) -> Value {
    use crate::services::agents::wire;
    wire::build(id, ship_dir)
        .and_then(|w| serde_json::to_value(w).ok())
        .unwrap_or(Value::Null)
}

fn settings_value(ship_dir: &Path, id: &str) -> Value {
    use crate::services::agents::settings_view;
    settings_view::build(id, ship_dir)
        .and_then(|s| serde_json::to_value(s).ok())
        .unwrap_or(Value::Null)
}

// ── Proto conversion ──────────────────────────────────────────────────────────
// CANONICAL ship_reactive ↔ reactive.proto bridge — pipeline-defs's
// wire delegates here instead of keeping a byte-identical copy.

pub fn patch_op_to_proto(op: &ship_reactive::PatchOp) -> ship_proto::ship::ops::v1::PatchOp {
    use ship_proto::ship::ops::v1::PatchOp as P;
    use ship_reactive::PatchOp::*;
    match op {
        Set => P::PATCH_OP_SET,
        Merge => P::PATCH_OP_MERGE,
        Append => P::PATCH_OP_APPEND,
        Prepend => P::PATCH_OP_PREPEND,
        Insert => P::PATCH_OP_INSERT,
        Remove => P::PATCH_OP_REMOVE,
        Move => P::PATCH_OP_MOVE,
        Splice => P::PATCH_OP_SPLICE,
        Inc => P::PATCH_OP_INC,
    }
}

pub fn patch_frame_to_proto(frame: ship_reactive::PatchFrame) -> WatchEvent {
    use ship_proto::ship::ops::v1::{Patch as ProtoPatch, PatchFrame as ProtoFrame};
    let patches: Vec<ProtoPatch> = frame
        .patches
        .into_iter()
        .map(|p| ProtoPatch {
            op: buffa::EnumValue::from(patch_op_to_proto(&p.op)),
            path: p.path,
            value: p.value.map(|v| serde_json::to_vec(&v).unwrap_or_default()),
            index: p.index.unwrap_or(0) as i32,
            from: p.from.unwrap_or(0) as i32,
            to: p.to.unwrap_or(0) as i32,
            count: p.count.unwrap_or(0) as i32,
            by: p.by.unwrap_or(0.0),
            v: 1,
            ..Default::default()
        })
        .collect();
    WatchEvent {
        body: Some(ship_proto::ship::ops::v1::__buffa::oneof::watch_event::Body::Patches(
            Box::new(ProtoFrame {
                patches,
                ..Default::default()
            }),
        )),
        ..Default::default()
    }
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl AgentsServiceTrait for AgentsWire {
    // ── Profile reads (C0) ────────────────────────────────────────────────────

    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListAgentsRequestView,
    ) -> ServiceResult<AgentIndexResponse> {
        // The C0 List read goes through the composed roll-up — no
        // `.ship/agents/*.jsonc` directory scan survives the W5 follow-up
        // (W5b migrated `view_state`; this completes the loop for the
        // wire). `None` → cwd-walked space root (`view_state::resolve_space_root`).
        let items = state::list(None);
        serde_json::from_value::<AgentIndexResponse>(json!({ "items": items }))
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("agents.list decode: {e}")))
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListAgentsSummariesRequestView,
    ) -> ServiceResult<AgentsSummariesResponse> {
        // Serve project scope from the index snapshot when mounted; the file
        // reader is the space-less / lean fallback.
        let items = match self.def_index.as_ref() {
            Some(index) => view_state::list_agents_from_index(index, &self.ship_dir).await,
            None => view_state::list_agents(&self.ship_dir),
        };
        serde_json::from_value::<AgentsSummariesResponse>(json!({ "items": items }))
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("agents.summaries decode: {e}")))
    }

    async fn detail(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGetAgentDetailRequestView,
    ) -> ServiceResult<AgentDetail> {
        let id = request.id;
        let detail = match self.def_index.as_ref() {
            Some(index) => view_state::load_agent_from_index(index, &self.ship_dir, id).await,
            None => view_state::load_agent(&self.ship_dir, id),
        }
        .ok_or_else(|| ConnectError::not_found(format!("agent `{id}`")))?;
        serde_json::from_value::<AgentDetail>(
            serde_json::to_value(detail)
                .map_err(|e| ConnectError::internal(format!("agents.detail serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.detail decode: {e}")))
    }

    async fn catalogs(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListAgentsCatalogsRequestView,
    ) -> ServiceResult<AgentCatalogsResponse> {
        let catalogs = view_state::list_agent_catalogs(&self.ship_dir);
        serde_json::from_value::<AgentCatalogsResponse>(
            serde_json::to_value(catalogs)
                .map_err(|e| ConnectError::internal(format!("agents.catalogs serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.catalogs decode: {e}")))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGetAgentRequestView,
    ) -> ServiceResult<AgentDoc> {
        let id = request.id;
        let scope = parse_scope_opt(request.scope)?;
        // The C0 Get read goes through the composed roll-up — the
        // `with_roots` (no-index) path the helmd posture mounts. The
        // prior `state::load_doc` reached `.ship/agents/*.jsonc` and
        // mapped `state::Error::NotFound(id.into())` to a bare-`id`
        // ConnectError; the composed read formats the NotFound as
        // `agent \`id\`` matching the canonical `view_state::load_agent`
        // shape `rpc_wire::detail` uses.
        let (resolved, doc) = state::load_doc(None, id, scope).map_err(|e| match e {
            state::Error::NotFound(m) => ConnectError::not_found(m),
            other => ConnectError::internal(other.to_string()),
        })?;
        let scope_tag = state::ScopeTag::from(resolved);
        let scope_str_val = serde_json::to_value(&scope_tag)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        Ok(Response::new(AgentDoc {
            id: id.to_string(),
            scope: scope_str_val,
            doc_json: doc.to_string(),
            ..Default::default()
        }))
    }

    async fn wire(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGetAgentWireRequestView,
    ) -> ServiceResult<AgentWireResponse> {
        use crate::services::agents::wire;
        let id = request.id;
        let ctx = wire::build(id, &self.ship_dir)
            .ok_or_else(|| ConnectError::not_found(format!("agent `{id}`")))?;
        serde_json::from_value::<AgentWireResponse>(
            serde_json::to_value(ctx)
                .map_err(|e| ConnectError::internal(format!("agents.wire serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.wire decode: {e}")))
    }

    async fn settings(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGetAgentSettingsRequestView,
    ) -> ServiceResult<AgentSettingsResponse> {
        use crate::services::agents::settings_view;
        let id = request.id;
        let ctx = settings_view::build(id, &self.ship_dir)
            .ok_or_else(|| ConnectError::not_found(format!("agent `{id}`")))?;
        serde_json::from_value::<AgentSettingsResponse>(
            serde_json::to_value(ctx)
                .map_err(|e| ConnectError::internal(format!("agents.settings serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.settings decode: {e}")))
    }

    async fn templates(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListAgentTemplatesRequestView,
    ) -> ServiceResult<AgentTemplatesResponse> {
        use crate::services::agents::create;
        serde_json::from_value::<AgentTemplatesResponse>(
            json!({ "items": create::builtin_templates() }),
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.templates decode: {e}")))
    }

    async fn permission_presets(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListAgentPermissionPresetsRequestView,
    ) -> ServiceResult<PermissionPresetsResponse> {
        use crate::services::agents::create;
        serde_json::from_value::<PermissionPresetsResponse>(
            json!({ "items": create::builtin_permission_presets() }),
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.permission_presets decode: {e}")))
    }

    async fn create_ctx(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedGetAgentCreateCtxRequestView,
    ) -> ServiceResult<JsonResponse> {
        use crate::services::agents::create;
        let ctx = create::build_create_ctx();
        let json_str = serde_json::to_string(&ctx)
            .map_err(|e| ConnectError::internal(format!("agents.create_ctx serialize: {e}")))?;
        Ok(Response::new(JsonResponse {
            json: json_str,
            ..Default::default()
        }))
    }

    // ── Runtime reads (C0) ────────────────────────────────────────────────────

    async fn sessions(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListSessionsReqView,
    ) -> ServiceResult<SessionsListResponse> {
        // sessions_fn() returns the items array Value directly.
        let items = (self.sessions_fn)();
        serde_json::from_value::<SessionsListResponse>(json!({ "items": items }))
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("agents.sessions decode: {e}")))
    }

    #[allow(refining_impl_trait)]
    async fn sessions_live(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListSessionsReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        // sessions_fn() returns the items array Value; wrap in {"items": ...} for the stream.
        let initial_items = (self.sessions_fn)();
        let sessions_fn = self.sessions_fn.clone();
        let mut rx = self.kernel.subscribe();
        let stream = async_stream::stream! {
            if let Ok(json) = serde_json::to_string(&json!({ "items": initial_items })) {
                yield Ok::<JsonResponse, ConnectError>(JsonResponse { json, ..Default::default() });
            }
            loop {
                match rx.recv().await {
                    Ok(_ev) => {
                        while rx.try_recv().is_ok() {}
                        let value = json!({ "items": (sessions_fn)() });
                        if let Ok(json) = serde_json::to_string(&value) {
                            yield Ok(JsonResponse { json, ..Default::default() });
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    // ── Profile writes (C2) — gate enforced by the gate closure ──────────────

    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedCreateAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        use crate::services::agents::service::CreateArgs;
        (self.gate)(&ctx, "agents.create")?;
        let id = request.id;
        let scope = parse_scope_opt(request.scope)?;
        let doc = serde_json::from_str(request.doc_json)
            .map_err(|e| ConnectError::invalid_argument(format!("doc_json: {e}")))?;
        let resolved = self
            .svc
            .create(CreateArgs {
                id: id.to_string(),
                doc,
                scope,
            })
            .map_err(agents_err)?;
        (self.notify_saved)(id.to_string(), scope_str(resolved)).await;
        Ok(Response::new(scoped(id, resolved)))
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedPutAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        use crate::services::agents::service::PutArgs;
        (self.gate)(&ctx, "agents.put")?;
        let id = request.id;
        let scope = parse_scope_opt(request.scope)?;
        let doc = serde_json::from_str(request.doc_json)
            .map_err(|e| ConnectError::invalid_argument(format!("doc_json: {e}")))?;
        let resolved = self
            .svc
            .put(PutArgs {
                id: id.to_string(),
                doc,
                scope,
            })
            .map_err(agents_err)?;
        (self.notify_saved)(id.to_string(), scope_str(resolved)).await;
        Ok(Response::new(scoped(id, resolved)))
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedPatchAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "agents.patch")?;
        let id = request.id;
        let scope = parse_scope_opt(request.scope)?;
        let patch: json_patch::Patch = serde_json::from_str(request.patch_json)
            .map_err(|e| ConnectError::invalid_argument(format!("patch_json: {e}")))?;
        let resolved = self.svc.patch(id, scope, &patch).map_err(agents_err)?;
        (self.notify_saved)(id.to_string(), scope_str(resolved)).await;
        Ok(Response::new(scoped(id, resolved)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedDeleteAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "agents.delete")?;
        let id = request.id;
        let scope = parse_scope_opt(request.scope)?;
        let resolved = self.svc.delete(id, scope).map_err(agents_err)?;
        (self.notify_saved)(id.to_string(), scope_str(resolved)).await;
        Ok(Response::new(scoped(id, resolved)))
    }

    async fn rename(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedRenameAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "agents.rename")?;
        let id = request.id;
        let new_id = request.new_id;
        let scope = parse_scope_opt(request.scope)?;
        let resolved = self.svc.rename(id, new_id, scope).map_err(agents_err)?;
        (self.notify_saved)(id.to_string(), scope_str(resolved)).await;
        (self.notify_saved)(new_id.to_string(), scope_str(resolved)).await;
        Ok(Response::new(scoped(new_id, resolved)))
    }

    async fn duplicate(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedDuplicateAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        use crate::services::agents::service::DuplicateArgs;
        (self.gate)(&ctx, "agents.duplicate")?;
        let id = request.id;
        let new_id = request.new_id;
        let source_scope = parse_scope_opt(request.scope)?;
        let to_scope = parse_scope_opt(request.to)?;
        let resolved = self
            .svc
            .duplicate(DuplicateArgs {
                source_id: id.to_string(),
                source_scope,
                new_id: new_id.to_string(),
                to_scope,
            })
            .map_err(agents_err)?;
        (self.notify_saved)(new_id.to_string(), scope_str(resolved)).await;
        Ok(Response::new(scoped(new_id, resolved)))
    }

    async fn move_scope(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedMoveAgentScopeRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "agents.move_scope")?;
        let id = request.id;
        let from = parse_scope_required(request.from)?;
        let to = parse_scope_required(request.to)?;
        self.svc.move_scope(id, from, to).map_err(agents_err)?;
        (self.notify_saved)(id.to_string(), scope_str(from)).await;
        (self.notify_saved)(id.to_string(), scope_str(to)).await;
        Ok(Response::new(scoped(id, to)))
    }

    // ── Orchestration control (C2/C3) ─────────────────────────────────────────
    //
    // The gate closure (same blast-radius source as rules/mcp/permissions/skills)
    // is called first. `agents.grant` is C3Critical — the gate blocks non-operators
    // before the GrantFn body runs, preserving the security invariant.

    async fn dispatch(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedDispatchAgentRequestView,
    ) -> ServiceResult<DispatchResult> {
        (self.gate)(&ctx, "agents.dispatch")?;

        // When the daemon wires the in-memory AgentDefIndex (shipd's normal
        // posture), consult the index FIRST (the canonical, live read
        // model) and fall back to the composed-roll-up check only when the
        // index doesn't carry this scope entry. That fallback (`with_roots`,
        // no `AgentDefIndex` — helmd's lean posture) resolves through
        // `space_resolver::roll_up_registered_blocking`, the SAME composed world the
        // index itself is seeded from — NOT a `.ship/agents/*.jsonc`
        // directory scan (format-migration W5 follow-up).
        let agent_in_index = if let Some(index) = self.def_index.as_ref() {
            crate::services::agents::view_state::agent_id_exists_in_index(index, request.agent)
                .await
        } else {
            crate::services::agents::view_state::agent_id_exists(&self.ship_dir, request.agent)
        };
        if !agent_in_index {
            return Err(ConnectError::invalid_argument(format!(
                "agent {:?} not found in agent index",
                request.agent
            )));
        }

        // The wire's "empty string = default" convention collapses to
        // `None` so the in-process core never sees a sentinel value.
        fn opt_str(s: &str) -> Option<String> {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        }

        // 2026-07-10). The wire carries `{agent, prompt, model, slug}`
        // — every coding-class field (spec / resume / harness /
        // invocation / keepalive / branch / source_spec_path /
        // depends_on / repo_path / base_ref) is reserved on
        // `DispatchAgentRequest` and lives on
        // `ship.code.v1.CodeDispatchRequest` instead. A coding agent is
        // simply not reachable on this surface.
        let dreq = DispatchRequest {
            agent: request.agent.to_string(),
            prompt: opt_str(request.prompt),
            model: opt_str(request.model),
            slug: opt_str(request.slug),
        };

        let outcome = dispatch_agent_session_core(self.dispatch_port.as_ref(), dreq)
            .await
            .map_err(|e| match e {
                DispatchCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                DispatchCoreError::Internal(m) => ConnectError::internal(m),
            })?;

        Ok(Response::new(DispatchResult {
            session_id: outcome.session_id,
            slug: outcome.slug,
            studio_url: outcome.studio_url,
            billing_warning: outcome.billing_warning,
            ..Default::default()
        }))
    }

    async fn stop(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedStopAgentRequestView,
    ) -> ServiceResult<StopResult> {
        (self.gate)(&ctx, "agents.stop")?;
        let result = (self.stop_fn)(request.slug.to_string()).await?;
        serde_json::from_value::<StopResult>(result)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode agents.stop response: {e}")))
    }

    async fn steer(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedSteerAgentRequestView,
    ) -> ServiceResult<SteerResult> {
        (self.gate)(&ctx, "agents.steer")?;
        let result = (self.steer_fn)(request.slug.to_string(), request.message.to_string()).await?;
        serde_json::from_value::<SteerResult>(result)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode agents.steer response: {e}")))
    }

    /// `agents.grant` — C3 operator-only authority grant.
    /// The gate closure enforces `C3Critical` / `ship.agents.grant` before the
    /// handler body executes. Non-operators are rejected at the gate. The `GrantFn`
    /// carries the two-effect domain logic: (1) upgrade live session grants in the
    /// auth session registry, (2) persist the approved baseline via
    /// `ship_auth::scopes::provision_grants`.
    async fn grant(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGrantAgentRequestView,
    ) -> ServiceResult<GrantResult> {
        // C3 gate — enforced here before the GrantFn body runs.
        (self.gate)(&ctx, "agents.grant")?;
        let grants: Vec<String> = request.grants.iter().map(|s| s.to_string()).collect();
        let result = (self.grant_fn)(request.agent.to_string(), grants, ctx).await?;
        serde_json::from_value::<GrantResult>(result)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode agents.grant response: {e}")))
    }

    // ── Watch (agents-arc E5) ─────────────────────────────────────────────────
    //
    // The ONE reactive surface replacing all deleted *Live twins. Decodes the scope
    // oneof, then sets up a snapshot-diff stream over the agent-def ReactiveRegistry.

    #[allow(refining_impl_trait)]
    async fn watch(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedAgentWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        use ship_proto::ship::ops::v1::__buffa::oneof::agent_watch_req::Scope;
        let scope = match request.to_owned_message().scope {
            Some(Scope::All(_)) => None,
            Some(Scope::AgentId(id)) => Some(id),
            None => {
                return Err(ConnectError::invalid_argument(
                    "watch scope is required (all | agent_id)",
                ));
            }
        };

        let registry =
            self.agent_reactive.as_ref().cloned().ok_or_else(|| {
                ConnectError::unavailable("agent watch unavailable: no active space")
            })?;

        // space_id is used for WatchKey cache partitioning only; fall back to empty
        // string when auth context is not injected (e.g. test environments).
        let space_id = String::new();
        let _ = &ctx; // ctx held for future auth extraction if needed

        let ship_dir = self.ship_dir.clone();
        let (scope_key, spec): (String, ResolverSpec) = match scope {
            None => {
                let sd = ship_dir.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(agent_watch::all_scope_relevant),
                    snapshot: Box::new(move || {
                        let sd = sd.clone();
                        Box::pin(async move { all_scope_snapshot(&sd) })
                    }),
                };
                (agent_watch::SCOPE_ALL.to_string(), spec)
            }
            Some(id) => {
                let sd = ship_dir.clone();
                let id_for_filter = id.clone();
                let id_for_snap = id.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(move |env| {
                        agent_watch::agent_scope_relevant(env, &id_for_filter)
                    }),
                    snapshot: Box::new(move || {
                        let sd = sd.clone();
                        let id = id_for_snap.clone();
                        Box::pin(async move { agent_scope_snapshot(&sd, &id) })
                    }),
                };
                (agent_watch::scope_agent(&id), spec)
            }
        };

        let key = WatchKey {
            space_id,
            scope: scope_key,
            query_hash: 0,
        };
        let mut rx = registry.watch(key, EventFilter::default(), spec);
        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(frame) => yield Ok(patch_frame_to_proto(frame)),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }
}
