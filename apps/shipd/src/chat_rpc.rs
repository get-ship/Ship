//! `ChatService` handler — proto-as-SSOT migration of `chat.*` (#89), mounted
//! additively beside the generated ReadOps until the GUI flip (#72).
//!
//! Chat reads are C0 and return a typed `ChatStateResponse`; they route
//! through the sync read registry (`registry::call`) with no gate.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::json;

use ship_ops_rpc::{ChatService, ChatStateResponse, JsonResponse};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl ChatService for OpsImpl {
    async fn state(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedChatStateReqView,
    ) -> ServiceResult<ChatStateResponse> {
        let mut args = serde_json::Map::new();
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        let value = crate::services::registry::call(
            "chat.state",
            &serde_json::Value::Object(args),
            self.state(),
        )
        .map_err(|e| ConnectError::internal(format!("chat.state: {e}")))?;
        let msg = serde_json::from_value::<ChatStateResponse>(value)
            .map_err(|e| ConnectError::internal(format!("chat.state decode: {e}")))?;
        Ok(Response::new(msg))
    }

    #[allow(refining_impl_trait)]
    async fn state_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedChatStateReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let mut args = serde_json::Map::new();
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        self.live_read("chat.state", serde_json::Value::Object(args)).await
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn chat_op_policy_is_c0_safe() {
        let p = policy_for("ship.ops.v1.ChatService/State")
            .expect("ChatService/State missing from policy map");
        assert_eq!(p.capability, "ship.chat.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }
}
