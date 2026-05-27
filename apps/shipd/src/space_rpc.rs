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
//! identity (id/root/name from the projects projection) plus its effective
//! settings snapshot. The settings are layered global ⟵ space: the daemon-global
//! `settings.state` object is the base, overlaid per-key by the space's
//! event-sourced overrides (`space_settings::get_all`). The merged object is
//! serialized into `settings_json` — space overrides win per-key. Space-scoped
//! *events* need no op here — they're served by the events surface, now that the
//! emit path stamps `space_id` per connection (#128).
//!
//! `set_setting` (#129) is the per-space override write: it routes through the
//! one gate (`space_gated_call` → `space.setting.set`), which emits
//! `space.setting.changed` with the caller's space riding the envelope.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{
    JsonResponse, ProjectRecord, SettingValue, SpaceInfoResponse, SpaceService,
    WorkspaceActiveResponse,
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
        // Config: the effective settings, layered global ⟵ space. The daemon
        // settings snapshot (`settings.state`) is the base; the space's
        // event-sourced overrides overlay it per-key (space wins). Global
        // `settings.state` itself is untouched — the overlay is read-time.
        let mut settings =
            crate::services::registry::call("settings.state", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("settings.state: {e}")))?;
        if let (Some(id), Some(obj)) = (space_id.as_deref(), settings.as_object_mut())
            && let Ok(overrides) = ship_platform::projections::space_settings::get_all(id)
        {
            for (key, value) in overrides {
                obj.insert(key, value);
            }
        }
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

    async fn set_setting(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSetSpaceSettingRequestView,
    ) -> ServiceResult<SettingValue> {
        let args = json!({ "key": request.key, "value_json": request.value_json });
        let value = space_gated_call(self.state(), &ctx, "space.setting.set", args).await?;
        let msg = serde_json::from_value::<SettingValue>(value).map_err(|e| {
            ConnectError::internal(format!("decode space.setting.set response: {e}"))
        })?;
        Ok(Response::new(msg))
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

    #[test]
    fn space_op_policy_set_setting_is_c1_workspace_settings_write() {
        // The per-space override write (#129) is a settings capability (it sets a
        // settings value), C1 like settings.set — distinct from the projects.write
        // registry writes above.
        let p = policy_for("ship.ops.v1.SpaceService/SetSetting")
            .expect("SpaceService/SetSetting must carry a policy in the descriptor");
        assert_eq!(p.capability, "ship.settings.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Workspace);
    }
}
