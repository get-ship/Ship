//! `AgentsService` handler — proto-as-SSOT migration of `agents.*` (#89).
//! Mounted additively beside the generated ReadOps/CommandOps until the GUI
//! flip (#72).
//!
//! Reads (C0) route through `services::registry::call` and decode onto
//! the typed proto shape or wrap as `JsonResponse`. Writes (C1) route through
//! `services::command::call` (the ONE gate) and decode onto `ScopedMutation`.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use ship_ops_rpc::{
    AgentCatalogsResponse, AgentDetail, AgentDoc, AgentIndexResponse, AgentSettingsResponse,
    AgentTemplatesResponse, AgentWireResponse, AgentsService, AgentsSummariesResponse,
    HarnessesResponse, JsonResponse, PermissionPresetsResponse, ScopedMutation,
};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl AgentsService for OpsImpl {
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
        let args =
            json!({ "id": request.id, "new_id": request.new_id, "scope": request.scope });
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
}

impl OpsImpl {
    /// C0 read → sync registry dispatch, decoded onto the typed proto message.
    fn agents_typed_read<T: DeserializeOwned>(
        &self,
        op: &str,
        args: Value,
    ) -> ServiceResult<T> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        serde_json::from_value::<T>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("{op} decode: {e}")))
    }

    /// C0 read → sync registry dispatch, wrapped as opaque JSON.
    fn agents_read_json(&self, op: &str, args: Value) -> ServiceResult<JsonResponse> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    /// C0 `agents.get` → the opaque-by-design `AgentDoc`. The reader yields
    /// `{ id, scope, doc }` where `doc` is the arbitrary user def object; we
    /// keep it opaque by serializing it into the `doc_json` string field
    /// (same open-payload convention as CollectionItem.data_json). Only the
    /// id + resolved scope are typed.
    fn agents_get_doc(&self, args: Value) -> ServiceResult<AgentDoc> {
        let value = crate::services::registry::call("agents.get", &args, self.state())
            .map_err(|e| ConnectError::internal(format!("agents.get: {e}")))?;
        let id = value.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
        let scope = value.get("scope").and_then(Value::as_str).unwrap_or_default().to_string();
        let doc_json = value.get("doc").map(|d| d.to_string()).unwrap_or_else(|| "null".to_string());
        Ok(Response::new(AgentDoc { id, scope, doc_json, ..Default::default() }))
    }

    /// C1 write → the one gate (`services::command::call`), decoded onto
    /// `ScopedMutation`. Named `agents_gated_mutation` to avoid collision with
    /// the same-shaped helper in `rules_rpc`.
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
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value =
            command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        serde_json::from_value::<ScopedMutation>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn agents_rpc_op_policy_in_descriptor() {
        let r = crate::ops_rpc::policy_for("ship.ops.v1.AgentsService/List")
            .expect("AgentsService/List must carry a policy in the descriptor");
        assert_eq!(r.capability, "ship.agents.read");
        assert_eq!(r.blast_radius, ship_ops_rpc::BlastRadius::C0Read);

        let w = crate::ops_rpc::policy_for("ship.ops.v1.AgentsService/Create")
            .expect("AgentsService/Create must carry a policy in the descriptor");
        assert_eq!(w.capability, "ship.agents.write");
        // Authoring (agent-profile CRUD) is elevated — C2, granted per service.
        assert_eq!(w.blast_radius, ship_ops_rpc::BlastRadius::C2Elevated);
    }
}
