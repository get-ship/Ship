//! ConnectRPC op surface — the daemon's implementation of `ship.ops.v1`
//! (`ship-ops-rpc`), mounted additively beside the REST/SSE routes. Proto owns
//! op shapes + policy; this owns the handlers and (later) gate enforcement. The
//! first op, `Meta` (C0), proves the proto → trait → mount pipeline end to end.
//! See docs/ship-architecture/references/docs/future/agent-access-and-skills-api.md.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use buffa::{Message, UnknownFieldData};
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_ops_rpc::{
    CommandOpsServiceExt, JsonResponse, MetaResponse, OpsService, OpsServiceExt,
    OwnedEventsRequestView, OwnedMetaRequestView, ReadOpsServiceExt, StreamService,
    StreamServiceExt,
};

use crate::ship_runtime::ApiState;

/// Daemon-side `OpsService`. Holds `ApiState` so ported ops can reach the
/// registry dispatch (`services::registry::call`).
pub struct OpsImpl {
    state: ApiState,
}

impl OpsImpl {
    /// Run a registry op by name and wrap its JSON projection in `JsonResponse`
    /// — the migration shape for ported read ops (see ops.proto / the spec).
    fn dispatch_json(&self, op: &str) -> ServiceResult<JsonResponse> {
        self.dispatch_json_args(op, serde_json::Value::Null)
    }

    /// As [`dispatch_json`] but with JSON `args` for ops that take parameters
    /// (the generated arg-bearing read methods build these from their request).
    fn dispatch_json_args(&self, op: &str, args: serde_json::Value) -> ServiceResult<JsonResponse> {
        let value = crate::services::registry::call(op, &args, &self.state)
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        Ok(Response::new(JsonResponse {
            json: value.to_string(),
            ..Default::default()
        }))
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

// Generated read ops (`ReadOpsService`) — projected from `schema::OPS` by
// `cargo xtask gen-ops`. Each delegates to the registry via `dispatch_json`.
// Regenerate after changing the read-op set; do not hand-edit the .rs.
include!("ops_reads_gen.rs");
// Generated commands (`CommandOpsService`) — projected from `schema::COMMANDS`
// by `cargo xtask gen-ops`; each builds args from the typed request and
// delegates to `services::command::call`. Do not hand-edit the .rs.
include!("ops_commands_gen.rs");

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
    let svc = Arc::new(OpsImpl { state });
    let router = OpsServiceExt::register(svc.clone(), connectrpc::Router::new());
    let router = ReadOpsServiceExt::register(svc.clone(), router);
    let router = CommandOpsServiceExt::register(svc.clone(), router);
    let router = StreamServiceExt::register(svc, router);
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

// Custom option field numbers — must match proto/ship/ops/v1/policy.proto.
const FIELD_CAPABILITY: u32 = 50001;
const FIELD_BLAST_RADIUS: u32 = 50002;
const FIELD_SENSITIVE: u32 = 50003;

/// Blast-radius class (mirrors `ship.ops.v1.BlastRadius`). Drives the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlastRadius {
    Unspecified,
    /// Read / own-scope; harmless to Ship.
    C0Safe,
    /// Mutates the caller's own space — the user's own risk.
    C1Workspace,
    /// Ship/cloud-dangerous; auth gate (off the agent surface).
    C2Elevated,
}

impl BlastRadius {
    fn from_proto(v: u64) -> Self {
        match v {
            1 => Self::C0Safe,
            2 => Self::C1Workspace,
            3 => Self::C2Elevated,
            _ => Self::Unspecified,
        }
    }
}

/// Policy for one op, extracted from its proto method options.
#[derive(Debug, Clone)]
pub struct OpPolicy {
    pub capability: String,
    pub blast_radius: BlastRadius,
    pub sensitive: bool,
}

/// Decode the descriptor set into a `wire-path → policy` map. Called once.
fn build_policy_map() -> HashMap<String, OpPolicy> {
    let mut map = HashMap::new();
    let set = match FileDescriptorSet::decode_from_slice(ship_ops_rpc::FILE_DESCRIPTOR_SET) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("ops gate: failed to decode descriptor set: {e}");
            return map;
        }
    };
    for file in &set.file {
        let package = file.package.as_deref().unwrap_or("");
        for svc in &file.service {
            let Some(svc_name) = svc.name.as_deref() else {
                continue;
            };
            for m in &svc.method {
                let Some(method) = m.name.as_deref() else {
                    continue;
                };
                let (mut capability, mut blast, mut sensitive) =
                    (String::new(), BlastRadius::Unspecified, false);
                if let Some(opts) = m.options.as_option() {
                    for uf in opts.__buffa_unknown_fields.iter() {
                        match (uf.number, &uf.data) {
                            (FIELD_CAPABILITY, UnknownFieldData::LengthDelimited(b)) => {
                                capability = String::from_utf8_lossy(b).into_owned();
                            }
                            (FIELD_BLAST_RADIUS, UnknownFieldData::Varint(v)) => {
                                blast = BlastRadius::from_proto(*v);
                            }
                            (FIELD_SENSITIVE, UnknownFieldData::Varint(v)) => {
                                sensitive = *v != 0;
                            }
                            _ => {}
                        }
                    }
                }
                map.insert(
                    format!("{package}.{svc_name}/{method}"),
                    OpPolicy { capability, blast_radius: blast, sensitive },
                );
            }
        }
    }
    map
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
        assert_eq!(p.blast_radius, BlastRadius::C0Safe);
        assert!(!p.sensitive);
    }

    #[test]
    fn ported_read_op_carries_its_policy() {
        // account.state ported as a per-op method keeps its own proto policy.
        let p = policy_for("ship.ops.v1.ReadOpsService/AccountState")
            .expect("AccountState policy must be present");
        assert_eq!(p.capability, "ship.account.read");
        assert_eq!(p.blast_radius, BlastRadius::C0Safe);
    }

    fn auth(kind: CallerKind, agent: Option<&str>) -> AuthContext {
        AuthContext {
            caller_kind: kind,
            agent_id: agent.map(str::to_string),
            workspace_id: None,
            session_id: None,
            scopes: vec![],
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
