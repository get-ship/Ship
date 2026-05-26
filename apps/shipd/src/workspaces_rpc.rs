//! `WorkspacesService` handler — proto-as-SSOT migration of `workspaces.*`
//! (#89), mounted additively beside the generated ReadOps until the GUI flip
//! (#72).
//!
//! Two C0 reads: `workspaces.list` returns opaque JSON (workspace rows embed
//! platform-specific fields); `workspaces.active` decodes onto the typed
//! `WorkspaceActiveResponse` from types.proto.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{JsonResponse, WorkspaceActiveResponse, WorkspacesService};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl WorkspacesService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListWorkspacesReqView,
    ) -> ServiceResult<JsonResponse> {
        let value =
            crate::services::registry::call("workspaces.list", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("workspaces.list: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn active(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetActiveWorkspaceReqView,
    ) -> ServiceResult<WorkspaceActiveResponse> {
        let value =
            crate::services::registry::call("workspaces.active", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("workspaces.active: {e}")))?;
        serde_json::from_value::<WorkspaceActiveResponse>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("workspaces.active decode: {e}")))
    }

    #[allow(refining_impl_trait)]
    async fn list_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListWorkspacesReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("workspaces.list", json!({})).await
    }

    #[allow(refining_impl_trait)]
    async fn active_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetActiveWorkspaceReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("workspaces.active", json!({})).await
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn workspaces_op_policy_is_c0_safe() {
        for path in [
            "ship.ops.v1.WorkspacesService/List",
            "ship.ops.v1.WorkspacesService/Active",
        ] {
            let p =
                policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.workspaces.read", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read, "{path}");
        }
    }
}
