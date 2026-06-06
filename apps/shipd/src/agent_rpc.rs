//! `AgentsService` handler — the ONE canonical agent RPC surface (#89,
//! agents-arc E4/E6). Collapsed from the historic split (profile CRUD + view
//! reads, orchestration, SessionsService runtime-read) into this single
//! `impl AgentsService for OpsImpl`. Mounted additively beside the generated
//! ReadOps/CommandOps until the GUI flip (#72). Plural by convention (E6 reverted
//! E4's outlier singular rename); chat was gutted in E6.
//!
//! Three planes meet here, each calling domain crates directly — no
//! string-registry or string-command dispatch: profile reads (C0) + runtime-scoped
//! sessions reads (C0) → domain fns in ship-platform; profile writes (C2) +
//! orchestration control (C2/C3) → `services::command::call` (the ONE gate).
//!
//! The impl-helper methods live in `agent_rpc_helpers` to keep this file within
//! the cap — not a parallel handler.

use connectrpc::{RequestContext, ServiceResult, ServiceStream};
use serde_json::json;

use ship_ops_rpc::{
    AgentCatalogsResponse, AgentDetail, AgentDoc, AgentIndexResponse, AgentSettingsResponse,
    AgentTemplatesResponse, AgentWireResponse, AgentsService, AgentsSummariesResponse,
    DispatchResult, GrantResult, HarnessesResponse, JsonResponse,
    PermissionPresetsResponse, ScopedMutation, SessionsListResponse, SteerResult, StopResult,
    WatchEvent,
};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl AgentsService for OpsImpl {
    // Profile reads (C0) — calling domain crates directly.
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsRequestView,
    ) -> ServiceResult<AgentIndexResponse> {
        self.agents_list()
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsSummariesRequestView,
    ) -> ServiceResult<AgentsSummariesResponse> {
        self.agents_summaries()
    }

    async fn detail(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentDetailRequestView,
    ) -> ServiceResult<AgentDetail> {
        self.agents_detail(&request.id)
    }

    async fn catalogs(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsCatalogsRequestView,
    ) -> ServiceResult<AgentCatalogsResponse> {
        self.agents_catalogs()
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentRequestView,
    ) -> ServiceResult<AgentDoc> {
        let args = json!({ "id": request.id, "scope": request.scope });
        self.agents_get_doc(args)
    }

    async fn wire(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentWireRequestView,
    ) -> ServiceResult<AgentWireResponse> {
        self.agents_wire(&request.id)
    }

    async fn settings(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentSettingsRequestView,
    ) -> ServiceResult<AgentSettingsResponse> {
        self.agents_settings(&request.id)
    }

    // The ONE reactive surface (agents-arc E5) — replaces the six deleted *Live
    // twins. Decodes the scope oneof, then delegates to the lean stream helper
    // (key + prefilter + snapshot live in `agent_rpc_helpers`).
    #[allow(refining_impl_trait)]
    async fn watch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedAgentWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        use ship_ops_rpc::__buffa::oneof::agent_watch_req::Scope;
        let scope = match request.to_owned_message().scope {
            Some(Scope::All(_)) => None,
            Some(Scope::AgentId(id)) => Some(id),
            None => {
                return Err(connectrpc::ConnectError::invalid_argument(
                    "watch scope is required (all | agent_id)",
                ));
            }
        };
        self.agent_watch_stream(&ctx, scope)
    }

    async fn templates(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentTemplatesRequestView,
    ) -> ServiceResult<AgentTemplatesResponse> {
        self.agents_templates()
    }

    async fn permission_presets(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentPermissionPresetsRequestView,
    ) -> ServiceResult<PermissionPresetsResponse> {
        self.agents_permission_presets()
    }

    async fn harnesses(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentHarnessesRequestView,
    ) -> ServiceResult<HarnessesResponse> {
        self.agents_harnesses()
    }

    async fn create_ctx(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetAgentCreateCtxRequestView,
    ) -> ServiceResult<JsonResponse> {
        // NOTE (E5): CreateCtxLive is DELETED with NO Watch replacement — it
        // watched templates/permission-presets/harnesses (built-in + static),
        // which never change at runtime, so the static `CreateCtx` read suffices
        // for the greenfield create UI. Intentional, not an oversight.
        self.agents_create_ctx()
    }

    // Runtime-scoped reads (C0) — folded from SessionsService.
    async fn sessions(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSessionsReqView,
    ) -> ServiceResult<SessionsListResponse> {
        self.agents_sessions()
    }

    #[allow(refining_impl_trait)]
    async fn sessions_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSessionsReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.agents_sessions_live().await
    }

    // Profile writes (C2) — gated through services::command::call.
    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "doc_json": request.doc_json, "scope": request.scope });
        self.agents_gated_mutation(&ctx, "agents.create", args).await
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "doc_json": request.doc_json, "scope": request.scope });
        self.agents_gated_mutation(&ctx, "agents.put", args).await
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args =
            json!({ "id": request.id, "patch_json": request.patch_json, "scope": request.scope });
        self.agents_gated_mutation(&ctx, "agents.patch", args).await
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "scope": request.scope });
        self.agents_gated_mutation(&ctx, "agents.delete", args).await
    }

    async fn rename(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedRenameAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "new_id": request.new_id, "scope": request.scope });
        self.agents_gated_mutation(&ctx, "agents.rename", args).await
    }

    async fn duplicate(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDuplicateAgentRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({
            "id": request.id, "new_id": request.new_id,
            "scope": request.scope, "to": request.to,
        });
        self.agents_gated_mutation(&ctx, "agents.duplicate", args).await
    }

    async fn move_scope(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedMoveAgentScopeRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "from": request.from, "to": request.to });
        self.agents_gated_mutation(&ctx, "agents.move_scope", args).await
    }

    // Orchestration control (C2/C3) — typed results.
    async fn dispatch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDispatchAgentRequestView,
    ) -> ServiceResult<DispatchResult> {
        let mut args = serde_json::Map::new();
        args.insert("agent".to_string(), json!(request.agent));
        args.insert("spec".to_string(), json!(request.spec));
        args.insert("prompt".to_string(), json!(request.prompt));
        if let Some(b) = request.resume { args.insert("resume".to_string(), json!(b)); }
        args.insert("harness".to_string(), json!(request.harness));
        args.insert("model".to_string(), json!(request.model));
        if let Some(b) = request.open_tab { args.insert("open_tab".to_string(), json!(b)); }
        args.insert("invocation".to_string(), json!(request.invocation));
        if let Some(b) = request.keepalive { args.insert("keepalive".to_string(), json!(b)); }
        args.insert("workspace".to_string(), json!(request.workspace));
        args.insert("slug".to_string(), json!(request.slug));
        args.insert("branch".to_string(), json!(request.branch));
        args.insert("source_spec_path".to_string(), json!(request.source_spec_path));
        args.insert(
            "depends_on".to_string(),
            json!(request.depends_on.iter().collect::<Vec<_>>()),
        );
        let mut vars = serde_json::Map::new();
        for (k, v) in request.vars.iter() {
            vars.insert(k.to_string(), json!(v));
        }
        args.insert("vars".to_string(), serde_json::Value::Object(vars));
        self.agent_gated_typed::<DispatchResult>(&ctx, "agent.dispatch", serde_json::Value::Object(args)).await
    }

    async fn stop(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedStopAgentRequestView,
    ) -> ServiceResult<StopResult> {
        let args = json!({ "slug": request.slug });
        self.agent_gated_typed::<StopResult>(&ctx, "agent.stop", args).await
    }

    async fn steer(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSteerAgentRequestView,
    ) -> ServiceResult<SteerResult> {
        let args = json!({ "slug": request.slug, "message": request.message });
        self.agent_gated_typed::<SteerResult>(&ctx, "agent.steer", args).await
    }

    async fn grant(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGrantAgentRequestView,
    ) -> ServiceResult<GrantResult> {
        let grants: Vec<_> = request.grants.iter().collect();
        let args = json!({ "agent": request.agent, "grants": grants });
        self.agent_gated_typed::<GrantResult>(&ctx, "agent.grant", args).await
    }
}

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
    // The inline gate in `agents_gate` (agent_rpc_helpers.rs) uses the same
    // `gate::decide` + `gate::service_of` logic as `command::call`. These tests
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
    /// `agent_rpc_helpers::agent_control_write`, called after the gate clears).
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
