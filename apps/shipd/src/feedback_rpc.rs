//! `FeedbackService` handler — proto-as-SSOT migration of `feedback.*` (#89),
//! mounted additively beside the generated CommandOpsService until the GUI
//! flip (#72).
//!
//! Write-only service. The single write (C1) routes through
//! `services::command::call`, the ONE gate.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;

use ship_ops_rpc::{FeedbackService, JsonResponse};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl FeedbackService for OpsImpl {
    async fn post(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPostFeedbackRequestView,
    ) -> ServiceResult<JsonResponse> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let args = json!({ "event_json": request.event_json });
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value = command::call("feedback.post", &args, &cmd_ctx)
            .await
            .map_err(command_error_to_connect)?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn feedback_op_policy_write_is_c1_workspace() {
        let p = policy_for("ship.ops.v1.FeedbackService/Post")
            .expect("FeedbackService/Post must carry a policy in the descriptor");
        assert_eq!(p.capability, "ship.feedback.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Workspace);
    }
}
