//! `AgentService` handler — the ONE canonical agent RPC surface (#89,
//! agents-arc E4). Collapsed from the historic split (AgentsService profile
//! CRUD + view reads, AgentService orchestration, SessionsService runtime-read,
//! ChatService chat-read) into this single `impl AgentService for OpsImpl`.
//! Mounted additively beside the generated ReadOps/CommandOps until the GUI
//! flip (#72).
//!
//! Three planes meet here, each routed to its existing chokepoint — this handler
//! only routes, never reimplements gating: profile reads (C0) + runtime-scoped
//! sessions/chat reads (C0) → `services::registry::call`; profile writes (C2) +
//! orchestration control (C2/C3) → `services::command::call` (the ONE gate).
//!
//! The impl-helper methods (`agents_typed_read`, `agent_gated_typed`, …) live in
//! `agent_rpc_helpers` to keep this file within the cap — not a parallel handler.

use connectrpc::{RequestContext, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{
    AgentCatalogsResponse, AgentDetail, AgentDoc, AgentIndexResponse, AgentSettingsResponse,
    AgentTemplatesResponse, AgentWireResponse, AgentService, AgentsSummariesResponse,
    ChatStateResponse, DispatchResult, GrantResult, HarnessesResponse, JsonResponse,
    PermissionPresetsResponse, ScopedMutation, SessionsListResponse, SteerResult, StopResult,
};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl AgentService for OpsImpl {
    // Profile reads (C0) — folded from AgentsService.
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsRequestView,
    ) -> ServiceResult<AgentIndexResponse> {
        self.agents_typed_read::<AgentIndexResponse>("agents.list", Value::Null)
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsSummariesRequestView,
    ) -> ServiceResult<AgentsSummariesResponse> {
        self.agents_typed_read::<AgentsSummariesResponse>("agents.summaries", Value::Null)
    }

    #[allow(refining_impl_trait)]
    async fn summaries_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsSummariesRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("agents.summaries", json!({})).await
    }

    async fn detail(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentDetailRequestView,
    ) -> ServiceResult<AgentDetail> {
        let args = json!({ "id": request.id });
        self.agents_typed_read::<AgentDetail>("agents.detail", args)
    }

    async fn catalogs(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsCatalogsRequestView,
    ) -> ServiceResult<AgentCatalogsResponse> {
        self.agents_typed_read::<AgentCatalogsResponse>("agents.catalogs", Value::Null)
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
        let args = json!({ "id": request.id });
        self.agents_typed_read::<AgentWireResponse>("agents.wire", args)
    }

    async fn settings(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentSettingsRequestView,
    ) -> ServiceResult<AgentSettingsResponse> {
        let args = json!({ "id": request.id });
        self.agents_typed_read::<AgentSettingsResponse>("agents.settings", args)
    }

    #[allow(refining_impl_trait)]
    async fn catalogs_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentsCatalogsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("agents.catalogs", json!({})).await
    }

    #[allow(refining_impl_trait)]
    async fn detail_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentDetailRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("agents.detail", json!({ "id": request.id })).await
    }

    #[allow(refining_impl_trait)]
    async fn wire_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentWireRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("agents.wire", json!({ "id": request.id })).await
    }

    #[allow(refining_impl_trait)]
    async fn settings_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetAgentSettingsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("agents.settings", json!({ "id": request.id })).await
    }

    async fn templates(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentTemplatesRequestView,
    ) -> ServiceResult<AgentTemplatesResponse> {
        self.agents_typed_read::<AgentTemplatesResponse>("agents.templates", Value::Null)
    }

    async fn permission_presets(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentPermissionPresetsRequestView,
    ) -> ServiceResult<PermissionPresetsResponse> {
        self.agents_typed_read::<PermissionPresetsResponse>("agents.permission_presets", Value::Null)
    }

    async fn harnesses(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListAgentHarnessesRequestView,
    ) -> ServiceResult<HarnessesResponse> {
        self.agents_typed_read::<HarnessesResponse>("agents.harnesses", Value::Null)
    }

    async fn create_ctx(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetAgentCreateCtxRequestView,
    ) -> ServiceResult<JsonResponse> {
        self.agents_read_json("agents.create_ctx", Value::Null)
    }

    #[allow(refining_impl_trait)]
    async fn create_ctx_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetAgentCreateCtxRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("agents.create_ctx", json!({})).await
    }

    // Runtime-scoped reads (C0) — folded from SessionsService + ChatService.
    async fn sessions(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSessionsReqView,
    ) -> ServiceResult<SessionsListResponse> {
        self.agents_typed_read::<SessionsListResponse>("sessions.list", Value::Null)
    }

    #[allow(refining_impl_trait)]
    async fn sessions_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSessionsReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("sessions.list", json!({})).await
    }

    async fn chat(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedChatStateReqView,
    ) -> ServiceResult<ChatStateResponse> {
        self.agents_typed_read::<ChatStateResponse>("chat.state", chat_args(request.workspace_id))
    }

    #[allow(refining_impl_trait)]
    async fn chat_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedChatStateReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("chat.state", chat_args(request.workspace_id)).await
    }

    // Profile writes (C2) — folded from AgentsService.
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
        args.insert("vars".to_string(), Value::Object(vars));
        self.agent_gated_typed::<DispatchResult>(&ctx, "agent.dispatch", Value::Object(args)).await
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

/// Build the `chat.state` args object, omitting `workspace_id` when empty (the
/// registry op treats a missing key as "all workspaces").
fn chat_args(workspace_id: &str) -> Value {
    let mut args = serde_json::Map::new();
    if !workspace_id.is_empty() {
        args.insert("workspace_id".to_string(), json!(workspace_id));
    }
    Value::Object(args)
}

#[cfg(test)]
mod tests {
    use ship_ops_rpc::BlastRadius;

    fn policy(method: &str) -> &'static crate::ops_rpc::OpPolicy {
        crate::ops_rpc::policy_for(&format!("ship.ops.v1.AgentService/{method}"))
            .unwrap_or_else(|| panic!("AgentService/{method} must carry a policy in the descriptor"))
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
        // Sessions + Chat folded from their own services keep their capabilities.
        let sessions = policy("Sessions");
        assert_eq!(sessions.capability, "ship.sessions.read");
        assert_eq!(sessions.blast_radius, BlastRadius::C0Read);

        let chat = policy("Chat");
        assert_eq!(chat.capability, "ship.chat.read");
        assert_eq!(chat.blast_radius, BlastRadius::C0Read);
    }
}
