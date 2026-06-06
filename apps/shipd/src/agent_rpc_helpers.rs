//! `impl OpsImpl` helpers for the canonical `AgentsService` handler
//! (`agent_rpc.rs`). Extracted to keep the handler within the file cap — this is
//! NOT a parallel handler, just the shared read/write/gate plumbing.
//!
//! Planes:
//!   * `agents_*` / `agents_get_doc` — C0 reads calling domain crates directly,
//!     no string-registry dispatch;
//!   * `agents_gated_mutation` — C2 profile writes, gate applied inline via
//!     `services::gate::decide`, then domain fn called directly;
//!   * `agent_gated_typed` — C2/C3 orchestration control, same inline gate pattern;
//!   * `agent_watch_stream` — the unified reactive `Watch` (agents-arc E5).

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{AgentDoc, JsonResponse, ScopedMutation, WatchEvent};
use ship_reactive::{ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;

use crate::auth::AuthContext;
use crate::ops_rpc::OpsImpl;
use crate::services::gate;
use ship_platform::services::agents::watch as agent_watch;

// ── Read helpers ─────────────────────────────────────────────────────────────

impl OpsImpl {
    /// C0: `agents.list` — index entries.
    pub(crate) fn agents_list(&self) -> ServiceResult<ship_ops_rpc::AgentIndexResponse> {
        use ship_platform::services::agents::state;
        use ship_platform::services::files::FilesService;
        let files = FilesService::resolve_roots(self.state().ship_dir());
        let items = state::list(&files);
        serde_json::from_value::<ship_ops_rpc::AgentIndexResponse>(json!({ "items": items }))
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("agents.list decode: {e}")))
    }

    /// C0: `agents.summaries`.
    pub(crate) fn agents_summaries(&self) -> ServiceResult<ship_ops_rpc::AgentsSummariesResponse> {
        use ship_platform::services::agents::view_state;
        let items = view_state::list_agents(self.state().ship_dir());
        serde_json::from_value::<ship_ops_rpc::AgentsSummariesResponse>(json!({ "items": items }))
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("agents.summaries decode: {e}")))
    }

    /// C0: `agents.detail`.
    pub(crate) fn agents_detail(&self, id: &str) -> ServiceResult<ship_ops_rpc::AgentDetail> {
        use ship_platform::services::agents::view_state;
        let detail = view_state::load_agent(self.state().ship_dir(), id)
            .ok_or_else(|| ConnectError::not_found(format!("agent `{id}`")))?;
        serde_json::from_value::<ship_ops_rpc::AgentDetail>(
            serde_json::to_value(detail)
                .map_err(|e| ConnectError::internal(format!("agents.detail serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.detail decode: {e}")))
    }

    /// C0: `agents.catalogs`.
    pub(crate) fn agents_catalogs(&self) -> ServiceResult<ship_ops_rpc::AgentCatalogsResponse> {
        use ship_platform::services::agents::view_state;
        let catalogs = view_state::list_agent_catalogs(self.state().ship_dir());
        serde_json::from_value::<ship_ops_rpc::AgentCatalogsResponse>(
            serde_json::to_value(catalogs)
                .map_err(|e| ConnectError::internal(format!("agents.catalogs serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.catalogs decode: {e}")))
    }

    /// C0: `agents.get` — opaque `AgentDoc`. The reader yields `{ id, scope, doc }`
    /// where `doc` is the arbitrary user def object; we keep it opaque by
    /// serializing into `doc_json` (same open-payload convention as
    /// CollectionItem.data_json).
    pub(crate) fn agents_get_doc(&self, args: Value) -> ServiceResult<AgentDoc> {
        use ship_platform::services::agents::state::{self, ScopeTag};
        use ship_platform::services::files::FilesService;
        let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let scope = parse_scope_opt(&args);
        let files = FilesService::resolve_roots(self.state().ship_dir());
        let (resolved, doc) = state::load_doc(&files, id, scope).map_err(|e| match e {
            state::Error::NotFound(m) => ConnectError::not_found(m),
            other => ConnectError::internal(other.to_string()),
        })?;
        let scope_tag = ScopeTag::from(resolved);
        let scope_str = serde_json::to_value(&scope_tag)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        Ok(Response::new(AgentDoc {
            id: id.to_string(),
            scope: scope_str,
            doc_json: doc.to_string(),
            ..Default::default()
        }))
    }

    /// C0: `agents.wire`.
    pub(crate) fn agents_wire(&self, id: &str) -> ServiceResult<ship_ops_rpc::AgentWireResponse> {
        use ship_platform::services::agents::wire;
        let ctx = wire::build(id, self.state().ship_dir())
            .ok_or_else(|| ConnectError::not_found(format!("agent `{id}`")))?;
        serde_json::from_value::<ship_ops_rpc::AgentWireResponse>(
            serde_json::to_value(ctx)
                .map_err(|e| ConnectError::internal(format!("agents.wire serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.wire decode: {e}")))
    }

    /// C0: `agents.settings`.
    pub(crate) fn agents_settings(
        &self,
        id: &str,
    ) -> ServiceResult<ship_ops_rpc::AgentSettingsResponse> {
        use ship_platform::services::agents::settings_view;
        let ctx = settings_view::build(id, self.state().ship_dir())
            .ok_or_else(|| ConnectError::not_found(format!("agent `{id}`")))?;
        serde_json::from_value::<ship_ops_rpc::AgentSettingsResponse>(
            serde_json::to_value(ctx)
                .map_err(|e| ConnectError::internal(format!("agents.settings serialize: {e}")))?,
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.settings decode: {e}")))
    }

    /// C0: `agents.templates`.
    pub(crate) fn agents_templates(
        &self,
    ) -> ServiceResult<ship_ops_rpc::AgentTemplatesResponse> {
        use ship_platform::services::agents::create;
        serde_json::from_value::<ship_ops_rpc::AgentTemplatesResponse>(
            json!({ "items": create::builtin_templates() }),
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.templates decode: {e}")))
    }

    /// C0: `agents.permission_presets`.
    pub(crate) fn agents_permission_presets(
        &self,
    ) -> ServiceResult<ship_ops_rpc::PermissionPresetsResponse> {
        use ship_platform::services::agents::create;
        serde_json::from_value::<ship_ops_rpc::PermissionPresetsResponse>(
            json!({ "items": create::builtin_permission_presets() }),
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.permission_presets decode: {e}")))
    }

    /// C0: `agents.harnesses`.
    pub(crate) fn agents_harnesses(&self) -> ServiceResult<ship_ops_rpc::HarnessesResponse> {
        use ship_platform::services::agents::create;
        serde_json::from_value::<ship_ops_rpc::HarnessesResponse>(
            json!({ "items": create::builtin_harnesses() }),
        )
        .map(Response::new)
        .map_err(|e| ConnectError::internal(format!("agents.harnesses decode: {e}")))
    }

    /// C0: `agents.create_ctx`.
    pub(crate) fn agents_create_ctx(&self) -> ServiceResult<JsonResponse> {
        use ship_platform::services::agents::create;
        let ctx = create::build_create_ctx();
        let json_str = serde_json::to_string(&ctx)
            .map_err(|e| ConnectError::internal(format!("agents.create_ctx serialize: {e}")))?;
        Ok(Response::new(JsonResponse { json: json_str, ..Default::default() }))
    }

    /// C0: `agents.sessions`.
    pub(crate) fn agents_sessions(&self) -> ServiceResult<ship_ops_rpc::SessionsListResponse> {
        let items = self.state().pty_sessions().list();
        serde_json::from_value::<ship_ops_rpc::SessionsListResponse>(json!({ "items": items }))
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("agents.sessions decode: {e}")))
    }

    /// C0 live-read: `agents.sessions` streamed — re-resolves on every `config`
    /// kernel event. Calls the domain fn directly, no string-registry in the path.
    pub(crate) async fn agents_sessions_live(
        &self,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        use ship_substrate::events::ActorConfig;

        let state = self.state().clone();
        let initial_items = state.pty_sessions().list();
        let initial = json!({ "items": initial_items });

        let actor_id = format!("live-read-{}", ship_substrate::gen_ulid());
        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: vec!["config".to_string()],
            see_directed_events: false,
        };
        let mailbox = {
            let mut kernel = state.kernel().lock().await;
            kernel
                .spawn_actor(&actor_id, config)
                .map(|(_store, mb)| mb)
                .map_err(|e| ConnectError::internal(e.to_string()))?
        };
        let guard = crate::ship_runtime::ActorGuard { actor_id, kernel: state.kernel().clone() };

        let stream = async_stream::stream! {
            let _guard = guard;
            let mut mb = mailbox;
            if let Ok(json) = serde_json::to_string(&initial) {
                yield Ok::<JsonResponse, ConnectError>(JsonResponse { json, ..Default::default() });
            }
            while let Some(_ev) = mb.recv().await {
                while mb.try_recv().is_ok() {}
                let items = state.pty_sessions().list();
                let value = json!({ "items": items });
                if let Ok(json) = serde_json::to_string(&value) {
                    yield Ok(JsonResponse { json, ..Default::default() });
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    // ── Write helpers (gate + domain call inlined) ────────────────────────────

    /// C2 profile write — gate enforced inline, then domain fn called directly.
    /// The gate logic mirrors `services::command::call` exactly: capability +
    /// blast-radius from proto (via `op_policy`), resolved service via
    /// `gate::service_of`, decision via `gate::decide`. Preserves the one-gate
    /// invariant at the handler layer.
    pub(crate) async fn agents_gated_mutation(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: Value,
    ) -> ServiceResult<ScopedMutation> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        self.agents_gate(op, auth)?;
        let result = agents_profile_write(op, &args, self.state(), auth).await?;
        serde_json::from_value::<ScopedMutation>(result)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))
    }

    /// C2/C3 orchestration control — gate enforced inline, then domain fn called
    /// directly. `agent.grant`'s C3 operator-only gate is preserved exactly.
    pub(crate) async fn agent_gated_typed<T: serde::de::DeserializeOwned>(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: Value,
    ) -> ServiceResult<T> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        self.agents_gate(op, auth)?;
        let result = agent_control_write(op, &args, self.state(), auth).await?;
        serde_json::from_value::<T>(result)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))
    }

    /// The inline gate: resolves blast-radius + service from proto, applies
    /// `gate::decide`. Mirrors `command::call`'s gate block exactly — fail closed:
    /// a known command with no resolvable blast is denied.
    fn agents_gate(&self, op: &str, auth: &AuthContext) -> Result<(), ConnectError> {
        let blast = match crate::services::op_policy::policy_for(op) {
            Some(pol) => pol.blast,
            None => {
                // Try COMMANDS fallback for ops not yet on a per-domain proto service.
                match crate::services::schema::COMMANDS.iter().find(|c| c.name == op) {
                    Some(cmd) => gate::BlastRadius::from_descriptor(cmd.blast_radius),
                    None => {
                        tracing::error!(
                            target: "ship::security::audit",
                            op = %op,
                            "known command has no resolvable blast radius — DENIED (fail closed)"
                        );
                        return Err(ConnectError::permission_denied(format!(
                            "command `{op}` has no gate policy — denied (fail closed)"
                        )));
                    }
                }
            }
        };
        let service = gate::service_of(op);
        let caller = gate::caller_from_auth(auth);
        // Security observability (S6, #124).
        if caller.is_operator
            && matches!(blast, gate::BlastRadius::C2Elevated | gate::BlastRadius::C3Critical)
        {
            tracing::warn!(
                target: "ship::security::audit",
                op = %op,
                blast = blast.as_descriptor(),
                agent_id = ?auth.agent_id,
                "elevated op (C2/C3) cleared by a verified operator credential (S6 #124)"
            );
        }
        if let gate::Decision::Deny(msg) = gate::decide(blast, service, &caller) {
            return Err(ConnectError::permission_denied(msg));
        }
        Ok(())
    }

    // ── Watch stream ──────────────────────────────────────────────────────────

    /// The unified reactive `Watch` (agents-arc E5) — open a snapshot-diff stream
    /// over the agent-def `ReactiveRegistry`. `scope` is the decoded oneof: `None`
    /// = all agents, `Some(id)` = one agent. Snapshot calls domain crates directly.
    pub(crate) fn agent_watch_stream(
        &self,
        ctx: &RequestContext,
        scope: Option<String>,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let registry = self
            .state()
            .agent_reactive()
            .cloned()
            .ok_or_else(|| ConnectError::unavailable("agent watch unavailable: no active space"))?;
        let space_id = ctx
            .extensions
            .get::<AuthContext>()
            .and_then(|a| a.space_id.clone())
            .unwrap_or_default();
        let state = self.state().clone();

        let (scope_key, spec): (String, ResolverSpec) = match scope {
            None => {
                let st = state.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(agent_watch::all_scope_relevant),
                    snapshot: Box::new(move || {
                        let st = st.clone();
                        Box::pin(async move { all_scope_snapshot(&st) })
                    }),
                };
                (agent_watch::SCOPE_ALL.to_string(), spec)
            }
            Some(id) => {
                let st = state.clone();
                let id_for_filter = id.clone();
                let id_for_snap = id.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(move |env| {
                        agent_watch::agent_scope_relevant(env, &id_for_filter)
                    }),
                    snapshot: Box::new(move || {
                        let st = st.clone();
                        let id = id_for_snap.clone();
                        Box::pin(async move { agent_scope_snapshot(&st, &id) })
                    }),
                };
                (agent_watch::scope_agent(&id), spec)
            }
        };

        let key = WatchKey { space_id, scope: scope_key, query_hash: 0 };
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

// ── Scope parse helper ────────────────────────────────────────────────────────

fn parse_scope_opt(args: &Value) -> Option<ship_platform::services::files::Scope> {
    match args.get("scope").and_then(|v| v.as_str()) {
        Some("project") => Some(ship_platform::services::files::Scope::Project),
        Some("user") => Some(ship_platform::services::files::Scope::User),
        _ => None,
    }
}

fn parse_scope_res(
    args: &Value,
    key: &str,
) -> Result<Option<ship_platform::services::files::Scope>, ConnectError> {
    match args.get(key).and_then(|v| v.as_str()) {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(ship_platform::services::files::Scope::Project)),
        Some("user") => Ok(Some(ship_platform::services::files::Scope::User)),
        Some(other) => Err(ConnectError::invalid_argument(format!(
            "`{key}` must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn req_scope(
    args: &Value,
    key: &str,
) -> Result<ship_platform::services::files::Scope, ConnectError> {
    parse_scope_res(args, key)?
        .ok_or_else(|| ConnectError::invalid_argument(format!("`{key}` is required (project|user)")))
}

fn req_str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, ConnectError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ConnectError::invalid_argument(format!("missing required arg `{key}`")))
}

fn opt_str_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string)
}

fn str_array_arg(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

// ── agents.* profile write dispatch ──────────────────────────────────────────

/// Profile write handler — the logic that was in `command/agents.rs`. Called by
/// `agents_gated_mutation` after the gate clears.
async fn agents_profile_write(
    op: &str,
    args: &Value,
    state: &crate::ship_runtime::ApiState,
    auth: &AuthContext,
) -> Result<Value, ConnectError> {
    use ship_platform::services::agents::service::{CreateArgs, DuplicateArgs, PutArgs};
    use ship_platform::services::agents::state::{self, ScopeTag};
    use ship_platform::services::files::Scope;
    use ship_substrate::ports::{KernelLog, LocalFsBroker};
    use ship_substrate::ShipHome;
    use ship_types::SpaceId;
    use std::sync::Arc;

    fn parse_scope(args: &Value, key: &str) -> Result<Option<Scope>, ConnectError> {
        parse_scope_res(args, key)
    }

    fn req_scope_local(args: &Value, key: &str) -> Result<Scope, ConnectError> {
        req_scope(args, key)
    }

    fn req_json(args: &Value, key: &str) -> Result<Value, ConnectError> {
        let raw = req_str_arg(args, key)?;
        serde_json::from_str(raw)
            .map_err(|e| ConnectError::invalid_argument(format!("`{key}` is not valid JSON: {e}")))
    }

    fn map_err(e: state::Error) -> ConnectError {
        match e {
            state::Error::NotFound(m) => ConnectError::not_found(m),
            state::Error::AlreadyExists(m) | state::Error::BadId(m) => {
                ConnectError::invalid_argument(m)
            }
            other => ConnectError::internal(other.to_string()),
        }
    }

    fn ok_val(id: &str, scope: Scope) -> Value {
        json!({ "ok": true, "id": id, "scope": ScopeTag::from(scope) })
    }

    /// Notify the reactive index after a write, deterministically (not via
    /// inotify). Mirrors `command/agents.rs::notify_saved`.
    async fn notify_saved(state: &crate::ship_runtime::ApiState, _auth: &AuthContext, id: &str, scope: Scope) {
        let (ship_dir, space) = match scope {
            Scope::Project => {
                let Some(ws) = state.workspace_dir() else { return };
                let space = SpaceId::new(ship_substrate::home::path_hash(ws));
                (ShipHome::project_ship_dir(ws), Some(space))
            }
            Scope::User => (state.ship_dir().to_path_buf(), None),
        };
        let kernel = match state.agent_def_kernel() {
            Some(shared) => shared.clone(),
            None => Arc::new(KernelLog::new(state.kernel().clone())),
        };
        let fs = LocalFsBroker::new();
        ship_platform::config::notify_agent_def_saved(&kernel, &fs, &ship_dir, id, space).await;
    }

    let agents = state.agents();
    match op {
        "agents.create" => {
            let id = req_str_arg(args, "id")?.to_string();
            let doc = req_json(args, "doc_json")?;
            let resolved = agents
                .create(CreateArgs { id: id.clone(), doc, scope: parse_scope(args, "scope")? })
                .map_err(map_err)?;
            notify_saved(state, auth, &id, resolved).await;
            Ok(ok_val(&id, resolved))
        }
        "agents.put" => {
            let id = req_str_arg(args, "id")?.to_string();
            let doc = req_json(args, "doc_json")?;
            let resolved = agents
                .put(PutArgs { id: id.clone(), doc, scope: parse_scope(args, "scope")? })
                .map_err(map_err)?;
            notify_saved(state, auth, &id, resolved).await;
            Ok(ok_val(&id, resolved))
        }
        "agents.patch" => {
            let id = req_str_arg(args, "id")?.to_string();
            let patch: json_patch::Patch =
                serde_json::from_value(req_json(args, "patch_json")?)
                    .map_err(|e| ConnectError::invalid_argument(format!("bad patch shape: {e}")))?;
            let resolved =
                agents.patch(&id, parse_scope(args, "scope")?, &patch).map_err(map_err)?;
            notify_saved(state, auth, &id, resolved).await;
            Ok(ok_val(&id, resolved))
        }
        "agents.delete" => {
            let id = req_str_arg(args, "id")?.to_string();
            let resolved = agents.delete(&id, parse_scope(args, "scope")?).map_err(map_err)?;
            notify_saved(state, auth, &id, resolved).await;
            Ok(ok_val(&id, resolved))
        }
        "agents.rename" => {
            let id = req_str_arg(args, "id")?.to_string();
            let new_id = req_str_arg(args, "new_id")?.to_string();
            let resolved =
                agents.rename(&id, &new_id, parse_scope(args, "scope")?).map_err(map_err)?;
            notify_saved(state, auth, &id, resolved).await;
            notify_saved(state, auth, &new_id, resolved).await;
            Ok(ok_val(&new_id, resolved))
        }
        "agents.duplicate" => {
            let source_id = req_str_arg(args, "id")?.to_string();
            let new_id = req_str_arg(args, "new_id")?.to_string();
            let resolved = agents
                .duplicate(DuplicateArgs {
                    source_id,
                    source_scope: parse_scope(args, "scope")?,
                    new_id: new_id.clone(),
                    to_scope: parse_scope(args, "to")?,
                })
                .map_err(map_err)?;
            notify_saved(state, auth, &new_id, resolved).await;
            Ok(ok_val(&new_id, resolved))
        }
        "agents.move_scope" => {
            let id = req_str_arg(args, "id")?.to_string();
            let from = req_scope_local(args, "from")?;
            let to = req_scope_local(args, "to")?;
            agents.move_scope(&id, from, to).map_err(map_err)?;
            notify_saved(state, auth, &id, from).await;
            notify_saved(state, auth, &id, to).await;
            Ok(ok_val(&id, to))
        }
        _ => Err(ConnectError::unimplemented(format!("unknown agents write op: {op}"))),
    }
}

// ── agent.* control write dispatch ───────────────────────────────────────────

/// Agent control handler — the logic that was in `command/agent.rs`. Called by
/// `agent_gated_typed` after the gate clears.
async fn agent_control_write(
    op: &str,
    args: &Value,
    state: &crate::ship_runtime::ApiState,
    auth: &AuthContext,
) -> Result<Value, ConnectError> {
    match op {
        // C3 — operator-only approval (already gated by agents_gate above).
        // Two effects: (1) add grants to every live session; (2) persist the
        // approved baseline so the approval survives re-dispatch.
        // See .artifacts/capability-provisioning-and-approval.md + #85.
        "agent.grant" => {
            let agent = req_str_arg(args, "agent")?.to_string();
            let new_grants = str_array_arg(args, "grants");
            let mut baseline: std::collections::BTreeSet<String> =
                ship_platform::services::permissions::projection::provisioned_grants_for(&agent)
                    .ok()
                    .flatten()
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
            let mut upgraded = 0usize;
            {
                let mut reg = state.session_registry().lock().await;
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
            ship_platform::auth::scopes::provision_grants(
                &agent,
                &provisioned,
                "operator",
                auth.agent_id.as_deref(),
            )
            .map_err(|e| ConnectError::internal(format!("failed to persist grants: {e}")))?;
            Ok(json!({
                "agent": agent,
                "grants": new_grants,
                "provisioned": provisioned,
                "sessions_upgraded": upgraded,
            }))
        }
        "agent.stop" => {
            let slug = req_str_arg(args, "slug")?.to_string();
            let dh = state
                .dispatch_handle()
                .ok_or_else(|| ConnectError::internal("dispatch service not initialized"))?;
            let handle = {
                let mut reg = dh.registry.lock().await;
                reg.take(&slug)
            }
            .ok_or_else(|| {
                ConnectError::not_found(format!("no running agent with slug '{slug}'"))
            })?;
            dh.executor
                .stop(&handle)
                .await
                .map_err(|e| ConnectError::internal(format!("failed to stop agent: {e}")))?;
            Ok(json!({ "slug": slug, "stopped": true }))
        }
        "agent.steer" => {
            let slug = req_str_arg(args, "slug")?.to_string();
            let message = req_str_arg(args, "message")?.to_string();
            let dh = state
                .dispatch_handle()
                .ok_or_else(|| ConnectError::internal("dispatch service not initialized"))?;
            let reg = dh.registry.lock().await;
            let handle = reg.get(&slug).ok_or_else(|| {
                ConnectError::not_found(format!("no running agent with slug '{slug}'"))
            })?;
            dh.executor
                .send(handle, &message)
                .await
                .map_err(|e| ConnectError::internal(format!("failed to steer agent: {e}")))?;
            Ok(json!({ "slug": slug, "steered": true }))
        }
        "agent.dispatch" => {
            use ship_substrate::events::agent_session::InvocationMode;
            let invocation = match args.get("invocation").and_then(|v| v.as_str()) {
                Some("print") => InvocationMode::Print,
                Some("stream") => InvocationMode::Stream,
                _ => InvocationMode::Pty,
            };
            let depends_on = {
                let v = str_array_arg(args, "depends_on");
                (!v.is_empty()).then_some(v)
            };
            let vars = {
                let m: std::collections::HashMap<String, String> = args
                    .get("vars")
                    .and_then(|v| v.as_object())
                    .map(|m| {
                        m.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                (!m.is_empty()).then_some(m)
            };
            let dreq = crate::routes::runtime_api::dispatch::DispatchRequest {
                agent: req_str_arg(args, "agent")?.to_string(),
                spec: opt_str_arg(args, "spec"),
                prompt: opt_str_arg(args, "prompt"),
                resume: args.get("resume").and_then(|v| v.as_bool()).unwrap_or(false),
                harness: opt_str_arg(args, "harness"),
                model: opt_str_arg(args, "model"),
                open_tab: args.get("open_tab").and_then(|v| v.as_bool()).unwrap_or(true),
                invocation,
                keepalive: args.get("keepalive").and_then(|v| v.as_bool()).unwrap_or(true),
                workspace: opt_str_arg(args, "workspace"),
                slug: opt_str_arg(args, "slug"),
                branch: opt_str_arg(args, "branch"),
                source_spec_path: opt_str_arg(args, "source_spec_path"),
                depends_on,
                vars,
            };
            let (status, axum::Json(mesh)) =
                crate::routes::runtime_api::dispatch::dispatch_agent_session_core(state, dreq)
                    .await;
            if status.is_success() {
                Ok(mesh.data)
            } else if status.is_client_error() {
                Err(ConnectError::invalid_argument(mesh.data.to_string()))
            } else {
                Err(ConnectError::internal(mesh.data.to_string()))
            }
        }
        _ => Err(ConnectError::unimplemented(format!("unknown agent control op: {op}"))),
    }
}

// ── Watch snapshot helpers ────────────────────────────────────────────────────

fn all_scope_snapshot(state: &crate::ship_runtime::ApiState) -> Value {
    json!({
        "summaries": agents_summaries_value(state),
        "catalogs": agents_catalogs_value(state),
    })
}

fn agent_scope_snapshot(state: &crate::ship_runtime::ApiState, id: &str) -> Value {
    json!({
        "detail": agents_detail_value(state, id),
        "wire": agents_wire_value(state, id),
        "settings": agents_settings_value(state, id),
    })
}

fn agents_summaries_value(state: &crate::ship_runtime::ApiState) -> Value {
    use ship_platform::services::agents::view_state;
    let items = view_state::list_agents(state.ship_dir());
    serde_json::to_value(json!({ "items": items })).unwrap_or(Value::Null)
}

fn agents_catalogs_value(state: &crate::ship_runtime::ApiState) -> Value {
    use ship_platform::services::agents::view_state;
    serde_json::to_value(view_state::list_agent_catalogs(state.ship_dir()))
        .unwrap_or(Value::Null)
}

fn agents_detail_value(state: &crate::ship_runtime::ApiState, id: &str) -> Value {
    use ship_platform::services::agents::view_state;
    view_state::load_agent(state.ship_dir(), id)
        .and_then(|d| serde_json::to_value(d).ok())
        .unwrap_or(Value::Null)
}

fn agents_wire_value(state: &crate::ship_runtime::ApiState, id: &str) -> Value {
    use ship_platform::services::agents::wire;
    wire::build(id, state.ship_dir())
        .and_then(|w| serde_json::to_value(w).ok())
        .unwrap_or(Value::Null)
}

fn agents_settings_value(state: &crate::ship_runtime::ApiState, id: &str) -> Value {
    use ship_platform::services::agents::settings_view;
    settings_view::build(id, state.ship_dir())
        .and_then(|s| serde_json::to_value(s).ok())
        .unwrap_or(Value::Null)
}

// ── Proto conversion ──────────────────────────────────────────────────────────

fn patch_op_to_proto(op: &ship_reactive::PatchOp) -> ship_ops_rpc::PatchOp {
    use ship_ops_rpc::PatchOp as P;
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

fn patch_frame_to_proto(frame: ship_reactive::PatchFrame) -> WatchEvent {
    use ship_ops_rpc::{Patch as ProtoPatch, PatchFrame as ProtoFrame};
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
        body: Some(ship_ops_rpc::__buffa::oneof::watch_event::Body::Patches(Box::new(
            ProtoFrame { patches, ..Default::default() },
        ))),
        ..Default::default()
    }
}
