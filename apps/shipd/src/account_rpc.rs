//! `AccountService` handler — proto-as-SSOT migration of `account.*` (#89),
//! mounted additively beside the generated ReadOps until the GUI flip (#72).
//!
//! Single C0 read: `account.state` returns opaque JSON because AccountState
//! embeds provider-specific fields that vary by harness.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{AccountService, JsonResponse};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl AccountService for OpsImpl {
    async fn state(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetAccountStateReqView,
    ) -> ServiceResult<JsonResponse> {
        let value =
            crate::services::registry::call("account.state", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("account.state: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    #[allow(refining_impl_trait)]
    async fn state_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetAccountStateReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("account.state", json!({})).await
    }

    async fn logout(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedLogoutReqView,
    ) -> ServiceResult<JsonResponse> {
        self.state()
            .account()
            .logout()
            .await
            .map_err(|e| ConnectError::internal(format!("account.logout: {e}")))?;
        Ok(Response::new(JsonResponse {
            json: json!({ "ok": true }).to_string(),
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn account_op_policy_is_c0_safe() {
        let p = policy_for("ship.ops.v1.AccountService/State")
            .expect("AccountService/State missing from policy map");
        assert_eq!(p.capability, "ship.account.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }
}
