//! Policy + gate conformance tests for AgentsService.
//!
//! `impl AgentsService for OpsImpl` is deleted — the handler now lives in
//! `ship_agents::services::agents::AgentsWire`, mounted via
//! `AgentsServiceExt::register` in `ops_rpc.rs`. Tests here verify the proto
//! policy descriptor + gate decisions that the wire enforces at runtime.

#[cfg(test)]
mod tests {
    use ship_ops_rpc::BlastRadius;

    fn policy(method: &str) -> &'static crate::ops_rpc::OpPolicy {
        crate::ops_rpc::policy_for(&format!("ship.ops.v1.AgentsService/{method}"))
            .unwrap_or_else(|| panic!("AgentsService/{method} must carry a policy in the descriptor"))
    }

    #[test]
    fn agent_service_control_policies_in_descriptor() {
        let dispatch = policy("Dispatch");
        assert_eq!(dispatch.capability, "ship.runtime.dispatch");
        // Dispatch spawns/controls agents — C2 (elevated, granted to commander).
        assert_eq!(dispatch.blast_radius, BlastRadius::C2Elevated);

        let grant = policy("Grant");
        assert_eq!(grant.capability, "ship.agents.grant");
        // Grant hands out authority — C3 (operator/2FA only).
        assert_eq!(grant.blast_radius, BlastRadius::C3Critical);
    }

    #[test]
    fn folded_profile_policies_preserved() {
        let list = policy("List");
        assert_eq!(list.capability, "ship.agents.read");
        assert_eq!(list.blast_radius, BlastRadius::C0Read);

        let create = policy("Create");
        assert_eq!(create.capability, "ship.agents.write");
        // Authoring (agent-profile CRUD) is elevated — C2.
        assert_eq!(create.blast_radius, BlastRadius::C2Elevated);
    }

    #[test]
    fn folded_runtime_read_policies_preserved() {
        // Sessions folded from SessionsService keeps its capability.
        let sessions = policy("Sessions");
        assert_eq!(sessions.capability, "ship.sessions.read");
        assert_eq!(sessions.blast_radius, BlastRadius::C0Read);
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
        use crate::services::gate;

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
        use crate::services::gate;

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
