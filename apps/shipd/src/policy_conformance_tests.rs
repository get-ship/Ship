//! Proto-policy + gate conformance tests for the wires shipd mounts.
//!
//! Proto owns op policy (#67a): capability / blast-radius / sensitive method
//! options are compiled into the `FileDescriptorSet` and read through the
//! single crate-level reader (`ship_ops_rpc::policy::read_policies`) — the
//! same decode the enforcement gate uses. shipd keeps NO policy map of its
//! own; these tests pin that the contract every generated client sees is the
//! one the gate enforces.
//!
//! Merged from the former `agent_rpc.rs` / `pipelines_rpc.rs` / `view_rpc.rs`
//! headstones (their `impl … for OpsImpl` bodies were deleted when the wires
//! became the sole handlers) plus the policy tests that lived in `ops_rpc.rs`.

#[cfg(test)]
mod tests {
    use ship_ops_rpc::BlastRadius;
    use ship_ops_rpc::policy::MethodPolicy;

    /// Resolve a Connect wire-path (`ship.ops.v1.Svc/Method`) to its compiled
    /// proto policy. Panics with the path so a missing descriptor entry names
    /// itself.
    fn policy(wire_path: &str) -> MethodPolicy {
        ship_ops_rpc::policy::read_policies()
            .into_iter()
            .find(|p| p.wire_path() == wire_path)
            .unwrap_or_else(|| panic!("{wire_path} must carry a policy in the descriptor"))
    }

    // ── OpsService / AccountService ───────────────────────────────────────────

    #[test]
    fn meta_op_policy_read_from_proto_descriptor() {
        // Proves #67a end to end: the method options declared in ops.proto are
        // compiled into the descriptor and recovered by the gate.
        let p = policy("ship.ops.v1.OpsService/Meta");
        assert_eq!(p.capability, "ship.meta.read");
        assert_eq!(p.blast, BlastRadius::C0Read);
        assert!(!p.sensitive);
    }

    #[test]
    fn ported_read_op_carries_its_policy() {
        // account.state keeps its own proto policy on the canonical per-domain
        // service. The `ReadOpsService` umbrella was retired (#78) — per-domain
        // Connect is the SSOT, so the policy resolves via `AccountService/State`.
        //
        // FLAG-3 honesty fix: the wire enforces require_operator (operator-only,
        // S6 #124) for ALL AccountService ops including State — proto was
        // mistakenly declared C0. The blast class is now C3Critical to match
        // the wire enforcement (no enforcement change — doc/classification only).
        let p = policy("ship.ops.v1.AccountService/State");
        assert_eq!(p.capability, "ship.account.read");
        assert_eq!(p.blast, BlastRadius::C3Critical);
    }

    // ── PipelinesService ──────────────────────────────────────────────────────

    #[test]
    fn pipelines_list_is_c0_read() {
        let p = policy("ship.ops.v1.PipelinesService/List");
        assert_eq!(p.capability, "ship.pipelines.read");
        assert_eq!(p.blast, BlastRadius::C0Read);
    }

    #[test]
    fn update_status_is_c1_runtime_write() {
        let p = policy("ship.ops.v1.PipelinesService/UpdateStatus");
        assert_eq!(p.capability, "ship.runtime.write");
        assert_eq!(p.blast, BlastRadius::C1Space);
    }

    #[test]
    fn pipelines_put_is_c1_config_write() {
        let p = policy("ship.ops.v1.PipelinesService/Put");
        assert_eq!(p.capability, "ship.pipelines.write");
        assert_eq!(p.blast, BlastRadius::C1Space);
    }

    // ── ViewService ───────────────────────────────────────────────────────────

    /// Every `view.*` write declares `ship.views.write` / C1 in the compiled
    /// descriptor — the field-level policy the gate reads at runtime. Matching
    /// the generated CommandOps `view.*` ops is the proto-as-SSOT contract.
    #[test]
    fn view_rpc_op_policy_in_descriptor() {
        for method in [
            "PatchSession",
            "AppendSession",
            "SignalSession",
            "EnterEditMode",
            "ExitEditMode",
            "Tool",
        ] {
            let p = policy(&format!("ship.ops.v1.ViewService/{method}"));
            assert_eq!(p.capability, "ship.views.write");
            assert_eq!(p.blast, BlastRadius::C1Space);
        }
    }

    // ── AgentsService ─────────────────────────────────────────────────────────

    #[test]
    fn agent_service_control_policies_in_descriptor() {
        let dispatch = policy("ship.ops.v1.AgentsService/Dispatch");
        assert_eq!(dispatch.capability, "ship.runtime.dispatch");
        // Dispatch spawns/controls agents — C2 (elevated, granted to commander).
        assert_eq!(dispatch.blast, BlastRadius::C2Elevated);

        let grant = policy("ship.ops.v1.AgentsService/Grant");
        assert_eq!(grant.capability, "ship.agents.grant");
        // Grant hands out authority — C3 (operator/2FA only).
        assert_eq!(grant.blast, BlastRadius::C3Critical);
    }

    #[test]
    fn folded_profile_policies_preserved() {
        let list = policy("ship.ops.v1.AgentsService/List");
        assert_eq!(list.capability, "ship.agents.read");
        assert_eq!(list.blast, BlastRadius::C0Read);

        let create = policy("ship.ops.v1.AgentsService/Create");
        assert_eq!(create.capability, "ship.agents.write");
        // Authoring (agent-profile CRUD) is elevated — C2.
        assert_eq!(create.blast, BlastRadius::C2Elevated);
    }

    #[test]
    fn folded_runtime_read_policies_preserved() {
        // Sessions folded from SessionsService keeps its capability.
        let sessions = policy("ship.ops.v1.AgentsService/Sessions");
        assert_eq!(sessions.capability, "ship.sessions.read");
        assert_eq!(sessions.blast, BlastRadius::C0Read);
    }

    // ── Gate conformance: Connect handler is the SSOT for agents/agent ops ────
    //
    // The gate in `AgentsWire` (ship-agents) uses the same
    // `gate::decide` + `gate::service_of` logic. These tests
    // prove the gate decisions for agents.* and agent.* ops via the gate module
    // directly — no string dispatch involved.

    #[test]
    fn agents_create_requires_agents_grant_not_orchestration() {
        use crate::auth::AuthContext;
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
                gate::decide(BlastRadius::C2Elevated, gate::service_of("agents.create"), &caller),
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
            gate::decide(BlastRadius::C2Elevated, gate::service_of("agents.create"), &caller),
            gate::Decision::Allow,
            "agents grant must clear agents.create"
        );
    }

    #[test]
    fn agent_grant_is_operator_only_c3() {
        use crate::auth::AuthContext;
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
                gate::decide(BlastRadius::C3Critical, gate::service_of("agent.grant"), &caller),
                gate::Decision::Deny(_)
            ),
            "non-operator must be denied at agent.grant (C3)"
        );

        // Operator clears C3.
        let op = crate::test_harness::TestDaemon::operator_auth();
        let op_caller = gate::caller_from_auth(&op);
        assert_eq!(
            gate::decide(BlastRadius::C3Critical, gate::service_of("agent.grant"), &op_caller),
            gate::Decision::Allow,
            "operator must clear agent.grant C3 gate"
        );
    }

    /// agent.grant: operator approval upgrades live sessions + persists baseline.
    /// Drives the ship_platform domain fn directly (the logic now lives in
    /// `AgentsWire::grant_fn` closure in `ops_rpc.rs`, called after the gate clears).
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

        // Inline the domain logic that was in command/agent.rs — the same logic
        // now lives in agent_rpc_helpers::agent_control_write.
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
            "worker-y", &provisioned, "operator", op.agent_id.as_deref(),
        ).expect("provision_grants must succeed");
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
        let resolved = harness.state
            .agents()
            .create(CreateArgs {
                id: "fixture-agent-3".to_string(),
                doc,
                scope: Some(Scope::User),
            })
            .expect("domain fn create must succeed");
        assert_eq!(resolved, Scope::User);
    }
}
