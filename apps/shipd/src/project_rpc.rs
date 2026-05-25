//! `ProjectService` handler — proto-as-SSOT migration of `project.*` and
//! `workspaces.*` (#89), mounted additively beside the generated
//! ReadOps/CommandOps until the GUI flip (#72).
//!
//! Reads (C0) call the sync read registry (`registry::call`) and wrap JSON in
//! `JsonResponse` or decode onto the typed `WorkspaceActiveResponse`. Writes
//! (C1) route through `services::command::call`, the ONE gate, which applies
//! `gate::decide` exactly as the generated CommandOpsService does.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::{json, Value};

use ship_ops_rpc::{JsonResponse, ProjectRecord, ProjectService, WorkspaceActiveResponse};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

/// Shared gated dispatch for project writes. Free function to avoid naming
/// collision with `gated_mutation` on `impl OpsImpl` in `rules_rpc`.
async fn project_gated_call(
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
impl ProjectService for OpsImpl {
    async fn list_projects(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedProjectsListRequestView,
    ) -> ServiceResult<JsonResponse> {
        let value =
            crate::services::registry::call("workspaces.list", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("workspaces.list: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn active_project(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedProjectsActiveRequestView,
    ) -> ServiceResult<WorkspaceActiveResponse> {
        let value =
            crate::services::registry::call("workspaces.active", &Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("workspaces.active: {e}")))?;
        let msg = serde_json::from_value::<WorkspaceActiveResponse>(value)
            .map_err(|e| ConnectError::internal(format!("workspaces.active decode: {e}")))?;
        Ok(Response::new(msg))
    }

    async fn add(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedAddProjectRequestView,
    ) -> ServiceResult<ProjectRecord> {
        let args = json!({ "path": request.path, "name": request.name });
        let value = project_gated_call(self.state(), &ctx, "project.add", args).await?;
        let msg = serde_json::from_value::<ProjectRecord>(value)
            .map_err(|e| ConnectError::internal(format!("decode project.add response: {e}")))?;
        Ok(Response::new(msg))
    }

    async fn activate(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedActivateProjectRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "id": request.id });
        let value = project_gated_call(self.state(), &ctx, "project.activate", args).await?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn project_op_policy_reads_are_c0_safe() {
        for path in [
            "ship.ops.v1.ProjectService/ListProjects",
            "ship.ops.v1.ProjectService/ActiveProject",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.workspaces.read", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read, "{path}");
        }
    }

    #[test]
    fn project_op_policy_writes_are_c1_workspace() {
        for path in [
            "ship.ops.v1.ProjectService/Add",
            "ship.ops.v1.ProjectService/Activate",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.projects.write", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Workspace, "{path}");
        }
    }
}
