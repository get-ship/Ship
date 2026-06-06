//! `ViewService` handler — the single code path for all `view.*` authoring
//! writes. The string-command dispatch layer is gone; each handler calls the
//! `ship_views::view_authoring` crate and the route-layer cores directly.
//!
//! Gate: every op is C1_SPACE (`ship.views.write`). C1 is the operational
//! baseline — any authenticated agent may execute it. The `AuthContext` is
//! still required (the middleware guarantees it); agent-identity binding is
//! enforced per handler where the op is identity-scoped.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_ops_rpc::{JsonResponse, ViewService};

use crate::auth::AuthContext;
use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl ViewService for OpsImpl {
    async fn patch_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewPatchReqView,
    ) -> ServiceResult<JsonResponse> {
        let auth = auth_from(&ctx)?;
        let agent = require_agent(auth)?;
        self.state()
            .view_sessions()
            .patch(&request.id, &agent, &request.selector, request.html.to_string())
            .await
            .map_err(|e| ConnectError::internal(e.to_string()))?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn append_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewAppendReqView,
    ) -> ServiceResult<JsonResponse> {
        let auth = auth_from(&ctx)?;
        let agent = require_agent(auth)?;
        self.state()
            .view_sessions()
            .append(&request.id, &agent, &request.selector, request.html.to_string())
            .await
            .map_err(|e| ConnectError::internal(e.to_string()))?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn signal_session(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewSignalReqView,
    ) -> ServiceResult<JsonResponse> {
        let auth = auth_from(&ctx)?;
        let agent = require_agent(auth)?;
        let value: serde_json::Value = if request.value_json.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&request.value_json)
                .map_err(|e| ConnectError::invalid_argument(format!("value_json: {e}")))?
        };
        self.state()
            .view_sessions()
            .signal(&request.id, &agent, &request.name, value)
            .await
            .map_err(|e| ConnectError::internal(e.to_string()))?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    async fn enter_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        let _auth = auth_from(&ctx)?;
        let view = request.view.to_string();
        let session_id = request.session_id.to_string();
        if !is_valid_view_name(&view) {
            return Err(ConnectError::invalid_argument("invalid view name"));
        }
        ship_views::view_authoring::grant(&session_id, &view);
        json_ok(serde_json::json!({ "view": view, "session_id": session_id, "granted": true }))
    }

    async fn exit_edit_mode(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewEditModeReqView,
    ) -> ServiceResult<JsonResponse> {
        let _auth = auth_from(&ctx)?;
        let view = request.view.to_string();
        let session_id = request.session_id.to_string();
        if !is_valid_view_name(&view) {
            return Err(ConnectError::invalid_argument("invalid view name"));
        }
        // Idempotent — revoking a missing grant is fine.
        ship_views::view_authoring::revoke(&session_id, &view);
        json_ok(serde_json::json!({ "view": view, "session_id": session_id, "granted": false }))
    }

    async fn tool(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedViewToolReqView,
    ) -> ServiceResult<JsonResponse> {
        let _auth = auth_from(&ctx)?;
        let args_value: serde_json::Value = if request.args_json.is_empty() {
            serde_json::Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&request.args_json)
                .map_err(|e| ConnectError::invalid_argument(format!("args_json: {e}")))?
        };
        let body = crate::routes::view_edit_tool_api::ToolBody {
            session_id: request.session_id.to_string(),
            op: request.op.to_string(),
            args: args_value,
        };
        let resp =
            crate::routes::view_edit_tool_api::dispatch_tool_core(self.state(), request.view.to_string(), body)
                .await;
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
            .await
            .map_err(|e| ConnectError::internal(e.to_string()))?;
        if status.is_success() {
            let json = String::from_utf8_lossy(&bytes).into_owned();
            Ok(Response::new(JsonResponse { json, ..Default::default() }))
        } else {
            let msg = String::from_utf8_lossy(&bytes).into_owned();
            if status.is_client_error() {
                Err(ConnectError::invalid_argument(msg))
            } else {
                Err(ConnectError::internal(msg))
            }
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn auth_from(ctx: &RequestContext) -> Result<&AuthContext, ConnectError> {
    ctx.extensions
        .get::<AuthContext>()
        .ok_or_else(|| ConnectError::unauthenticated("auth context required"))
}

fn require_agent(auth: &AuthContext) -> Result<String, ConnectError> {
    auth.agent_id
        .clone()
        .ok_or_else(|| ConnectError::permission_denied("agent identity required"))
}

/// Identical validation used by the REST edit-mode routes — view names are
/// ASCII alphanumeric + `-` / `_`, max 128 chars.
fn is_valid_view_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn json_ok(value: serde_json::Value) -> ServiceResult<JsonResponse> {
    Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
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
