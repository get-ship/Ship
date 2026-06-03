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
//!
//! `window_intent` (#125) is the focus-vs-spawn decision for opening a space
//! window: the daemon mediates (it owns window liveness via the actor
//! projection) and returns "focus"+actor_id or "spawn"; the Tauri shell adaptor
//! executes it (only it can touch an OS window). The daemon never spawns.
//!
//! The window control surface makes the daemon authoritative over windows it
//! can't itself spawn: `window_list` (C0) enumerates window actors + pid + live
//! state; `close_window` (C3, operator-only) revokes the window's pid-bound
//! token (instant authority cut) and closes its lifecycle. The shell destroys
//! the OS frame; revocation neuters the webview regardless. A space shell (C2)
//! cannot close another window — that's the C3 boundary.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{json, Value};

use ship_ops_rpc::{
    CloseWindowResponse, JsonResponse, ProjectRecord, SettingValue, SpaceInfoResponse,
    SpaceService, SpaceWindowIntentResponse, SpaceWindowListResponse, WindowInfo,
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

/// Map an L1 `SpaceServiceError` to its `ConnectError` wire shape. The service
/// carries a transport-agnostic kind; this is the daemon's marshal of it.
fn space_err_to_connect(e: ship_platform::services::space::SpaceServiceError) -> ConnectError {
    use ship_platform::services::space::SpaceErrorKind;
    match e.kind {
        SpaceErrorKind::BadArgs => ConnectError::invalid_argument(e.message),
        SpaceErrorKind::NotFound => ConnectError::not_found(e.message),
        SpaceErrorKind::Forbidden => ConnectError::permission_denied(e.message),
        SpaceErrorKind::Conflict => ConnectError::aborted(e.message),
        SpaceErrorKind::Service => ConnectError::internal(e.message),
    }
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
        // The daemon resolves its active/home space and fetches its global
        // settings snapshot (`settings.state`); the L1 space service owns the
        // identity lookup + the effective-settings overlay (global ⟵ space).
        // FLAG: daemon-home read; replace with active-space source (space/shell
        // arc). Non-emit read of the global — deferred past P2 (#3): no
        // active-space source yet; deleting it now breaks `space.info`.
        let space_id = ship_substrate::project::daemon_space_id();
        let global_settings =
            crate::services::registry::call("settings.state", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("settings.state: {e}")))?;
        let info =
            ship_platform::services::space::info::compose(space_id.as_deref(), global_settings);
        Ok(Response::new(SpaceInfoResponse {
            space_id: info.space_id,
            root: info.root,
            name: info.name,
            settings_json: info.settings.to_string(),
            ..Default::default()
        }))
    }

    async fn window_intent(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSpaceWindowIntentRequestView,
    ) -> ServiceResult<SpaceWindowIntentResponse> {
        // Mediation, not initiation: the service consults the actor projection
        // for a live window of this space (focus) or returns spawn. The daemon
        // decides; it never spawns.
        use ship_platform::services::space::windows::{self, WindowIntent};
        let resp = match windows::intent(&request.space_id).map_err(space_err_to_connect)? {
            WindowIntent::Focus { actor_id } => SpaceWindowIntentResponse {
                decision: "focus".to_string(),
                actor_id: Some(actor_id),
                ..Default::default()
            },
            WindowIntent::Spawn => SpaceWindowIntentResponse {
                decision: "spawn".to_string(),
                ..Default::default()
            },
        };
        Ok(Response::new(resp))
    }

    async fn window_list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpaceWindowListRequestView,
    ) -> ServiceResult<SpaceWindowListResponse> {
        // The daemon knows what's open — the service enumerates window actors.
        let rows = ship_platform::services::space::windows::list().map_err(space_err_to_connect)?;
        let windows = rows
            .into_iter()
            .map(|r| WindowInfo {
                actor_id: r.id,
                space_id: r.space_id,
                pid: r.pid,
                live: r.live,
                ..Default::default()
            })
            .collect();
        Ok(Response::new(SpaceWindowListResponse { windows, ..Default::default() }))
    }

    async fn close_window(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCloseWindowRequestView,
    ) -> ServiceResult<CloseWindowResponse> {
        // C3 — the one gate enforces operator-only before the handler runs.
        let args = json!({ "actor_id": request.actor_id });
        let value = space_gated_call(self.state(), &ctx, "space.window.close", args).await?;
        let msg = serde_json::from_value::<CloseWindowResponse>(value).map_err(|e| {
            ConnectError::internal(format!("decode space.window.close response: {e}"))
        })?;
        Ok(Response::new(msg))
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
            "ship.ops.v1.SpaceService/WindowIntent",
            "ship.ops.v1.SpaceService/WindowList",
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

    #[test]
    fn space_op_policy_close_window_is_c3_critical() {
        // Revoking a window's token is operator-only — C3 so a space shell (C2)
        // structurally cannot close another window.
        let p = policy_for("ship.ops.v1.SpaceService/CloseWindow")
            .expect("SpaceService/CloseWindow must carry a policy in the descriptor");
        assert_eq!(p.capability, "ship.projects.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C3Critical);
    }
}
