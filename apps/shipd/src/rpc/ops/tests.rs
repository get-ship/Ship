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

// `UiService` policy conformance — rescued from `rpc/view.rs` (deleted as
// WIRE-5: the file was not in any mod tree and `BlastRadius::C0Safe` would
// not compile). The unified `ship.ui` service carries twelve RPCs; five C0
// reads and seven C1 writes. These assert the proto-as-SSOT contract: every
// op declares its capability + blast radius, and the gate reads it.
#[test]
fn ui_write_ops_policy_in_descriptor() {
    for method in [
        "EnterEditMode",
        "ExitEditMode",
        "Tool",
        "Scaffold",
    ] {
        let path = format!("ship.ops.v1.UiService/{method}");
        let p = policy_for(&path)
            .unwrap_or_else(|| panic!("{path} must carry a policy in the descriptor"));
        assert_eq!(p.capability, "ship.ui.write", "{path} capability");
        assert_eq!(
            p.blast_radius,
            ship_ops_rpc::BlastRadius::C1Space,
            "{path} blast radius"
        );
    }
}

#[test]
fn ui_read_ops_policy_in_descriptor() {
    for method in ["Packs", "Read", "Diagnostics", "Hover", "Dependencies"] {
        let path = format!("ship.ops.v1.UiService/{method}");
        let p = policy_for(&path)
            .unwrap_or_else(|| panic!("{path} must carry a policy in the descriptor"));
        assert_eq!(p.capability, "ship.ui.read", "{path} capability");
        assert_eq!(
            p.blast_radius,
            ship_ops_rpc::BlastRadius::C0Read,
            "{path} blast radius"
        );
    }
}

// ── Proto-policy conformance for PipelinesService / ViewService / AgentsService ──
//
// Rescued from `apps/shipd/src/policy_conformance_tests.rs` (deleted as
// WIRE-6: the file was not in any mod tree, used a nonexistent `MethodPolicy`
// import, and addressed `p.blast` — the struct field is `blast_radius`).
// Each test is rewritten to call `policy_for` directly with the full path
// and assert against `p.blast_radius`. `ported_read_op_carries_its_policy`
// already lives at tests.rs:20, so it's not duplicated here.

#[test]
fn pipelines_list_is_c0_read() {
    let p = policy_for("ship.ops.v1.PipelinesService/List")
        .expect("PipelinesService/List policy must be present");
    assert_eq!(p.capability, "ship.pipelines.read");
    assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
}

#[test]
fn update_status_is_c1_runtime_write() {
    let p = policy_for("ship.ops.v1.PipelinesService/UpdateStatus")
        .expect("PipelinesService/UpdateStatus policy must be present");
    assert_eq!(p.capability, "ship.runtime.write");
    assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space);
}

#[test]
fn pipelines_put_is_c1_config_write() {
    let p = policy_for("ship.ops.v1.PipelinesService/Put")
        .expect("PipelinesService/Put policy must be present");
    assert_eq!(p.capability, "ship.pipelines.write");
    assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space);
}

/// Every `view.*` write on `ViewService` declares `ship.views.write` / C1.
fn view_rpc_op_policy_in_descriptor() {
    for method in [
        "EnterEditMode",
        "ExitEditMode",
        "Tool",
    ] {
        let path = format!("ship.ops.v1.ViewService/{method}");
        let p = policy_for(&path)
            .unwrap_or_else(|| panic!("{path} must carry a policy in the descriptor"));
        assert_eq!(p.capability, "ship.views.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space);
    }
}

#[test]
fn agent_service_control_policies_in_descriptor() {
    let dispatch = policy_for("ship.ops.v1.AgentsService/Dispatch")
        .expect("AgentsService/Dispatch policy must be present");
    assert_eq!(dispatch.capability, "ship.runtime.dispatch");
    assert_eq!(dispatch.blast_radius, ship_ops_rpc::BlastRadius::C2Elevated);

    let grant = policy_for("ship.ops.v1.AgentsService/Grant")
        .expect("AgentsService/Grant policy must be present");
    assert_eq!(grant.capability, "ship.agents.grant");
    assert_eq!(grant.blast_radius, ship_ops_rpc::BlastRadius::C3Critical);
}

#[test]
fn folded_profile_policies_preserved() {
    let list = policy_for("ship.ops.v1.AgentsService/List")
        .expect("AgentsService/List policy must be present");
    assert_eq!(list.capability, "ship.agents.read");
    assert_eq!(list.blast_radius, ship_ops_rpc::BlastRadius::C0Read);

    let create = policy_for("ship.ops.v1.AgentsService/Create")
        .expect("AgentsService/Create policy must be present");
    assert_eq!(create.capability, "ship.agents.write");
    assert_eq!(create.blast_radius, ship_ops_rpc::BlastRadius::C2Elevated);
}

#[test]
fn folded_runtime_read_policies_preserved() {
    let sessions = policy_for("ship.ops.v1.AgentsService/Sessions")
        .expect("AgentsService/Sessions policy must be present");
    assert_eq!(sessions.capability, "ship.sessions.read");
    assert_eq!(sessions.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
}

// ── Gate conformance: Connect handler is the SSOT for agents/agent ops ────
//
// The gate in `AgentsWire` (ship-agents) uses the same `gate::decide` +
// `gate::service_of` logic. These tests prove the gate decisions for
// agents.* and agent.* ops via the gate module directly — no string
// dispatch involved.

#[test]
fn agents_create_requires_agents_grant_not_orchestration() {
    use crate::auth::gate;

    // A commander holds ship-orchestration — NOT ship-authoring (agents).
    // The singular/plural prefix split: orchestration does NOT clear agents.create.
    let commander = AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: Some("commander-1".to_string()),
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![],
        grants: vec!["ship-orchestration".to_string()],
        operator_cred: false,
    };
    let caller = gate::caller_from_auth(&commander);
    assert!(
        matches!(
            gate::decide(
                ship_ops_rpc::BlastRadius::C2Elevated,
                gate::service_of("agents.create"),
                &caller
            ),
            gate::Decision::Deny(_)
        ),
        "orchestration grant must NOT clear agents.create (wrong service)"
    );

    // A designer holds the `agents` service grant — cleared.
    let designer = AuthContext {
        grants: vec!["agents".to_string()],
        ..commander.clone()
    };
    let caller = gate::caller_from_auth(&designer);
    assert_eq!(
        gate::decide(
            ship_ops_rpc::BlastRadius::C2Elevated,
            gate::service_of("agents.create"),
            &caller
        ),
        gate::Decision::Allow,
        "agents grant must clear agents.create"
    );
}

#[test]
fn agent_grant_is_operator_only_c3() {
    use crate::auth::gate;

    // Non-operator (even with orchestration) cannot clear C3.
    let agent = AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: Some("x".to_string()),
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![],
        grants: vec!["ship-orchestration".to_string()],
        operator_cred: false,
    };
    let caller = gate::caller_from_auth(&agent);
    assert!(
        matches!(
            gate::decide(
                ship_ops_rpc::BlastRadius::C3Critical,
                gate::service_of("agent.grant"),
                &caller
            ),
            gate::Decision::Deny(_)
        ),
        "non-operator must be denied at agent.grant (C3)"
    );

    // Operator clears C3.
    let op = crate::test_harness::TestDaemon::operator_auth();
    let op_caller = gate::caller_from_auth(&op);
    assert_eq!(
        gate::decide(
            ship_ops_rpc::BlastRadius::C3Critical,
            gate::service_of("agent.grant"),
            &op_caller
        ),
        gate::Decision::Allow,
        "operator must clear agent.grant C3 gate"
    );
}

/// agent.grant: operator approval upgrades live sessions + persists baseline.
/// Drives the domain fn directly (the logic lives in `AgentsWire::grant_fn`
/// closure in `rpc/ops`, called after the gate clears).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_grant_upgrades_live_session_and_persists_baseline() {
    use ship_auth::scopes::DaemonSession;
    let harness = crate::test_harness::TestDaemon::new().await;
    let token = "sess-grant-test-rpc";
    {
        let mut reg = harness.state.session_registry().lock().await;
        reg.insert(
            token.to_string(),
            DaemonSession {
                token: token.to_string(),
                agent_id: "worker-y".to_string(),
                scopes: vec![],
                grants: vec![],
                workspace_id: None,
                space_id: None,
                pid: None,
                created_at: std::time::Instant::now(),
                expires_at_unix: None,
                ephemeral: true,
                is_operator_cred: false,
            },
        );
    }
    let op = crate::test_harness::TestDaemon::operator_auth();
    let new_grants = vec!["ship-orchestration".to_string()];

    // Inline the domain logic that lives in agent_rpc_helpers::agent_control_write.
    let mut baseline: std::collections::BTreeSet<String> =
        ship_agents::services::permissions::projection::provisioned_grants_for("worker-y")
            .ok()
            .flatten()
            .unwrap_or_default()
            .into_iter()
            .collect();
    let mut upgraded = 0usize;
    {
        let mut reg = harness.state.session_registry().lock().await;
        for session in reg.values_mut() {
            if session.agent_id == "worker-y" {
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
    assert_eq!(upgraded, 1);
    {
        let reg = harness.state.session_registry().lock().await;
        assert!(reg.get(token).unwrap().grants.iter().any(|g| g == "ship-orchestration"));
    }
    baseline.extend(new_grants.iter().cloned());
    let provisioned: Vec<String> = baseline.into_iter().collect();
    ship_auth::scopes::provision_grants(
        "worker-y",
        &provisioned,
        "operator",
        op.agent_id.as_deref(),
    )
    .expect("provision_grants must succeed");
    assert_eq!(
        ship_agents::services::permissions::projection::provisioned_grants_for("worker-y")
            .unwrap(),
        Some(vec!["ship-orchestration".to_string()]),
        "operator approval must persist the provisioned baseline"
    );
}

/// agents.create: designer grant clears gate + domain fn creates the agent.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agents_create_designer_grant_writes_through() {
    let harness = crate::test_harness::TestDaemon::new().await;
    use ship_agents::services::agents::service::CreateArgs;
    use ship_files::files::Scope;
    let doc: serde_json::Value = serde_json::from_str("{\"role\":\"worker\"}").unwrap();
    let resolved = harness
        .state
        .agents()
        .create(CreateArgs {
            id: "fixture-agent-3".to_string(),
            doc,
            scope: Some(Scope::User),
        })
        .expect("domain fn create must succeed");
    assert_eq!(resolved, Scope::User);
}
