//! `SkillsService` handler — proto-as-SSOT migration of `skills.*` (#89).
//! Mounted additively beside the generated ReadOps/CommandOps until the GUI
//! flip (#72).
//!
//! Reads (C0) call the sync read registry (`registry::call`) and decode the
//! JSON projection onto the proto message. The three opaque reads (summaries /
//! detail / get) wrap the registry JSON in `JsonResponse`; the two typed reads
//! (list / file) decode onto the shared `types.proto` shapes. Writes (C1) route
//! through `services::command::call`, the ONE gate, which enforces the grant.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;
use ship_ops_rpc::{
    JsonResponse, ScopedMutation, SkillFileResponse, SkillsIndexResponse, SkillsService,
};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl SkillsService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSkillsRequestView,
    ) -> ServiceResult<SkillsIndexResponse> {
        let value =
            crate::services::registry::call("skills.list", &serde_json::Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("skills.list: {e}")))?;
        serde_json::from_value::<SkillsIndexResponse>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("skills.list decode: {e}")))
    }

    async fn file(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSkillFileRequestView,
    ) -> ServiceResult<SkillFileResponse> {
        let args = json!({ "id": request.id, "path": request.path });
        let value =
            crate::services::registry::call("skills.file", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("skills.file: {e}")))?;
        serde_json::from_value::<SkillFileResponse>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("skills.file decode: {e}")))
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSkillSummariesRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "q": request.q, "kind": request.kind, "tag": request.tag });
        let value =
            crate::services::registry::call("skills.summaries", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("skills.summaries: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn detail(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSkillDetailRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id });
        let value =
            crate::services::registry::call("skills.detail", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("skills.detail: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSkillRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id, "scope": request.scope });
        let value =
            crate::services::registry::call("skills.get", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("skills.get: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({
            "id": request.id, "scope": request.scope,
            "frontmatter_json": request.frontmatter_json, "body": request.body,
        });
        self.gated_skill_mutation(&ctx, "skills.create", args).await
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "scope": request.scope });
        self.gated_skill_mutation(&ctx, "skills.delete", args).await
    }

    async fn rename(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedRenameSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "new_id": request.new_id, "scope": request.scope });
        self.gated_skill_mutation(&ctx, "skills.rename", args).await
    }

    async fn duplicate(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDuplicateSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "new_id": request.new_id, "scope": request.scope });
        self.gated_skill_mutation(&ctx, "skills.duplicate", args).await
    }

    async fn write_frontmatter(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedWriteSkillFrontmatterRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({
            "id": request.id, "frontmatter_json": request.frontmatter_json, "scope": request.scope,
        });
        self.gated_skill_mutation(&ctx, "skills.write_frontmatter", args).await
    }

    async fn write_body(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedWriteSkillBodyRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "body": request.body, "scope": request.scope });
        self.gated_skill_mutation(&ctx, "skills.write_body", args).await
    }

    async fn move_scope(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedMoveSkillScopeRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "from": request.from, "to": request.to });
        self.gated_skill_mutation(&ctx, "skills.move_scope", args).await
    }

    async fn write_file(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedWriteSkillFileRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({
            "id": request.id, "path": request.path,
            "content": request.content, "scope": request.scope,
        });
        self.gated_skill_mutation(&ctx, "skills.write_file", args).await
    }

    async fn delete_file(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteSkillFileRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "path": request.path, "scope": request.scope });
        self.gated_skill_mutation(&ctx, "skills.delete_file", args).await
    }

    async fn set_var(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSetSkillVarRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id, "key": request.key, "value_json": request.value_json });
        self.gated_skill_json(&ctx, "skills.set_var", args).await
    }

    async fn delete_var(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteSkillVarRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id, "key": request.key });
        self.gated_skill_json(&ctx, "skills.delete_var", args).await
    }
}

impl OpsImpl {
    /// Route a `skills.*` write through the one gate and decode onto
    /// `ScopedMutation`. The gate reads blast/capability from the descriptor.
    async fn gated_skill_mutation(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: serde_json::Value,
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

    /// Route a `skills.*` write through the one gate and wrap the result in
    /// `JsonResponse` (for set_var / delete_var whose returns are opaque).
    async fn gated_skill_json(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: serde_json::Value,
    ) -> ServiceResult<JsonResponse> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value =
            command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

#[cfg(test)]
mod tests {
    /// Read ops declare ship.skills.read/C0 and write ops declare
    /// ship.skills.write/C1 in the compiled descriptor — the field-level policy
    /// option that the gate reads at runtime. Matching the existing skills ops
    /// from the generated ReadOps/CommandOps is the proto-as-SSOT contract.
    #[test]
    fn skills_rpc_op_policy_in_descriptor() {
        let r = crate::ops_rpc::policy_for("ship.ops.v1.SkillsService/List")
            .expect("SkillsService/List must carry a policy in the descriptor");
        assert_eq!(r.capability, "ship.skills.read");
        assert_eq!(r.blast_radius, ship_ops_rpc::BlastRadius::C0Read);

        let w = crate::ops_rpc::policy_for("ship.ops.v1.SkillsService/Create")
            .expect("SkillsService/Create must carry a policy in the descriptor");
        assert_eq!(w.capability, "ship.skills.write");
        // Authoring (skill CRUD) is elevated — C2, granted per service.
        assert_eq!(w.blast_radius, ship_ops_rpc::BlastRadius::C2Elevated);
    }
}
