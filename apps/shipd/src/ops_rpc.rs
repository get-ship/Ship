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

/// Daemon-side `OpsService`. Holds `ApiState` so ported ops can reach the
/// registry dispatch (`services::registry::call`).
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
pub fn router(state: ApiState) -> axum::Router {
    let svc = Arc::new(OpsImpl { state: state.clone() });
    let router = OpsServiceExt::register(svc.clone(), connectrpc::Router::new());
    let router = StreamServiceExt::register(svc.clone(), router);
    // Hand-authored proto-as-SSOT domains (impls in analytics_rpc / runtime_rpc).
    // C0 reads, edge-authenticated like the other reads.
    let router = ship_ops_rpc::AnalyticsServiceExt::register(svc.clone(), router);
    // isolation.* — capability probe + recommender reads (impl in isolation_rpc).
    let router = ship_ops_rpc::IsolationServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::RuntimeServiceExt::register(svc.clone(), router);
    // First MIGRATED domain (#89): rules.* served additively from the authored
    // rules.proto. The generated ReadOps/CommandOps still serve rules.* too,
    // until the GUI flip (#72) re-points clients and retires the old path.
    let router = ship_ops_rpc::RulesServiceExt::register(svc.clone(), router);
    // artifacts.* — typed-return migration (ArtifactRecord / Deletion), additive.
    let router = ship_ops_rpc::ArtifactsServiceExt::register(svc.clone(), router);
    // collections.* — data plane; reads opaque JSON, item writes typed. Additive.
    let router = ship_ops_rpc::CollectionsServiceExt::register(svc.clone(), router);
    // permissions.* — C2_ELEVATED writes (operator-only via the command gate). Additive.
    let router = ship_ops_rpc::PermissionsServiceExt::register(svc.clone(), router);
    // mcp.* — external MCP server config CRUD, additive.
    let router = ship_ops_rpc::McpServiceExt::register(svc.clone(), router);
    // skills.* (restricted ship-authoring writes), pipelines.* (runtime + config),
    // git.* — all additive beside the generated ReadOps/CommandOps until #72.
    let router = ship_ops_rpc::SkillsServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::PipelinesServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::GitServiceExt::register(svc.clone(), router);
    // Wave 2 (#89) — agents/agent (profiles + orchestration), config writes
    // (project/settings/annotation/feedback), and read aggregates
    // (sessions/workspaces/account/chat/dashboard/events/specs). All additive
    // beside the generated ReadOps/CommandOps until the GUI flip (#72).
    let router = ship_ops_rpc::AgentsServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::AgentServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::SpaceServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::SettingsServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::SessionsServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::AccountServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::ChatServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::EventsServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::ViewServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::RegistryServiceExt::register(svc.clone(), router);
    let router = ship_ops_rpc::SuiServiceExt::register(svc.clone(), router);
    // search — trait impl in ship-search/src/wire.rs via SearchWire<F>;
    // ShipdSearchContext is the per-request adapter in search_context.rs.
    let search_state = state.clone();
    let search_wire = std::sync::Arc::new(ship_search::SearchWire::new(
        move |_ctx: &connectrpc::RequestContext| {
            crate::search_context::ShipdSearchContext::new(search_state.clone())
        },
    ));
    let router = ship_ops_rpc::SearchServiceExt::register(search_wire, router);
    // collection.* v2 — CollectionService (collection.proto). Mounted with the
    // runtime's ONE shared SpaceContext + schema registry (C-2 §3.1 / UD-7) so
    // a wire-created schema is visible to the internal trigger/view callers.
    let collection_ctx = state.collection_ctx();
    let collection_kernel = collection_ctx.kernel.clone();
    let collection_schemas = state.collection_schemas();
    let collection_wire = std::sync::Arc::new(ship_collection::CollectionWire::with_schemas(
        collection_kernel,
        collection_schemas,
        move |_ctx: &connectrpc::RequestContext| collection_ctx.clone(),
    ));
    let router = ship_ops_rpc::CollectionServiceExt::register(collection_wire, router);
    // config.* — ConfigService (config.proto): daemon settings tier + Watch.
    // Mounted ADDITIVELY beside the legacy SettingsService until the operator's
    // sequential cutover. Reuses the collection ctx's shared kernel (settings are
    // KV-global; the wire needs the kernel for config.changed emit + Watch).
    let config_ctx = state.collection_ctx();
    let config_kernel = config_ctx.kernel.clone();
    let config_wire = std::sync::Arc::new(ship_config::ConfigWire::new(
        config_kernel,
        move |_ctx: &connectrpc::RequestContext| config_ctx.clone(),
    ));
    let router = ship_ops_rpc::ConfigServiceExt::register(config_wire, router);
    // artifacts.* v2 — ship-artifacts (DataStore + blob backend), overrides
    // OpsImpl's ArtifactsService registration for all 5 RPCs including Watch.
    let artifact_ctx = state.artifact_ctx();
    let artifact_kernel = artifact_ctx.kernel.clone();
    let global_dir = state.global_blob_dir().to_path_buf();
    let artifacts_wire = std::sync::Arc::new(ship_artifacts::ArtifactsWire::new(
        artifact_kernel,
        global_dir,
        move |_ctx: &connectrpc::RequestContext| artifact_ctx.clone(),
    ));
    let router = ship_ops_rpc::ArtifactsServiceExt::register(artifacts_wire, router);
    router.into_axum_router()
}

/// Map a `CommandError` (from `services::command::call`) to a `ConnectError`,
/// preserving the status → code mapping. Used by the generated `CommandOpsService`.
pub fn command_error_to_connect(e: crate::services::command::CommandError) -> ConnectError {
    use axum::http::StatusCode;
    match e.status {
        StatusCode::FORBIDDEN => ConnectError::permission_denied(e.message),
        StatusCode::UNAUTHORIZED => ConnectError::unauthenticated(e.message),
        s if s.is_client_error() => ConnectError::invalid_argument(e.message),
        _ => ConnectError::internal(e.message),
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
        // account.state ported as a per-op method keeps its own proto policy.
        let p = policy_for("ship.ops.v1.ReadOpsService/AccountState")
            .expect("AccountState policy must be present");
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
