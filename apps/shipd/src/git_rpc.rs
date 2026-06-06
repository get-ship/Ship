//! `GitService` handler — proto-as-SSOT migration of `git.*` (#89), mounted
//! additively beside the generated ReadOps/CommandOps until the GUI flip (#72).
//!
//! Reads (C0) call ship-git crate functions directly. Writes (C1) route
//! through `services::command::call`, the ONE gate.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::Value;

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
        let ws = None::<&str>;
        let report = self
            .state()
            .git()
            .status(ws)
            .map_err(|e| ConnectError::internal(format!("git.status: {e}")))?;
        let value = serde_json::to_value(&report)
            .map_err(|e| ConnectError::internal(format!("git.status serialize: {e}")))?;
        serde_json::from_value::<StatusBuckets>(value)
            .map_err(|e| ConnectError::internal(format!("git.status decode: {e}")))
            .map(Response::new)
    }

    async fn log(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitLogReqView,
    ) -> ServiceResult<GitLogResponse> {
        let limit = if request.limit != 0 { request.limit as usize } else { 50 };
        let commits = self
            .state()
            .git()
            .log(None, None, limit)
            .map_err(|e| ConnectError::internal(format!("git.log: {e}")))?;
        let value = serde_json::to_value(&commits)
            .map_err(|e| ConnectError::internal(format!("git.log serialize: {e}")))?;
        serde_json::from_value::<GitLogResponse>(serde_json::json!({ "commits": value }))
            .map_err(|e| ConnectError::internal(format!("git.log decode: {e}")))
            .map(Response::new)
    }

    async fn branches(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGitBranchesReqView,
    ) -> ServiceResult<GitBranchesResponse> {
        let git = self.state().git();
        let branches = git
            .branches(None)
            .map_err(|e| ConnectError::internal(format!("git.branches: {e}")))?;
        let (ahead, behind) = branches
            .iter()
            .find(|b| b.active)
            .map(|b| (b.ahead, b.behind))
            .unwrap_or((0, 0));
        let value = serde_json::to_value(&branches)
            .map_err(|e| ConnectError::internal(format!("git.branches serialize: {e}")))?;
        serde_json::from_value::<GitBranchesResponse>(serde_json::json!({
            "branches": value,
            "ahead": ahead,
            "behind": behind,
        }))
        .map_err(|e| ConnectError::internal(format!("git.branches decode: {e}")))
        .map(Response::new)
    }

    async fn overview(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitOverviewReqView,
    ) -> ServiceResult<GitOverviewResponse> {
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty());
        let limit = if request.limit != 0 { request.limit as usize } else { 20 };
        let value = git_overview_value(self.state(), ws, limit)
            .map_err(|e| ConnectError::internal(format!("git.overview: {e}")))?;
        serde_json::from_value::<GitOverviewResponse>(value)
            .map_err(|e| ConnectError::internal(format!("git.overview decode: {e}")))
            .map(Response::new)
    }

    #[allow(refining_impl_trait)]
    async fn overview_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitOverviewReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let ws = Some(request.workspace_id)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let limit = if request.limit != 0 { request.limit as usize } else { 20 };
        self.live_read_with(move |st| {
            let ws_ref = ws.as_deref();
            git_overview_value(st, ws_ref, limit).unwrap_or(Value::Null)
        })
        .await
    }

    #[allow(refining_impl_trait)]
    async fn diff_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitDiffReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let commit = request.commit.clone();
        let ws = Some(request.workspace_id)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        self.live_read_with(move |st| {
            let ws_ref = ws.as_deref();
            git_diff_value(st, ws_ref, &commit).unwrap_or(Value::Null)
        })
        .await
    }

    #[allow(refining_impl_trait)]
    async fn annotations_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitAnnotationsReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let file = request.file.clone();
        let ws = Some(request.workspace_id)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        self.live_read_with(move |st| {
            let ws_ref = ws.as_deref();
            git_annotations_value(st, ws_ref, &file)
        })
        .await
    }

    async fn diff(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitDiffReqView,
    ) -> ServiceResult<GitDiffResponse> {
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty());
        let value = git_diff_value(self.state(), ws, &request.commit)
            .map_err(|e| ConnectError::internal(format!("git.diff: {e}")))?;
        serde_json::from_value::<GitDiffResponse>(value)
            .map_err(|e| ConnectError::internal(format!("git.diff decode: {e}")))
            .map(Response::new)
    }

    async fn annotations(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitAnnotationsReqView,
    ) -> ServiceResult<JsonResponse> {
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty());
        let value = git_annotations_value(self.state(), ws, &request.file);
        let json = serde_json::to_string(&value)
            .map_err(|e| ConnectError::internal(format!("git.annotations serialize: {e}")))?;
        Ok(Response::new(JsonResponse { json, ..Default::default() }))
    }

    async fn push(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitPushReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        args.insert("workspace_id".to_string(), serde_json::json!(request.workspace_id));
        args.insert("remote".to_string(), serde_json::json!(request.remote));
        if let Some(f) = request.force {
            args.insert("force".to_string(), serde_json::json!(f));
        }
        self.gated_git_write("git.push", Value::Object(args), &ctx).await
    }

    async fn fetch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitFetchReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({ "workspace_id": request.workspace_id, "remote": request.remote });
        self.gated_git_write("git.fetch", args, &ctx).await
    }

    async fn pull(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitPullReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({ "workspace_id": request.workspace_id, "remote": request.remote });
        self.gated_git_write("git.pull", args, &ctx).await
    }

    async fn stage(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitStageReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({
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
        let args = serde_json::json!({
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
        let args = serde_json::json!({
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
        let args = serde_json::json!({ "workspace_id": request.workspace_id, "refname": request.refname });
        self.gated_git_write("git.checkout", args, &ctx).await
    }

    async fn branch_create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitBranchCreateReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        args.insert("workspace_id".to_string(), serde_json::json!(request.workspace_id));
        args.insert("name".to_string(), serde_json::json!(request.name));
        if let Some(c) = request.checkout {
            args.insert("checkout".to_string(), serde_json::json!(c));
        }
        self.gated_git_write("git.branch_create", Value::Object(args), &ctx).await
    }

    async fn branch_delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitBranchDeleteReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        args.insert("workspace_id".to_string(), serde_json::json!(request.workspace_id));
        args.insert("name".to_string(), serde_json::json!(request.name));
        if let Some(f) = request.force {
            args.insert("force".to_string(), serde_json::json!(f));
        }
        self.gated_git_write("git.branch_delete", Value::Object(args), &ctx).await
    }

    async fn init(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitInitReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({ "workspace_id": request.workspace_id });
        self.gated_git_write("git.init", args, &ctx).await
    }
}

// ── git read helpers ─────────────────────────────────────────────────────────

/// Build the git.overview JSON value from ship-git crate calls.
fn git_overview_value(
    state: &crate::ship_runtime::ApiState,
    workspace_id: Option<&str>,
    limit: usize,
) -> Result<Value, String> {
    let git = state.git();
    let status = git.status(workspace_id).map_err(|e| e.to_string())?;
    let commits = git.log(workspace_id, None, limit).map_err(|e| e.to_string())?;
    let branches = git.branches(workspace_id).map_err(|e| e.to_string())?;
    let (ahead, behind) = branches
        .iter()
        .find(|b| b.active)
        .map(|b| (b.ahead, b.behind))
        .unwrap_or((0, 0));
    serde_json::to_value(serde_json::json!({
        "workspace_id": workspace_id,
        "branch": status.branch,
        "head_short": status.head,
        "clean": status.clean,
        "total_changed": status.total,
        "staged": status.staged,
        "modified": status.modified,
        "untracked": status.untracked,
        "commits": commits,
        "branches": branches,
        "ahead": ahead,
        "behind": behind,
    }))
    .map_err(|e| e.to_string())
}

/// Build the git.diff JSON value from ship-git crate calls.
fn git_diff_value(
    state: &crate::ship_runtime::ApiState,
    workspace_id: Option<&str>,
    commit: &str,
) -> Result<Value, String> {
    let report = state.git().diff(workspace_id, Some(commit), None).map_err(|e| e.to_string())?;
    let is_working = report.commit.eq_ignore_ascii_case("WORKING");
    let short_sha: String = report.commit.chars().take(7).collect();
    let parsed = ship_git::diff::parse_unified_diff(&report.diff);
    serde_json::to_value(serde_json::json!({
        "sha": report.commit,
        "short_sha": short_sha,
        "is_working": is_working,
        "diff_text": report.diff,
        "parsed": parsed,
    }))
    .map_err(|e| e.to_string())
}

/// Build the git.annotations JSON value.
///
/// The annotations feature queries the built-in `annotations` collection for
/// comments pinned to diff lines in the git view. Relocated from
/// `ship_git::annotations` during the L0 collapse (ship-collection dep removed
/// from ship-git). shipd already depends on both ship-git and ship-collection,
/// so this is the natural L1 home.
fn git_annotations_value(
    state: &crate::ship_runtime::ApiState,
    workspace_id: Option<&str>,
    file: &str,
) -> Value {
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use ship_collection::builtin::annotations_schema;
    use ship_collection::projection::SchemaRegistry;
    use ship_collection::service::CollectionService;
    use ship_substrate::ports::{
        KernelLog, Predicate, Query, SingleBackendDataStore, Sort, SpaceContext, SpaceId,
    };
    use serde::Serialize;

    #[derive(Clone, Serialize)]
    struct GitAnnotation {
        id: String,
        selector: String,
        note: Option<String>,
        intent: Option<String>,
        author_kind: String,
        created_at: String,
    }
    type AnnotationsByLine = BTreeMap<String, Vec<GitAnnotation>>;

    let kernel = state.kernel().clone();
    let ship_dir = state.ship_dir().to_path_buf();

    let data_svc = match ship_substrate::data::DataService::open(&ship_dir) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "failed to open data store for git annotations");
            return Value::Null;
        }
    };
    let store = Arc::new(SingleBackendDataStore::new(data_svc, "global"));
    let kernel_log = Arc::new(KernelLog::new(kernel));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new("global"), store, kernel_log));
    let registry = SchemaRegistry::new();
    registry.insert(annotations_schema());
    let svc = CollectionService { ctx, schemas: Arc::new(registry) };

    let view_id = match workspace_id.filter(|s| !s.is_empty() && *s != "project") {
        Some(id) => format!("git:{id}"),
        None => "git".to_string(),
    };
    let q = Query {
        table: format!("collection_annotations"),
        filters: vec![
            Predicate::eq("file_path", serde_json::Value::String(file.to_string())),
            Predicate::eq("view_id", serde_json::Value::String(view_id)),
        ],
        sort: Some(Sort { column: "created_at".into(), descending: false }),
        limit: 200,
        after: None,
    };

    let items = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(svc.query_items_by_query(q))
    });
    let items = match items {
        Ok((rows, _)) => rows,
        Err(e) => {
            tracing::warn!(error = %e, file, "failed to load git annotations");
            return Value::Null;
        }
    };

    let mut out: AnnotationsByLine = BTreeMap::new();
    for item in items {
        let data = &item.data;
        let selector = match data.get("selector").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let annotation = GitAnnotation {
            id: item.id.clone(),
            selector: selector.clone(),
            note: data.get("note").and_then(|v| v.as_str()).map(str::to_string),
            intent: data.get("intent").and_then(|v| v.as_str()).map(str::to_string),
            author_kind: data
                .get("author_kind")
                .and_then(|v| v.as_str())
                .unwrap_or("human")
                .to_string(),
            created_at: item.created_at.clone(),
        };
        out.entry(selector).or_default().push(annotation);
    }
    serde_json::to_value(&out).unwrap_or(Value::Null)
}

impl OpsImpl {
    async fn gated_git_write(
        &self,
        op: &str,
        args: Value,
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
