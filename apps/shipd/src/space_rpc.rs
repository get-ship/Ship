//! `SpaceService` handler — proto-as-SSOT migration of the space registry
//! surface (#89), mounted additively beside the generated ReadOps/CommandOps
//! until the GUI flip (#72). A space = the unit of isolation/coordination/config
//! (one .ship/ directory).
//!
//! Reads (C0) call the sync read registry (`registry::call`) and wrap JSON in
//! `JsonResponse` or decode onto the typed `WorkspaceActiveResponse` (the shared
//! return shape keeps its current wire name until the umbrella retires, #72).
//! Writes (C1) route through `services::command::call`, the ONE gate, which
//! applies `gate::decide` exactly as the generated CommandOpsService does. The
//! internal dispatch op names (`workspaces.list`, `project.add`, …) are the
//! retiring-umbrella op ids and stay as-is this pass.
//!
//! `list_live`/`active_live` are the streaming C0 reads folded in from the
//! retired `WorkspacesService` (#127): the space list and active-space pointer
//! pushed on each projection change via `live_read`.
//!
//! `info` (#129) is the single-space introspection read: the active space's
//! identity (id/root/name from the projects projection) plus its settings
//! snapshot (`settings.state`). NOTE: `settings.state` is the daemon-global
//! settings table today — per-space settings layering (theme/layout/isolation
//! per space) is not yet wired; when it lands it flows through this same
//! `settings_json` field with no API change. Space-scoped *events* need no op
//! here — they're served by the events surface, now that the emit path stamps
//! `space_id` per connection (#128).

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{
    JsonResponse, ProjectRecord, SpaceInfoResponse, SpaceService, WorkspaceActiveResponse,
};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

/// Shared gated dispatch for space writes. Free function to avoid naming
/// collision with `gated_mutation` on `impl OpsImpl` in `rules_rpc`.
async fn space_gated_call(
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
impl SpaceService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceListRequestView,
    ) -> ServiceResult<JsonResponse> {
        let value =
            crate::services::registry::call("workspaces.list", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("workspaces.list: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn active(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceActiveRequestView,
    ) -> ServiceResult<WorkspaceActiveResponse> {
        let value =
            crate::services::registry::call("workspaces.active", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("workspaces.active: {e}")))?;
        let msg = serde_json::from_value::<WorkspaceActiveResponse>(value)
            .map_err(|e| ConnectError::internal(format!("workspaces.active decode: {e}")))?;
        Ok(Response::new(msg))
    }

    async fn info(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceInfoRequestView,
    ) -> ServiceResult<SpaceInfoResponse> {
        // Identity: the daemon's active/home space, with root+name from the
        // projects projection (the registry of known spaces).
        let space_id = ship_substrate::project::daemon_space_id();
        let (root, name) = match space_id.as_deref() {
            Some(id) => match ship_platform::projections::projects::get(id) {
                Ok(Some(row)) => (Some(row.path), Some(row.name)),
                _ => (None, None),
            },
            None => (None, None),
        };
        // Config: the daemon settings snapshot. `settings.state` is global today;
        // per-space layering lands later and flows through this same field.
        let settings = crate::services::registry::call("settings.state", &Value::Null, self.state())
            .map_err(|e| ConnectError::internal(format!("settings.state: {e}")))?;
        Ok(Response::new(SpaceInfoResponse {
            space_id,
            root,
            name,
            settings_json: settings.to_string(),
            ..Default::default()
        }))
    }

    async fn add(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedAddSpaceRequestView,
    ) -> ServiceResult<ProjectRecord> {
        let args = json!({ "path": request.path, "name": request.name });
        let value = space_gated_call(self.state(), &ctx, "project.add", args).await?;
        let msg = serde_json::from_value::<ProjectRecord>(value)
            .map_err(|e| ConnectError::internal(format!("decode project.add response: {e}")))?;
        Ok(Response::new(msg))
    }

    async fn activate(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedActivateSpaceRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id });
        let value = space_gated_call(self.state(), &ctx, "project.activate", args).await?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    #[allow(refining_impl_trait)]
    async fn list_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceListRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("workspaces.list", json!({})).await
    }

    #[allow(refining_impl_trait)]
    async fn active_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceActiveRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("workspaces.active", json!({})).await
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn space_op_policy_reads_are_c0_safe() {
        for path in [
            "ship.ops.v1.SpaceService/List",
            "ship.ops.v1.SpaceService/Active",
            "ship.ops.v1.SpaceService/Info",
            "ship.ops.v1.SpaceService/ListLive",
            "ship.ops.v1.SpaceService/ActiveLive",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.workspaces.read", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read, "{path}");
        }
    }

    #[test]
    fn space_op_policy_writes_are_c1_workspace() {
        for path in [
            "ship.ops.v1.SpaceService/Add",
            "ship.ops.v1.SpaceService/Activate",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.projects.write", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Workspace, "{path}");
        }
    }
}
