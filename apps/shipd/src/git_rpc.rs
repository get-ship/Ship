//! `GitService` handler — proto-as-SSOT migration of `git.*` (#89), mounted
//! additively beside the generated ReadOps/CommandOps until the GUI flip (#72).
//!
//! Reads (C0) route through the sync read registry (`registry::call`). Writes
//! (C1) route through `services::command::call`, the ONE gate.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde::de::DeserializeOwned;
use serde_json::json;

use ship_ops_rpc::{
    GitBranchesResponse, GitDiffResponse, GitLogResponse, GitOverviewResponse, GitService,
    JsonResponse, StatusBuckets,
};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl GitService for OpsImpl {
    async fn status(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGitStatusReqView,
    ) -> ServiceResult<StatusBuckets> {
        self.read_git_typed("git.status", serde_json::Value::Null)
    }

    async fn log(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitLogReqView,
    ) -> ServiceResult<GitLogResponse> {
        let args = if request.limit != 0 {
            json!({ "limit": request.limit })
        } else {
            serde_json::Value::Null
        };
        self.read_git_typed("git.log", args)
    }

    async fn branches(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGitBranchesReqView,
    ) -> ServiceResult<GitBranchesResponse> {
        self.read_git_typed("git.branches", serde_json::Value::Null)
    }

    async fn overview(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitOverviewReqView,
    ) -> ServiceResult<GitOverviewResponse> {
        let mut args = serde_json::Map::new();
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        if request.limit != 0 {
            args.insert("limit".to_string(), json!(request.limit));
        }
        self.read_git_typed("git.overview", serde_json::Value::Object(args))
    }

    #[allow(refining_impl_trait)]
    async fn overview_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitOverviewReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let mut args = serde_json::Map::new();
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        if request.limit != 0 {
            args.insert("limit".to_string(), json!(request.limit));
        }
        self.live_read("git.overview", serde_json::Value::Object(args)).await
    }

    #[allow(refining_impl_trait)]
    async fn diff_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitDiffReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let mut args = serde_json::Map::new();
        args.insert("commit".to_string(), json!(request.commit));
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        self.live_read("git.diff", serde_json::Value::Object(args)).await
    }

    #[allow(refining_impl_trait)]
    async fn annotations_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitAnnotationsReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let mut args = serde_json::Map::new();
        args.insert("file".to_string(), json!(request.file));
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        self.live_read("git.annotations", serde_json::Value::Object(args)).await
    }

    async fn diff(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitDiffReqView,
    ) -> ServiceResult<GitDiffResponse> {
        let mut args = serde_json::Map::new();
        args.insert("commit".to_string(), json!(request.commit));
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        self.read_git_typed("git.diff", serde_json::Value::Object(args))
    }

    async fn annotations(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitAnnotationsReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        args.insert("file".to_string(), json!(request.file));
        if !request.workspace_id.is_empty() {
            args.insert("workspace_id".to_string(), json!(request.workspace_id));
        }
        let value = crate::services::registry::call(
            "git.annotations",
            &serde_json::Value::Object(args),
            self.state(),
        )
        .map_err(|e| ConnectError::internal(format!("git.annotations: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn push(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitPushReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        args.insert("workspace_id".to_string(), json!(request.workspace_id));
        args.insert("remote".to_string(), json!(request.remote));
        if let Some(f) = request.force {
            args.insert("force".to_string(), json!(f));
        }
        self.gated_git_write("git.push", serde_json::Value::Object(args), &ctx).await
    }

    async fn fetch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitFetchReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "workspace_id": request.workspace_id, "remote": request.remote });
        self.gated_git_write("git.fetch", args, &ctx).await
    }

    async fn pull(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitPullReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "workspace_id": request.workspace_id, "remote": request.remote });
        self.gated_git_write("git.pull", args, &ctx).await
    }

    async fn stage(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitStageReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "workspace_id": request.workspace_id,
            "paths": request.paths.iter().collect::<Vec<_>>(),
        });
        self.gated_git_write("git.stage", args, &ctx).await
    }

    async fn unstage(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitUnstageReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "workspace_id": request.workspace_id,
            "paths": request.paths.iter().collect::<Vec<_>>(),
        });
        self.gated_git_write("git.unstage", args, &ctx).await
    }

    async fn commit(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitCommitReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "workspace_id": request.workspace_id,
            "message": request.message,
            "author": request.author,
        });
        self.gated_git_write("git.commit", args, &ctx).await
    }

    async fn checkout(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitCheckoutReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "workspace_id": request.workspace_id, "refname": request.refname });
        self.gated_git_write("git.checkout", args, &ctx).await
    }

    async fn branch_create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitBranchCreateReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        args.insert("workspace_id".to_string(), json!(request.workspace_id));
        args.insert("name".to_string(), json!(request.name));
        if let Some(c) = request.checkout {
            args.insert("checkout".to_string(), json!(c));
        }
        self.gated_git_write("git.branch_create", serde_json::Value::Object(args), &ctx).await
    }

    async fn branch_delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitBranchDeleteReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        args.insert("workspace_id".to_string(), json!(request.workspace_id));
        args.insert("name".to_string(), json!(request.name));
        if let Some(f) = request.force {
            args.insert("force".to_string(), json!(f));
        }
        self.gated_git_write("git.branch_delete", serde_json::Value::Object(args), &ctx).await
    }

    async fn init(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitInitReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "workspace_id": request.workspace_id });
        self.gated_git_write("git.init", args, &ctx).await
    }
}

impl OpsImpl {
    fn read_git_typed<T: DeserializeOwned>(&self, op: &str, args: serde_json::Value) -> ServiceResult<T> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        serde_json::from_value::<T>(value)
            .map_err(|e| ConnectError::internal(format!("{op} decode: {e}")))
            .map(Response::new)
    }

    async fn gated_git_write(
        &self,
        op: &str,
        args: serde_json::Value,
        ctx: &RequestContext,
    ) -> ServiceResult<JsonResponse> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value = command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn git_op_policy_reads_are_c0_safe() {
        for path in [
            "ship.ops.v1.GitService/Status",
            "ship.ops.v1.GitService/Log",
            "ship.ops.v1.GitService/Branches",
            "ship.ops.v1.GitService/Overview",
            "ship.ops.v1.GitService/Diff",
            "ship.ops.v1.GitService/Annotations",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.git.read", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read, "{path}");
        }
    }

    #[test]
    fn git_op_policy_writes_are_c1_workspace() {
        for path in [
            "ship.ops.v1.GitService/Push",
            "ship.ops.v1.GitService/Fetch",
            "ship.ops.v1.GitService/Pull",
            "ship.ops.v1.GitService/Stage",
            "ship.ops.v1.GitService/Unstage",
            "ship.ops.v1.GitService/Commit",
            "ship.ops.v1.GitService/Checkout",
            "ship.ops.v1.GitService/BranchCreate",
            "ship.ops.v1.GitService/BranchDelete",
            "ship.ops.v1.GitService/Init",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.git.write", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space, "{path}");
        }
    }
}
