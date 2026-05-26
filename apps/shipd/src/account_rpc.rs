//! `AccountService` handler — proto-as-SSOT migration of `account.*` (#89, #141).
//!
//! All account ops require **user presence** (CallerKind::Cli or browser
//! cookie) — MCP session tokens are categorically denied (SECURITY.md
//! "CallerKind gate on daemon account routes"). The REST routes enforced this
//! via the per-route `@auth cli` middleware; the Connect `ops_rpc` router does
//! NOT run that middleware, so every impl here calls `require_user_presence`.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{AccountService, JsonResponse};

use crate::auth::AuthContext;
use crate::ops_rpc::OpsImpl;

/// Reject any caller that isn't user-present (CLI daemon token or browser
/// cookie — both map to `CallerKind::Cli`). Mirrors `enforce_required_auth`
/// `RequiredAuth::Cli` for the Connect surface.
fn require_user_presence(ctx: &RequestContext) -> Result<(), ConnectError> {
    let auth = ctx
        .extensions
        .get::<AuthContext>()
        .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
    if matches!(auth.caller_kind, ship_substrate::events::CallerKind::Cli) {
        Ok(())
    } else {
        Err(ConnectError::permission_denied(
            "requires user-presence (CallerKind::Cli)",
        ))
    }
}

#[allow(refining_impl_trait)]
impl AccountService for OpsImpl {
    async fn state(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetAccountStateReqView,
    ) -> ServiceResult<JsonResponse> {
        require_user_presence(&ctx)?;
        let value =
            crate::services::registry::call("account.state", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("account.state: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    #[allow(refining_impl_trait)]
    async fn state_live(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetAccountStateReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        require_user_presence(&ctx)?;
        self.live_read("account.state", json!({})).await
    }

    async fn logout(
        &self,
        ctx: RequestContext,
        _request: ship_ops_rpc::OwnedLogoutReqView,
    ) -> ServiceResult<JsonResponse> {
        require_user_presence(&ctx)?;
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

    async fn login(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedLoginReqView,
    ) -> ServiceResult<JsonResponse> {
        require_user_presence(&ctx)?;
        let redirect_uri = request.redirect_uri;
        if !redirect_uri.starts_with("http://127.0.0.1:")
            && !redirect_uri.starts_with("http://localhost:")
        {
            return Err(ConnectError::invalid_argument(
                "redirect_uri must be a localhost http URL",
            ));
        }
        let resp = self
            .state()
            .account()
            .start_login(redirect_uri)
            .await
            .map_err(|e| ConnectError::internal(format!("account.login: {e}")))?;
        let value = json!({ "state_id": resp.state_id, "browser_url": resp.browser_url });
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn login_complete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedLoginCompleteReqView,
    ) -> ServiceResult<JsonResponse> {
        require_user_presence(&ctx)?;
        let me = self
            .state()
            .account()
            .complete_login(request.state_id, request.code)
            .await
            .map_err(|e| ConnectError::internal(format!("account.login_complete: {e}")))?;
        let value = json!({ "user_id": me.id });
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
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
