//! Conformance + vertical tests for the ops router: proto-policy
//! introspection, the in-process gate, and a full Meta round trip.

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
    assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C3Critical);
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

// Gate enforcement moved to `auth::gate` (invoked by the `GateFn` closure);
// its decision is unit-tested there. This module keeps only the proto-policy
// introspection + projection conformance above.

/// **Phase-B verification — RPC-as-IPC carries the gate.** The embedded agent
/// reaches ops not over a socket but via `tower::oneshot` of this same Connect
/// router, in-process. The load-bearing assumption that gates all trunk work:
/// that in-process path runs the *identical* capability gate the socket path
/// does, reading the caller's `AuthContext` from the request extensions.
///
/// This proves it behaviorally. We oneshot the real `router(state)` — no TCP,
/// no generated client — injecting an `AuthContext` into the request
/// extensions exactly as the daemon's `auth_middleware` stamps it in
/// production (and which connectrpc moves verbatim into
/// `RequestContext.extensions`, the map the `GateFn` reads). For an
/// embedded-style caller (`operator_cred = false`, scoped grants) we flip a
/// single variable — the grant — and watch the verdict on one C2 op flip with
/// it: ungranted `mcp.create` is denied; the *same* request with the `mcp`
/// service granted clears the gate. A C0 read by the same ungranted caller
/// stays open, so the denial is C2-and-grant-specific, not a dead transport.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn in_process_router_enforces_the_gate_for_an_embedded_caller() {
    use axum::body::Body;
    use http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let harness = crate::test_harness::TestDaemon::new().await;
    let app = router(harness.state.clone());

    // POST a Connect-JSON unary request through the in-process transport with
    // `caller` injected into the request extensions (the production seam).
    // Returns (http status, connect error code — "" on success).
    async fn call_op(
        app: &axum::Router,
        caller: AuthContext,
        wire_path: &str,
        body: serde_json::Value,
    ) -> (u16, String) {
        let req = Request::builder()
            .method("POST")
            .uri(format!("/{wire_path}"))
            .header("content-type", "application/json")
            .extension(caller)
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status().as_u16();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let code = serde_json::from_slice::<serde_json::Value>(&bytes)
            .ok()
            .and_then(|v| v.get("code").and_then(|c| c.as_str()).map(str::to_string))
            .unwrap_or_default();
        (status, code)
    }

    // An embedded agent: MCP-kind, NOT the operator, no elevated services.
    let bare = auth(CallerKind::Mcp, Some("agent-x"));
    assert!(!bare.grants.contains(&"mcp".to_string()) && !bare.operator_cred);
    // The SAME caller, elevated in the `mcp` service — the C2 grant.
    let granted = AuthContext { grants: vec!["mcp".to_string()], ..bare.clone() };

    let create = serde_json::json!({
        "id": "probe",
        "serverJson": "{\"command\":\"echo\"}",
        "scope": "project",
    });
    const MCP_CREATE: &str = "ship.ops.v1.McpService/Create";

    // C2 write WITHOUT the grant → the gate denies, in-process, before the
    // handler touches anything.
    let (status, code) = call_op(&app, bare.clone(), MCP_CREATE, create.clone()).await;
    assert_eq!(status, 403, "ungranted C2 must be denied through the in-proc router");
    assert_eq!(code, "permission_denied");

    // The SAME request WITH the `mcp` grant → the gate clears. Whatever the
    // handler then returns, it is NOT a gate denial. The grant is the only
    // thing that changed, so this isolates the gate from the handler.
    let (status, code) = call_op(&app, granted, MCP_CREATE, create).await;
    assert_ne!(status, 403, "granted C2 must clear the gate (got code {code:?})");
    assert_ne!(code, "permission_denied", "granted C2 must clear the gate");

    // Control: a C0 read by the SAME ungranted caller is allowed — proving the
    // denial above is C2-and-grant-specific, not a blanket transport failure.
    let (status, _) = call_op(&app, bare, "ship.ops.v1.McpService/List", serde_json::json!({})).await;
    assert_eq!(status, 200, "C0 reads stay open to any in-scope caller");
}

/// **SECURITY (#124).** `registry.publish` is the C2_ELEVATED op that forwards
/// the operator's stored gateway key to the cloud registry. The audit flagged
/// it as the ONE ungated C2 op — its handler ignored the request context and
/// forwarded unconditionally. This mirrors
/// `in_process_router_enforces_the_gate_for_an_embedded_caller` for

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
    use ship_pipeline_runtime::status_core::{RunStatusReq, update_job_status_core};
    use crate::ports::pipeline::PipelineDispatchAdapter;
    let harness = crate::test_harness::TestDaemon::new().await;
    let body = RunStatusReq {
        status: "gate_passed".to_string(),
        worktree: None,
        pid: None,
        gate_agent: None,
        blocker: None,
        error: None,
    };
    let port = PipelineDispatchAdapter::new(harness.state.clone());
    let v = update_job_status_core(&port, "run-test", body)
        .await
        .expect("gate_passed has no required fields");
    assert_eq!(v["pipeline_run_id"], "run-test");
    assert_eq!(v["status"], "gate_passed");
}
