//! `SettingsService` handler — proto-as-SSOT migration of `settings.*` (#89),
//! mounted additively beside the generated ReadOps/CommandOps until the GUI
//! flip (#72).
//!
//! The read (C0) calls the sync read registry (`registry::call`) and wraps the
//! result in `JsonResponse`. The write (C1) routes through
//! `services::command::call`, the ONE gate, which enforces the grant and
//! decodes the typed `SettingValue` return.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::{json, Value};

use ship_ops_rpc::{JsonResponse, SettingValue, SettingsService};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

/// Gated dispatch for settings writes. Free function to avoid naming collision
/// with helpers on `impl OpsImpl` in other rpc modules.
async fn settings_gated_call(
    state: &crate::ship_runtime::ApiState,
    ctx: &RequestContext,
    op: &str,
    args: Value,
) -> Result<Value, ConnectError> {
    let auth = ctx
        .extensions
        .get::<AuthContext>()
        .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
    let cmd_ctx = CommandCtx { state, auth };
    command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)
}

#[allow(refining_impl_trait)]
impl SettingsService for OpsImpl {
    async fn state(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetSettingsRequestView,
    ) -> ServiceResult<JsonResponse> {
        let value =
            crate::services::registry::call("settings.state", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("settings.state: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn set(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSetSettingRequestView,
    ) -> ServiceResult<SettingValue> {
        let args = json!({ "key": request.key, "value_json": request.value_json });
        let value = settings_gated_call(self.state(), &ctx, "settings.set", args).await?;
        let msg = serde_json::from_value::<SettingValue>(value)
            .map_err(|e| ConnectError::internal(format!("decode settings.set response: {e}")))?;
        Ok(Response::new(msg))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn settings_op_policy_read_is_c0_safe() {
        let p = policy_for("ship.ops.v1.SettingsService/State")
            .expect("SettingsService/State must carry a policy in the descriptor");
        assert_eq!(p.capability, "ship.settings.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }

    #[test]
    fn settings_op_policy_write_is_c1_workspace() {
        let p = policy_for("ship.ops.v1.SettingsService/Set")
            .expect("SettingsService/Set must carry a policy in the descriptor");
        assert_eq!(p.capability, "ship.settings.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Workspace);
    }
}
