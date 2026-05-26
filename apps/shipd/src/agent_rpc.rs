//! `AgentService` handler — proto-as-SSOT migration of `agent.*` (#89) —
//! the orchestration surface (dispatch / stop / steer / grant). Mounted
//! additively beside the generated CommandOps until the GUI flip (#72).
//!
//! All four ops are C1/C2 writes routed through `services::command::call`
//! (the ONE gate). Capabilities and blast radii mirror the generated
//! CommandOpsService exactly: dispatch → ship.runtime.dispatch/C1,
//! stop/steer → ship.agents.write/C1, grant → ship.agents.grant/C2_ELEVATED.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::{Value, json};

use ship_ops_rpc::{AgentService, JsonResponse};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl AgentService for OpsImpl {
    async fn dispatch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDispatchAgentRequestView,
    ) -> ServiceResult<JsonResponse> {
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
        self.agent_gated_json(&ctx, "agent.dispatch", Value::Object(args)).await
    }

    async fn stop(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedStopAgentRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "slug": request.slug });
        self.agent_gated_json(&ctx, "agent.stop", args).await
    }

    async fn steer(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSteerAgentRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "slug": request.slug, "message": request.message });
        self.agent_gated_json(&ctx, "agent.steer", args).await
    }

    async fn grant(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGrantAgentRequestView,
    ) -> ServiceResult<JsonResponse> {
        let grants: Vec<_> = request.grants.iter().collect();
        let args = json!({ "agent": request.agent, "grants": grants });
        self.agent_gated_json(&ctx, "agent.grant", args).await
    }
}

impl OpsImpl {
    /// C1/C2 write → the one gate (`services::command::call`), result wrapped
    /// as opaque `JsonResponse`. Named `agent_gated_json` to be unique across
    /// modules (orchestration surface, not profile CRUD).
    pub(crate) async fn agent_gated_json(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: Value,
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
    #[test]
    fn agent_rpc_op_policy_in_descriptor() {
        let dispatch = crate::ops_rpc::policy_for("ship.ops.v1.AgentService/Dispatch")
            .expect("AgentService/Dispatch must carry a policy in the descriptor");
        assert_eq!(dispatch.capability, "ship.runtime.dispatch");
        // Dispatch spawns/controls agents — C2 (elevated, granted to commander).
        assert_eq!(dispatch.blast_radius, ship_ops_rpc::BlastRadius::C2Elevated);

        let grant = crate::ops_rpc::policy_for("ship.ops.v1.AgentService/Grant")
            .expect("AgentService/Grant must carry a policy in the descriptor");
        assert_eq!(grant.capability, "ship.agents.grant");
        // Grant hands out authority — C3 (operator/2FA only).
        assert_eq!(grant.blast_radius, ship_ops_rpc::BlastRadius::C3Critical);
    }
}
