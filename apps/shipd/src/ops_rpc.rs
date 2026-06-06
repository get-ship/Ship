//! ConnectRPC op surface — the daemon's implementation of `ship.ops.v1`
//! (`ship-ops-rpc`), mounted additively beside the REST/SSE routes. Proto owns
//! op shapes + policy; this owns the handlers and (later) gate enforcement. The
//! first op, `Meta` (C0), proves the proto → trait → mount pipeline end to end.
//! See docs/ship-architecture/references/docs/op-surface.md.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_ops_rpc::{
    BytesChunk, JsonResponse, MetaResponse, OpsService, OpsServiceExt,
    OwnedEventsRequestView, OwnedMetaRequestView, OwnedSessionStreamRequestView,
    StreamService, StreamServiceExt,
};

use crate::ship_runtime::ApiState;

/// Daemon-side `OpsService`. Holds `ApiState` so ported ops can reach runtime state.
pub struct OpsImpl {
    state: ApiState,
}

impl OpsImpl {
    /// Borrow the runtime — for handlers implemented in sibling modules
    /// (e.g. `analytics_rpc`, `runtime_rpc`) that need `ApiState`.
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

// ── Streaming (the Connect-native event bus; retires the SSE event route) ─────
// Mirrors runtime_api::event_stream: spawn a broadcast subscriber actor, then
// stream every kernel event as JSON. SSE was just one adapter over this.
impl StreamService for OpsImpl {
    #[allow(refining_impl_trait)]
    async fn events(
        &self,
        _ctx: RequestContext,
        _req: OwnedEventsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        use ship_substrate::events::ActorConfig;
        let actor_id = format!(
            "studio-events-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let config = ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: vec!["".to_string()],
            see_directed_events: true,
        };
        let mailbox = {
            let mut kernel = self.state.kernel().lock().await;
            kernel
                .spawn_actor(&actor_id, config)
                .map(|(_store, mb)| mb)
                .map_err(|e| ConnectError::internal(e.to_string()))?
        };
        let guard = crate::ship_runtime::ActorGuard {
            actor_id,
            kernel: self.state.kernel().clone(),
        };
        let stream = async_stream::stream! {
            let _guard = guard;
            let mut mb = mailbox;
            while let Some(event) = mb.recv().await {
                if let Ok(json) = serde_json::to_string(&event) {
                    yield Ok::<JsonResponse, ConnectError>(JsonResponse { json, ..Default::default() });
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    // Raw PTY byte stream — the Connect adapter for a session, mirroring
    // `sessions::api::io::stream_session` (the SSE route it retires): replay the
    // log snapshot, then tail the live broadcast. `bytes` is byte-exact, so the
    // CR/LF mangling that forced base64 over SSE is gone.
    #[allow(refining_impl_trait)]
    async fn session_bytes(
        &self,
        _ctx: RequestContext,
        request: OwnedSessionStreamRequestView,
    ) -> ServiceResult<ServiceStream<BytesChunk>> {
        let id = request.session_id.to_string();
        let (log_path, mut rx) = if let Some(entry) = self.state.actor_sessions().get(&id) {
            (entry.log_path(), entry.subscribe())
        } else if let Some(session) = self.state.pty_sessions().get(&id) {
            (session.0.log_path.clone(), session.subscribe())
        } else {
            return Err(ConnectError::not_found(format!("session '{id}' not found")));
        };
        let replay = tokio::fs::read(&log_path).await.unwrap_or_default();
        let stream = async_stream::stream! {
            if !replay.is_empty() {
                yield Ok::<BytesChunk, ConnectError>(BytesChunk { data: replay, ..Default::default() });
            }
            loop {
                match rx.recv().await {
                    Ok(bytes) => yield Ok(BytesChunk { data: bytes, ..Default::default() }),
                    // Slow consumer dropped frames — keep tailing rather than abort.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    // Sender gone — session ended.
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }
}

/// The ConnectRPC op surface as an `axum::Router`, merged into the daemon's
/// authenticated routes so `auth_middleware` wraps it (injecting `AuthContext`).
/// Wire paths: `/ship.ops.v1.OpsService/<Method>`.
///
/// No transport-level gate: enforcement lives at the dispatch chokepoint
/// (`services::gate::decide`, invoked by `services::command::call`), so commands
/// are gated identically on every transport — REST and Connect — rather than
/// once per surface. Reads are C0 (authenticated at the edge). This is the "one
/// gate" of `.artifacts/auth-gate-synthesis.md`.
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

pub fn router(state: ApiState) -> axum::Router {
    let svc = Arc::new(OpsImpl { state: state.clone() });
    let router = OpsServiceExt::register(svc.clone(), connectrpc::Router::new());
    let router = StreamServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::RuntimeServiceExt::register(svc.clone(), router);

    // ── Gate closure shared by all four crate-owned wires ──────────────────────
    // Reads the blast from the proto descriptor (the ONE policy source) and runs
    // gate::decide — the same enforcement that command::call did, now at the
    // transport handler for crate-owned wires (#94).
    let gate: ship_agents::services::rules::wire::GateFn = Arc::new({
        move |ctx: &RequestContext, op: &str| {
            let auth = ctx
                .extensions
                .get::<crate::auth::AuthContext>()
                .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
            let pol = crate::services::op_policy::policy_for(op)
                .ok_or_else(|| ConnectError::internal(format!("no policy for {op}")))?;
            let caller = crate::services::gate::caller_from_auth(auth);
            match crate::services::gate::decide(
                pol.blast,
                crate::services::gate::service_of(op),
                &caller,
            ) {
                crate::services::gate::Decision::Deny(m) => {
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
    // permissions.* — C2_ELEVATED writes (operator-only via the command gate).
    // Crate-owned PermissionsWire (ship-agents). Replaces impl PermissionsService for OpsImpl.
    let permissions_wire = Arc::new(ship_agents::services::permissions::PermissionsWire::new(
        state.permissions_arc(),
        gate.clone(),
    ));
    let router = ship_ops_rpc::PermissionsServiceExt::register(permissions_wire, router);
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
    // analytics.* — crate-owned AnalyticsWire (ship-analytics).
    // Replaces impl AnalyticsService for OpsImpl in analytics_rpc.rs (deleted).
    // All four ops are C0 reads; the wire reaches analytics data through
    // ship-analytics' service module (no raw DB in this file — SF-4 preserved).
    let analytics_wire = Arc::new(ship_analytics::wire::AnalyticsWire);
    let router = ship_ops_rpc::AnalyticsServiceExt::register(analytics_wire, router);
    // isolation.* — crate-owned IsolationWire (ship-isolation).
    // Replaces impl IsolationService for OpsImpl in isolation_rpc.rs (deleted).
    // All ops are C0 reads; the wire calls ship_isolation::decision / decision_ops.
    let isolation_wire = Arc::new(ship_isolation::wire::IsolationWire);
    let router = ship_ops_rpc::IsolationServiceExt::register(isolation_wire, router);
    // device.* — crate-owned DeviceWire (ship-device).
    // Replaces impl DeviceService for OpsImpl in device_rpc.rs (deleted).
    // C0 reads + C3 writes (operator-only); ports injected from shipd.
    let device_gate = gate.clone();
    let device_sessions: std::sync::Arc<dyn ship_device::WindowSessions> =
        Arc::new(crate::device_ports::OwnedRegistryWindowSessions::new(state.session_registry().clone()));
    let device_space_reg: std::sync::Arc<dyn ship_device::SpaceRegistry> =
        Arc::new(crate::device_ports::ProjectsSpaceRegistry);
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
    // git.* — crate-owned GitWire (ship-git).
    // Replaces impl GitService for OpsImpl in git_rpc.rs (deleted).
    // C0 reads + C1 writes (via command::call write_dispatch closure).
    let git_kernel = Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone()));
    let git_state_for_write = state.clone();
    let git_write: ship_git::wire::WriteFn = Arc::new(move |op, args, ctx| {
        let git = git_state_for_write.git_arc();
        Box::pin(async move {
            let _auth = ctx
                .extensions
                .get::<crate::auth::AuthContext>()
                .cloned()
                .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
            use ship_git::service::{CommitArgs, PushArgs, CheckoutArgs, BranchCreateArgs, BranchDeleteArgs};
            fn opt_str(v: &serde_json::Value, k: &str) -> Option<String> {
                v.get(k).and_then(|s| s.as_str()).filter(|s| !s.is_empty()).map(str::to_string)
            }
            fn str_list(v: &serde_json::Value, k: &str) -> Vec<String> {
                v.get(k).and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|s| s.as_str().map(str::to_string)).collect())
                    .unwrap_or_default()
            }
            let ws = opt_str(&args, "workspace_id");
            let result: Result<serde_json::Value, ship_git::state::GitError> = match op.as_str() {
                "git.stage" => {
                    let paths = str_list(&args, "paths");
                    git.stage(ws.as_deref(), &paths).map(|n| serde_json::json!({ "staged": n }))
                }
                "git.unstage" => {
                    let paths = str_list(&args, "paths");
                    git.unstage(ws.as_deref(), &paths).map(|n| serde_json::json!({ "unstaged": n }))
                }
                "git.commit" => {
                    let cargs = CommitArgs {
                        workspace_id: ws,
                        message: opt_str(&args, "message").unwrap_or_default(),
                        author: opt_str(&args, "author"),
                    };
                    git.commit(cargs).map(|r| serde_json::json!({ "sha": r.sha, "branch": r.branch }))
                }
                "git.push" => {
                    let pargs = PushArgs {
                        workspace_id: ws,
                        remote: opt_str(&args, "remote"),
                        force: args.get("force").and_then(|v| v.as_bool()).unwrap_or(false),
                    };
                    git.push(pargs).map(|s| serde_json::json!({ "output": s }))
                }
                "git.fetch" => {
                    git.fetch(ws.as_deref(), opt_str(&args, "remote").as_deref())
                        .map(|s| serde_json::json!({ "output": s }))
                }
                "git.pull" => {
                    git.pull(ws.as_deref(), opt_str(&args, "remote").as_deref())
                        .map(|s| serde_json::json!({ "output": s }))
                }
                "git.checkout" => {
                    let cargs = CheckoutArgs {
                        workspace_id: ws,
                        refname: opt_str(&args, "refname").unwrap_or_default(),
                    };
                    git.checkout(cargs).map(|s| serde_json::json!({ "output": s }))
                }
                "git.branch.create" => {
                    let cargs = BranchCreateArgs {
                        workspace_id: ws,
                        name: opt_str(&args, "name").unwrap_or_default(),
                        checkout: args.get("checkout").and_then(|v| v.as_bool()).unwrap_or(false),
                    };
                    git.branch_create(cargs).map(|s| serde_json::json!({ "output": s }))
                }
                "git.branch.delete" => {
                    let dargs = BranchDeleteArgs {
                        workspace_id: ws,
                        name: opt_str(&args, "name").unwrap_or_default(),
                        force: args.get("force").and_then(|v| v.as_bool()).unwrap_or(false),
                    };
                    git.branch_delete(dargs).map(|s| serde_json::json!({ "output": s }))
                }
                "git.init" => {
                    git.init(ws.as_deref()).map(|_| serde_json::json!({ "initialized": true }))
                }
                other => return Err(ConnectError::internal(format!("unknown git write op: {other}"))),
            };
            result.map_err(|e| ConnectError::internal(e.to_string()))
        })
    });
    let git_state_for_ann = state.clone();
    let git_annotations: ship_git::wire::AnnotationsFn =
        Arc::new(move |workspace_id: Option<&str>, file: &str| {
            crate::git_rpc::git_annotations_value(&git_state_for_ann, workspace_id, file)
        });
    let git_wire = Arc::new(ship_git::wire::GitWire::new(
        state.git_arc(),
        gate.clone(),
        git_kernel,
        git_write,
        git_annotations,
    ));
    let router = ship_ops_rpc::GitServiceExt::register(git_wire, router);
    // pipelines.* — crate-owned PipelinesWire (ship-pipeline-runtime).
    // Replaces `impl PipelinesService for OpsImpl` in pipelines_rpc.rs (deleted).
    // Reads (C0): list/get call the crate service directly; summaries/run/history
    // go through the read_fn closure (shipd view helpers). Writes (C1): all config
    // + runtime writes go through the write_fn closure (shipd route cores + gated
    // command calls). Live-read streams go through the live_read_fn closure (shipd
    // kernel subscriber loop).
    let pipelines_state_for_read = state.clone();
    let pipelines_read_fn: ship_pipeline_runtime::wire::ReadFn = Arc::new(move |op, args| {
        match op {
            "pipelines.summaries" => {
                use crate::services::pipelines::summaries;
                use crate::views::state::Filter;
                let filter = Filter {
                    agent: args.get("agent").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty()).map(str::to_string),
                    status: args.get("status").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty()).map(str::to_string),
                    pipeline: args.get("pipeline").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty()).map(str::to_string),
                    q: args.get("q").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty()).map(str::to_string),
                };
                let active_tab = args.get("active_tab").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(str::to_string);
                let ctx = summaries::build(pipelines_state_for_read.ship_dir(), filter, active_tab);
                serde_json::to_value(&ctx).unwrap_or(serde_json::Value::Null)
            }
            "pipelines.run" => {
                use crate::services::pipelines::run_detail;
                let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                let ctx = run_detail::build(slug, pipelines_state_for_read.ship_dir());
                serde_json::to_value(&ctx).unwrap_or(serde_json::Value::Null)
            }
            "pipelines.history" => {
                use crate::services::pipelines::history;
                let q = args.get("q").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(str::to_string);
                let pipeline = args.get("pipeline").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(str::to_string);
                let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                let ctx = history::build(pipelines_state_for_read.ship_dir(), q, pipeline, offset, limit);
                serde_json::to_value(&ctx).unwrap_or(serde_json::Value::Null)
            }
            _ => serde_json::Value::Null,
        }
    });
    let pipelines_state_for_live = state.clone();
    let pipelines_svc_for_live = svc.clone();
    let pipelines_live_fn: ship_pipeline_runtime::wire::LiveReadFn = Arc::new(move |op, args| {
        let st = pipelines_state_for_live.clone();
        let svc = pipelines_svc_for_live.clone();
        Box::pin(async move {
            // Build a temporary OpsImpl-equivalent live_read using the kernel.
            use connectrpc::{ConnectError, Response, ServiceResult, ServiceStream};
            use ship_substrate::events::ActorConfig;
            use ship_ops_rpc::JsonResponse;
            let resolve: Box<dyn Fn(&crate::ship_runtime::ApiState) -> serde_json::Value + Send + Sync> = match op {
                "pipelines.summaries" => {
                    Box::new(move |st| {
                        use crate::services::pipelines::summaries;
                        use crate::views::state::Filter;
                        let ctx = summaries::build(st.ship_dir(), Filter::default(), None);
                        serde_json::to_value(&ctx).unwrap_or(serde_json::Value::Null)
                    })
                }
                "pipelines.run" => {
                    let slug = args.get("slug").and_then(|v| v.as_str())
                        .unwrap_or("").to_string();
                    Box::new(move |st| {
                        use crate::services::pipelines::run_detail;
                        let ctx = run_detail::build(&slug, st.ship_dir());
                        serde_json::to_value(&ctx).unwrap_or(serde_json::Value::Null)
                    })
                }
                "pipelines.history" => {
                    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                    let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let pipeline = args.get("pipeline").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty()).map(str::to_string);
                    let q = args.get("q").and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty()).map(str::to_string);
                    Box::new(move |st| {
                        use crate::services::pipelines::history;
                        let ctx = history::build(st.ship_dir(), q.clone(), pipeline.clone(), offset, limit);
                        serde_json::to_value(&ctx).unwrap_or(serde_json::Value::Null)
                    })
                }
                _ => Box::new(|_| serde_json::Value::Null),
            };
            let initial = resolve(&st);
            let actor_id = format!("live-read-{}", ship_substrate::gen_ulid());
            let config = ActorConfig {
                namespace: actor_id.clone(),
                write_namespaces: vec![],
                read_namespaces: vec![],
                subscribe_namespaces: vec!["config".to_string()],
                see_directed_events: false,
            };
            let mailbox = {
                let mut kernel = st.kernel().lock().await;
                kernel
                    .spawn_actor(&actor_id, config)
                    .map(|(_store, mb)| mb)
                    .map_err(|e| ConnectError::internal(e.to_string()))?
            };
            let guard = crate::ship_runtime::ActorGuard { actor_id, kernel: st.kernel().clone() };
            let stream = async_stream::stream! {
                let _guard = guard;
                let mut mb = mailbox;
                if let Ok(json) = serde_json::to_string(&initial) {
                    yield Ok::<JsonResponse, ConnectError>(JsonResponse { json, ..Default::default() });
                }
                while let Some(_ev) = mb.recv().await {
                    while mb.try_recv().is_ok() {}
                    let value = resolve(&st);
                    if let Ok(json) = serde_json::to_string(&value) {
                        yield Ok(JsonResponse { json, ..Default::default() });
                    }
                }
            };
            Ok(Response::new(Box::pin(stream) as ServiceStream<JsonResponse>))
        })
    });
    let pipelines_state_for_write = state.clone();
    let pipelines_write_fn: ship_pipeline_runtime::wire::WriteFn =
        Arc::new(move |op: String, args: serde_json::Value, ctx: connectrpc::RequestContext| {
            let st = pipelines_state_for_write.clone();
            Box::pin(async move {
                use axum::response::IntoResponse;
                let auth = ctx
                    .extensions
                    .get::<crate::auth::AuthContext>()
                    .cloned()
                    .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
                match op.as_str() {
                    "pipelines.dispatch" => {
                        use ship_pipeline_runtime::run_core::{PipelineRunRequest, RunPipelineCoreError, run_pipeline_core};
                        use crate::pipeline_ports::PipelineDispatchAdapter;
                        let opt = |k: &str| {
                            args.get(k).and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty())
                                .map(str::to_string)
                        };
                        let vars: std::collections::HashMap<String, String> = args
                            .get("vars")
                            .and_then(|v| v.as_object())
                            .map(|m| {
                                m.iter()
                                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let req = PipelineRunRequest {
                            pipeline: args.get("pipeline").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            spec: opt("spec"),
                            vars: if vars.is_empty() { None } else { Some(vars) },
                            harness: opt("harness"),
                            model: opt("model"),
                            workspace: opt("workspace"),
                            slug: opt("slug"),
                            auto: args.get("auto").and_then(|v| v.as_bool()).unwrap_or(false),
                            source_spec_path: opt("source_spec_path"),
                        };
                        // Scope gate: require CreateRuns (same as the HTTP handler).
                        use crate::auth::Scope;
                        let _ = &auth; // scope check via auth fields
                        if !auth.scopes.iter().any(|s| matches!(s, Scope::Full | Scope::CreateRuns)) {
                            return Err(ConnectError::permission_denied("requires CreateRuns scope"));
                        }
                        let port = PipelineDispatchAdapter::new(st.clone());
                        run_pipeline_core(&port, req).await
                            .map(|outcome| serde_json::json!({
                                "pipeline_run_id": outcome.pipeline_run_id,
                                "slug": outcome.slug,
                                "branch": outcome.branch,
                                "resume": outcome.resume,
                            }))
                            .map_err(|e| match e {
                                RunPipelineCoreError::NotFound(m) => ConnectError::not_found(m),
                                RunPipelineCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                RunPipelineCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    "pipelines.end_turn" => {
                        use ship_pipeline_runtime::run_core::{EndTurnCoreError, EndTurnRequest, end_turn_core};
                        use crate::pipeline_ports::{PipelineDispatchAdapter, TurnSignalAdapter};
                        let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let dispatch_port = PipelineDispatchAdapter::new(st.clone());
                        let signal_port = TurnSignalAdapter::new(st.clone());
                        end_turn_core(&dispatch_port, &signal_port, EndTurnRequest { slug })
                            .await
                            .map(|msg| serde_json::json!(msg))
                            .map_err(|e| match e {
                                EndTurnCoreError::NotFound(m) => ConnectError::not_found(m),
                                EndTurnCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                EndTurnCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    "pipeline.update_status" => {
                        let pipeline_run_id = args.get("pipeline_run_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let body = crate::project_api::RunStatusReq {
                            status: args.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            worktree: args.get("worktree").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            pid: args.get("pid").and_then(|v| v.as_u64()).map(|n| n as u32),
                            gate_agent: args.get("gate_agent").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            blocker: args.get("blocker").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            error: args.get("error").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                        };
                        crate::project_api::jobs_status::update_job_status_core(&st, &pipeline_run_id, body)
                            .await
                            .map_err(|(status, msg)| {
                                use axum::http::StatusCode;
                                if status == StatusCode::NOT_FOUND {
                                    ConnectError::not_found(msg)
                                } else if status.is_client_error() {
                                    ConnectError::invalid_argument(msg)
                                } else {
                                    ConnectError::internal(msg)
                                }
                            })
                    }
                    "pipeline.step_done" => {
                        let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let outputs: std::collections::BTreeMap<String, String> =
                            args.get("outputs").and_then(|v| v.as_object())
                                .map(|m| {
                                    m.iter()
                                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                        .collect()
                                })
                                .unwrap_or_default();
                        let body = crate::project_api::StepDoneReq {
                            summary: args.get("summary").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            outputs,
                        };
                        crate::project_api::jobs_status::step_done_core(&st, &slug, body)
                            .await
                            .map_err(|(status, msg)| {
                                use axum::http::StatusCode;
                                if status == StatusCode::NOT_FOUND {
                                    ConnectError::not_found(msg)
                                } else if status.is_client_error() {
                                    ConnectError::invalid_argument(msg)
                                } else {
                                    ConnectError::internal(msg)
                                }
                            })
                    }
                    _ => Err(ConnectError::internal(format!("unknown pipelines write op: {op}"))),
                }
            })
        });
    let pipelines_wire = Arc::new(ship_pipeline_runtime::wire::PipelinesWire::new(
        state.pipelines_arc(),
        gate.clone(),
        pipelines_read_fn,
        pipelines_live_fn,
        pipelines_write_fn,
    ));
    let router = ship_ops_rpc::PipelinesServiceExt::register(pipelines_wire, router);
    // agents.* + agent.* — crate-owned AgentsWire (ship-agents).
    // Replaces `impl AgentsService for OpsImpl` in agent_rpc.rs (deleted).
    // All 23 methods: profile reads (C0) + runtime reads (C0) + profile writes (C2)
    // + orchestration control (C2/C3) + Watch stream. agent.grant keeps its C3 gate.
    let agents_kernel = Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone()));
    let agents_sessions_state = state.clone();
    // SessionsFn returns the raw items list as a Value — the wire wraps it in {"items": ...}.
    let agents_sessions_fn: ship_agents::services::agents::rpc_wire::SessionsFn =
        Arc::new(move || {
            let items = agents_sessions_state.pty_sessions().list();
            serde_json::to_value(items).unwrap_or(serde_json::Value::Array(vec![]))
        });
    let notify_saved_state = state.clone();
    let notify_saved_fn: ship_agents::services::agents::rpc_wire::NotifySavedFn =
        Arc::new(move |id: String, scope_str: String| {
            let st = notify_saved_state.clone();
            Box::pin(async move {
                use ship_files::files::Scope;
                use ship_substrate::ShipHome;
                use ship_types::SpaceId;
                use ship_substrate::ports::{KernelLog, LocalFsBroker};
                let scope = match scope_str.as_str() {
                    "project" => Scope::Project,
                    _ => Scope::User,
                };
                let (ship_dir, space) = match scope {
                    Scope::Project => {
                        let Some(ws) = st.workspace_dir() else { return };
                        let space = SpaceId::new(ship_substrate::home::path_hash(ws));
                        (ShipHome::project_ship_dir(ws), Some(space))
                    }
                    Scope::User => (st.ship_dir().to_path_buf(), None),
                };
                let kernel = match st.agent_def_kernel() {
                    Some(shared) => shared.clone(),
                    None => Arc::new(KernelLog::new(st.kernel().clone())),
                };
                let fs = LocalFsBroker::new();
                ship_platform::config::notify_agent_def_saved(
                    &kernel, &fs, &ship_dir, &id, space,
                )
                .await;
            })
        });
    let dispatch_port: std::sync::Arc<dyn ship_agents::services::agents::dispatch_core::AgentDispatchPort> =
        Arc::new(crate::agent_event_ports::AgentDispatchAdapter::new(state.clone()));
    let stop_state = state.clone();
    let stop_fn: ship_agents::services::agents::rpc_wire::StopFn =
        Arc::new(move |slug: String| {
            let st = stop_state.clone();
            Box::pin(async move {
                let dh = st.dispatch_handle()
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
                Ok(serde_json::json!({ "slug": slug, "stopped": true }))
            })
        });
    let steer_state = state.clone();
    let steer_fn: ship_agents::services::agents::rpc_wire::SteerFn =
        Arc::new(move |slug: String, message: String| {
            let st = steer_state.clone();
            Box::pin(async move {
                let dh = st.dispatch_handle()
                    .ok_or_else(|| ConnectError::internal("dispatch service not initialized"))?;
                let reg = dh.registry.lock().await;
                let handle = reg.get(&slug).ok_or_else(|| {
                    ConnectError::not_found(format!("no running agent with slug '{slug}'"))
                })?;
                dh.executor
                    .send(handle, &message)
                    .await
                    .map_err(|e| ConnectError::internal(format!("failed to steer agent: {e}")))?;
                Ok(serde_json::json!({ "slug": slug, "steered": true }))
            })
        });
    let grant_state = state.clone();
    let grant_fn: ship_agents::services::agents::rpc_wire::GrantFn =
        Arc::new(move |agent: String, new_grants: Vec<String>, ctx: connectrpc::RequestContext| {
            let st = grant_state.clone();
            Box::pin(async move {
                // C3 gate already enforced by the AgentsWire gate closure before this runs.
                // Extract approver agent_id from the auth context injected by the middleware.
                let auth = ctx
                    .extensions
                    .get::<crate::auth::AuthContext>()
                    .cloned()
                    .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
                let mut baseline: std::collections::BTreeSet<String> =
                    ship_agents::services::permissions::projection::provisioned_grants_for(&agent)
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                        .into_iter()
                        .collect();
                let mut upgraded = 0usize;
                {
                    let mut reg = st.session_registry().lock().await;
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
                    auth.agent_id.as_deref(),
                )
                .map_err(|e| ConnectError::internal(format!("failed to persist grants: {e}")))?;
                Ok(serde_json::json!({
                    "agent": agent,
                    "grants": new_grants,
                    "provisioned": provisioned,
                    "sessions_upgraded": upgraded,
                }))
            })
        });
    let feedback_auth = Arc::new(crate::agent_event_ports::FeedbackAuthAdapter);
    let view_broadcast = Arc::new(crate::agent_event_ports::ViewBroadcastAdapter::new(state.clone()));
    let collection = Arc::new(crate::agent_event_ports::CollectionPersistAdapter::new(state.clone()));
    let agents_wire = Arc::new(ship_agents::services::agents::AgentsWire::new(
        state.agents_arc(),
        state.ship_dir().to_path_buf(),
        agents_kernel,
        gate,
        agents_sessions_fn,
        notify_saved_fn,
        dispatch_port,
        stop_fn,
        steer_fn,
        grant_fn,
        state.agent_reactive().cloned(),
        feedback_auth,
        view_broadcast,
        collection,
    ));
    let router = ship_ops_rpc::AgentsServiceExt::register(agents_wire, router);
    // account.* — typed AccountService (account.proto), served by the
    // `ship-account` crate (`AccountWire`). Process-global identity: token at
    // rest under `<daemon_dir>/account.jsonc`, `account.*` + `daemon.owner.*`
    // audit ride a fresh `KernelLog` over the shared kernel (Account-scoped →
    // account.db). If the gateway client fails to build, the mount is skipped
    // and the four account RPCs return Unimplemented.
    let router = match crate::account_rpc::build_account_wire(
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
    // view.* — crate-owned ViewWire (ship-views).
    // Replaces `impl ViewService for OpsImpl` in view_rpc.rs (deleted).
    // Session writes (patch/append/signal) capture view_sessions; edit-mode
    // grant/revoke call ship_views::view_authoring directly (no daemon state);
    // tool dispatch captures ApiState via the tool_fn closure.
    let view_sessions_for_patch = state.view_sessions().clone();
    let view_patch_fn: ship_views::wire::PatchFn =
        Arc::new(move |id, agent, selector, html| {
            let vs = view_sessions_for_patch.clone();
            Box::pin(async move {
                vs.patch(&id, &agent, &selector, html)
                    .await
                    .map_err(|e| ConnectError::internal(e.to_string()))
            })
        });
    let view_sessions_for_append = state.view_sessions().clone();
    let view_append_fn: ship_views::wire::AppendFn =
        Arc::new(move |id, agent, selector, html| {
            let vs = view_sessions_for_append.clone();
            Box::pin(async move {
                vs.append(&id, &agent, &selector, html)
                    .await
                    .map_err(|e| ConnectError::internal(e.to_string()))
            })
        });
    let view_sessions_for_signal = state.view_sessions().clone();
    let view_signal_fn: ship_views::wire::SignalFn =
        Arc::new(move |id, agent, name, value| {
            let vs = view_sessions_for_signal.clone();
            Box::pin(async move {
                vs.signal(&id, &agent, &name, value)
                    .await
                    .map_err(|e| ConnectError::internal(e.to_string()))
            })
        });
    let view_state_for_tool = state.clone();
    let view_tool_fn: ship_views::wire::ToolFn =
        Arc::new(move |view, session_id, op, args| {
            let st = view_state_for_tool.clone();
            Box::pin(async move {
                let body = crate::routes::view_edit_tool_api::ToolBody {
                    session_id,
                    op,
                    args,
                };
                let resp = crate::routes::view_edit_tool_api::dispatch_tool_core(
                    &st, view, body,
                )
                .await;
                let status = resp.status();
                let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
                    .await
                    .map_err(|e| ConnectError::internal(e.to_string()))?;
                if status.is_success() {
                    Ok(String::from_utf8_lossy(&bytes).into_owned())
                } else {
                    let msg = String::from_utf8_lossy(&bytes).into_owned();
                    if status.is_client_error() {
                        Err(ConnectError::invalid_argument(msg))
                    } else {
                        Err(ConnectError::internal(msg))
                    }
                }
            })
        });
    let view_agent_fn: ship_views::wire::AgentFn =
        Arc::new(|ctx: &connectrpc::RequestContext| {
            ctx.extensions
                .get::<crate::auth::AuthContext>()
                .and_then(|a| a.agent_id.clone())
                .ok_or_else(|| ConnectError::permission_denied("agent identity required"))
        });
    // scaffold_fn: moves scaffold_view route-core into the ViewWire.
    // Runs synchronous fs I/O in spawn_blocking so the future is Send.
    let scaffold_ship_dir = state.ship_dir().to_path_buf();
    let view_scaffold_fn: ship_views::wire::ScaffoldViewFn =
        Arc::new(move |name: String, shape: String, purpose: Option<String>| {
            let ship_dir = scaffold_ship_dir.clone();
            Box::pin(async move {
                tokio::task::spawn_blocking(move || {
                    scaffold_view_sync(&ship_dir, &name, &shape, purpose.as_deref())
                })
                .await
                .map_err(|e| ConnectError::internal(format!("scaffold task: {e}")))?
            })
        });
    let view_wire = Arc::new(ship_views::wire::ViewWire::new(
        view_agent_fn,
        view_patch_fn,
        view_append_fn,
        view_signal_fn,
        view_tool_fn,
        view_scaffold_fn,
    ));
    let router = ship_ops_rpc::ViewServiceExt::register(view_wire, router);
    let router = ship_ops_rpc::RegistryServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::SuiServiceExt::register(svc.clone(), router);
    // search — ShipdSearchWire (search_rpc.rs) branches on the request scope:
    // THIS_SPACE runs the active-space ShipdSearchContext (the historical
    // single-context path); GLOBAL fans out across reachable spaces via
    // ship_search::run_global. Composition + ranking + merge stay in the
    // scope-blind ship-search crate.
    let search_wire =
        std::sync::Arc::new(crate::search_rpc::ShipdSearchWire::new(state.clone()));
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
    let settings_wire = crate::settings_rpc::settings_wire(state.clone());
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
    let space_wire = crate::space_rpc::space_wire(state.clone());
    let router = ship_ops_rpc::SpaceServiceExt::register(space_wire, router);
    router.into_axum_router()
}

// ── scaffold_view_sync ────────────────────────────────────────────────────────
// Synchronous scaffold logic for ViewService::Scaffold. Moved from
// routes/create_api.rs::scaffold_view; runs in spawn_blocking from the closure.
fn scaffold_view_sync(
    ship_dir: &std::path::Path,
    name: &str,
    shape: &str,
    _purpose: Option<&str>,
) -> Result<(String, String), ConnectError> {
    const KEBAB: &str = "abcdefghijklmnopqrstuvwxyz0123456789-";
    let valid = !name.is_empty()
        && name.len() <= 48
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name.chars().all(|c| KEBAB.contains(c));
    if !valid {
        return Err(ConnectError::invalid_argument("name must be kebab-case (a-z0-9-, max 48 chars)"));
    }

    let scaffold_rel = std::path::PathBuf::from("skills")
        .join("ship-views")
        .join("scaffolds")
        .join(shape);
    let mut scaffold_src: Option<std::path::PathBuf> = None;
    if let Ok(d) = ship_substrate::project::get_project_dir(None) {
        let p = d.join(&scaffold_rel);
        if p.is_dir() { scaffold_src = Some(p); }
    }
    if scaffold_src.is_none() {
        let p = ship_dir.join(&scaffold_rel);
        if p.is_dir() { scaffold_src = Some(p); }
    }
    let src = scaffold_src.ok_or_else(|| {
        ConnectError::invalid_argument(format!("unknown shape '{shape}'"))
    })?;

    let views_dir = ship_dir.join("views");
    std::fs::create_dir_all(&views_dir)
        .map_err(|e| ConnectError::internal(format!("create views dir: {e}")))?;
    let target = views_dir.join(name);
    if target.exists() {
        return Err(ConnectError::already_exists(format!("view '{name}' already exists")));
    }

    let title = title_case_view(name);
    copy_scaffold_view(&src, &target, &title, name)
        .map_err(|e| { let _ = std::fs::remove_dir_all(&target); e })?;

    let route = format!("/api/v1/v/{name}");
    let path = target.display().to_string();
    Ok((route, path))
}

fn title_case_view(name: &str) -> String {
    name.split('-')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn copy_scaffold_view(
    src: &std::path::Path,
    dst: &std::path::Path,
    title: &str,
    name: &str,
) -> Result<(), ConnectError> {
    std::fs::create_dir_all(dst)
        .map_err(|e| ConnectError::internal(format!("mkdir: {e}")))?;
    for entry in std::fs::read_dir(src)
        .map_err(|e| ConnectError::internal(format!("read src dir: {e}")))?
    {
        let entry = entry.map_err(|e| ConnectError::internal(format!("dir entry: {e}")))?;
        let ty = entry.file_type().map_err(|e| ConnectError::internal(e.to_string()))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_scaffold_view(&from, &to, title, name)?;
        } else if ty.is_file() {
            let body = std::fs::read_to_string(&from).unwrap_or_default();
            let substituted = body.replace("{{TITLE}}", title).replace("{{NAME}}", name);
            std::fs::write(&to, substituted)
                .map_err(|e| ConnectError::internal(format!("write file: {e}")))?;
        }
    }
    Ok(())
}

/// Map an Axum `Response` body to a `serde_json::Value`, converting HTTP errors
/// to `ConnectError`. Used by the pipelines write closure (replaces the deleted
/// `services::command::response_to_value` + `command_error_to_connect` pair).
async fn axum_response_to_value(
    resp: axum::response::Response,
) -> Result<serde_json::Value, ConnectError> {
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 4 << 20)
        .await
        .map_err(|e| ConnectError::internal(e.to_string()))?;
    if status.is_success() {
        serde_json::from_slice(&bytes).map_err(|e| ConnectError::internal(e.to_string()))
    } else {
        let msg = String::from_utf8_lossy(&bytes).into_owned();
        if status == axum::http::StatusCode::FORBIDDEN {
            Err(ConnectError::permission_denied(msg))
        } else if status == axum::http::StatusCode::UNAUTHORIZED {
            Err(ConnectError::unauthenticated(msg))
        } else if status.is_client_error() {
            Err(ConnectError::invalid_argument(msg))
        } else {
            Err(ConnectError::internal(msg))
        }
    }
}

// ── Policy introspection (not the enforcement path) ───────────────────────────
//
// Enforcement is `services::gate` via `command::call`. The map below reads the
// capability / blast-radius / sensitive method options that `gen-ops` projects
// into the proto `FileDescriptorSet` — kept so the contract is visible to every
// generated client (TS/docs/SDK) and conformance-tested here, NOT to gate calls.
//
// Proto owns op policy (#67a): the capability / blast-radius / sensitive method
// options are compiled into the FileDescriptorSet. We read them once at first
// use rather than from generated Rust (the runtime drops custom options in
// normal codegen). Keyed by the Connect wire-path body "<package>.<Svc>/<Method>".

/// Policy for one op, extracted from its proto method options. `blast_radius` is
/// the shared `ship_ops_rpc::BlastRadius` (one enum across the gate + this map).
#[derive(Debug, Clone)]
pub struct OpPolicy {
    pub capability: String,
    pub blast_radius: ship_ops_rpc::BlastRadius,
    pub sensitive: bool,
}

/// Build the `wire-path → policy` map. Delegates to the single crate-level
/// descriptor reader (`ship_ops_rpc::policy::read_policies`) — one decode of the
/// FileDescriptorSet, shared with the gate's op-name-keyed lookup. Called once.
fn build_policy_map() -> HashMap<String, OpPolicy> {
    ship_ops_rpc::policy::read_policies()
        .into_iter()
        .map(|p| {
            (
                p.wire_path(),
                OpPolicy {
                    capability: p.capability,
                    blast_radius: p.blast,
                    sensitive: p.sensitive,
                },
            )
        })
        .collect()
}

/// The op-policy map, built once from the descriptor at first lookup.
pub static OP_POLICY: LazyLock<HashMap<String, OpPolicy>> = LazyLock::new(build_policy_map);

/// Look up policy by Connect wire-path, e.g. `ship.ops.v1.OpsService/Meta`
/// (leading slash optional).
pub fn policy_for(path: &str) -> Option<&'static OpPolicy> {
    OP_POLICY.get(path.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthContext;
    use ship_substrate::events::CallerKind;

    #[test]
    fn meta_op_policy_read_from_proto_descriptor() {
        // Proves #67a end to end: the method options declared in ops.proto are
        // compiled into the descriptor and recovered by the gate.
        let p = policy_for("ship.ops.v1.OpsService/Meta")
            .expect("Meta policy must be present in the descriptor");
        assert_eq!(p.capability, "ship.meta.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
        assert!(!p.sensitive);
    }

    #[test]
    fn ported_read_op_carries_its_policy() {
        // account.state keeps its own proto policy on the canonical per-domain
        // service. The `ReadOpsService` umbrella was retired (#78) — per-domain
        // Connect is the SSOT, so the policy resolves via `AccountService/State`.
        let p = policy_for("ship.ops.v1.AccountService/State")
            .expect("AccountService/State policy must be present");
        assert_eq!(p.capability, "ship.account.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }

    fn auth(kind: CallerKind, agent: Option<&str>) -> AuthContext {
        AuthContext {
            caller_kind: kind,
            agent_id: agent.map(str::to_string),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![],
            grants: vec![],
            // Meta is C0 (a read) — no operator authority needed. The CLI
            // transport grants no elevation (S6), so this stays false.
            operator_cred: false,
        }
    }

    // Gate enforcement moved to `services::gate` (invoked by `command::call`);
    // its decision is unit-tested there. This module keeps only the proto-policy
    // introspection + projection conformance above.

    // Full vertical: serve the real router (with test ApiState) on an ephemeral
    // port and call Meta through the generated client. Multi-thread flavor so
    // any block_in_place in ported ops is safe.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn meta_round_trip_through_generated_client() {
        use connectrpc::client::{ClientConfig, HttpClient};
        use ship_ops_rpc::{MetaRequest, OpsServiceClient};

        let harness = crate::test_harness::TestDaemon::new().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        // Inject an AuthContext the way the daemon's auth_middleware does (the
        // gate requires one); a human-operator credential clears any class.
        let app = router(harness.state.clone()).layer(axum::middleware::from_fn(
            |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
                req.extensions_mut().insert(auth(CallerKind::Cli, None));
                next.run(req).await
            },
        ));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let uri: http::Uri = format!("http://{addr}").parse().unwrap();
        let client = OpsServiceClient::new(HttpClient::plaintext(), ClientConfig::new(uri));
        let resp = client
            .meta(MetaRequest::default())
            .await
            .expect("Meta RPC should succeed over the wire");
        assert_eq!(resp.view().version, env!("CARGO_PKG_VERSION"));
    }

    // The update_job_status command and the REST handler share one core; this
    // exercises that core (refactor preserved behavior + it's transport-neutral).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn update_job_status_core_is_shared_and_emits() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let body = crate::project_api::jobs_status::RunStatusReq {
            status: "gate_passed".to_string(),
            worktree: None,
            pid: None,
            gate_agent: None,
            blocker: None,
            error: None,
        };
        let v = crate::project_api::jobs_status::update_job_status_core(
            &harness.state,
            "run-test",
            body,
        )
        .await
        .expect("gate_passed has no required fields");
        assert_eq!(v["pipeline_run_id"], "run-test");
        assert_eq!(v["status"], "gate_passed");
    }
}
