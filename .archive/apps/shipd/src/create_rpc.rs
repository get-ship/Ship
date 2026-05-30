//! `CreateService` handler — proto-as-SSOT migration of `create.surface` (#89).
//! Mounted additively beside the generated CommandOpsService until the GUI flip
//! (#72).
//!
//! The single op scaffolds a new surface (view | skill). It routes through
//! `services::command::call`, the ONE gate (enforcing `ship.surfaces.write`),
//! which dispatches to `services::command::surfaces`. The return is opaque, so
//! it wraps the command JSON in `JsonResponse`, mirroring the generated path.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;
use ship_ops_rpc::{CreateService, JsonResponse};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl CreateService for OpsImpl {
    async fn surface(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateSurfaceReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "surface": request.surface, "name": request.name,
            "shape": request.shape, "purpose": request.purpose,
        });
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value = command::call("create.surface", &args, &cmd_ctx)
            .await
            .map_err(command_error_to_connect)?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

#[cfg(test)]
mod tests {
    /// `create.surface` declares `ship.surfaces.write` / C1 in the compiled
    /// descriptor — the policy the gate reads at runtime.
    #[test]
    fn create_rpc_op_policy_in_descriptor() {
        let p = crate::ops_rpc::policy_for("ship.ops.v1.CreateService/Surface")
            .expect("CreateService/Surface must carry a policy in the descriptor");
        assert_eq!(p.capability, "ship.surfaces.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Workspace);
    }
}
