//! `ViewService` handler — proto-as-SSOT migration of the `view.*` authoring
//! writes (#89). Mounted additively beside the generated CommandOpsService
//! until the GUI flip (#72).
//!
//! Every op is a C1 view-authoring write. Each routes through
//! `services::command::call`, the ONE gate, which enforces `ship.views.write`
//! and the per-session edit-mode grant exactly as the generated CommandOps does
//! — and dispatches to `services::command::views`, which owns the logic
//! (session fragment edits, edit-mode grant/revoke, authoring-tool dispatch).
//! The returns are opaque, so each wraps the command JSON in `JsonResponse`,
//! mirroring the generated path's shape for a clean transport swap.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;
use ship_ops_rpc::{JsonResponse, ViewService};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl ViewService for OpsImpl {
    async fn patch_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewPatchReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id, "selector": request.selector, "html": request.html });
        self.gated_view_json(&ctx, "view.patch_session", args).await
    }

    async fn append_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewAppendReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id, "selector": request.selector, "html": request.html });
        self.gated_view_json(&ctx, "view.append_session", args).await
    }

    async fn signal_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewSignalReqView,
    ) -> ServiceResult<JsonResponse> {
        let args =
            json!({ "id": request.id, "name": request.name, "value_json": request.value_json });
        self.gated_view_json(&ctx, "view.signal_session", args).await
    }

    async fn enter_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "view": request.view, "session_id": request.session_id });
        self.gated_view_json(&ctx, "view.enter_edit_mode", args).await
    }

    async fn exit_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "view": request.view, "session_id": request.session_id });
        self.gated_view_json(&ctx, "view.exit_edit_mode", args).await
    }

    async fn tool(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewToolReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "view": request.view, "session_id": request.session_id,
            "op": request.op, "args_json": request.args_json,
        });
        self.gated_view_json(&ctx, "view.tool", args).await
    }
}

impl OpsImpl {
    /// Route a `view.*` write through the one gate and wrap the opaque result in
    /// `JsonResponse`. The gate reads blast/capability from the descriptor; the
    /// agent-identity binding lives in `services::command::views`.
    async fn gated_view_json(
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
    /// Every `view.*` write declares `ship.views.write` / C1 in the compiled
    /// descriptor — the field-level policy the gate reads at runtime. Matching
    /// the generated CommandOps `view.*` ops is the proto-as-SSOT contract.
    #[test]
    fn view_rpc_op_policy_in_descriptor() {
        for method in [
            "PatchSession",
            "AppendSession",
            "SignalSession",
            "EnterEditMode",
            "ExitEditMode",
            "Tool",
        ] {
            let path = format!("ship.ops.v1.ViewService/{method}");
            let p = crate::ops_rpc::policy_for(&path)
                .unwrap_or_else(|| panic!("{path} must carry a policy in the descriptor"));
            assert_eq!(p.capability, "ship.views.write");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space);
        }
    }
}
