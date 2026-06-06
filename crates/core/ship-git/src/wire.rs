//! ConnectRPC wire layer for `GitService`.
//!
//! `GitWire` holds:
//!   - `Arc<dyn GitService>` — the domain read/write service.
//!   - `gate: GateFn` — the shared gate closure (enforces C1 capability on writes).
//!   - `kernel: Arc<KernelLog>` — for live-read kernel subscriptions.
//!   - `write_dispatch: WriteFn` — an async closure that maps an op name + args +
//!     request context to a JSON value, wrapping `command::call` in shipd so the
//!     crate doesn't depend on shipd's internal gate types.
//!   - `collection_annotations: AnnotationsFn` — a closure that resolves git
//!     annotations from the collection service; shipd supplies this since it owns
//!     the active-space context the annotations query needs.
//!
//! Replaces `impl GitService for OpsImpl` in `apps/shipd/git_rpc.rs`.
// FILE_CAP_OVERRIDE: GitService has 14 RPC methods including 3 live streams; one impl block.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::Value;

use ship_ops_rpc::{
    GitBranchesResponse, GitDiffResponse, GitLogResponse, GitOverviewResponse,
    GitService as GitServiceTrait, JsonResponse, StatusBuckets,
};
use ship_substrate::ports::KernelLog;

use crate::service::GitService;

/// Gate closure type.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

/// Async write dispatcher closure: (op_name, args, request_ctx) → Result<Value, ConnectError>.
/// Shipd wires this to `command::call` + `command_error_to_connect`.
pub type WriteFn = Arc<
    dyn Fn(
            String,
            Value,
            RequestContext,
        ) -> Pin<Box<dyn Future<Output = Result<Value, ConnectError>> + Send>>
        + Send
        + Sync,
>;

/// Annotations resolver: (workspace_id, file) → JSON Value.
/// Shipd supplies this since it owns the active-space collection context.
pub type AnnotationsFn =
    Arc<dyn Fn(Option<&str>, &str) -> Value + Send + Sync>;

/// ConnectRPC wire for `GitService`.
pub struct GitWire {
    svc: Arc<dyn GitService>,
    gate: GateFn,
    kernel: Arc<KernelLog>,
    write_dispatch: WriteFn,
    annotations_fn: AnnotationsFn,
}

impl GitWire {
    pub fn new(
        svc: Arc<dyn GitService>,
        gate: GateFn,
        kernel: Arc<KernelLog>,
        write_dispatch: WriteFn,
        annotations_fn: AnnotationsFn,
    ) -> Self {
        Self { svc, gate, kernel, write_dispatch, annotations_fn }
    }
}

// ── read helpers ─────────────────────────────────────────────────────────────

fn overview_value(svc: &Arc<dyn GitService>, workspace_id: Option<&str>, limit: usize)
    -> Result<Value, String>
{
    let status = svc.status(workspace_id).map_err(|e| e.to_string())?;
    let commits = svc.log(workspace_id, None, limit).map_err(|e| e.to_string())?;
    let branches = svc.branches(workspace_id).map_err(|e| e.to_string())?;
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

fn diff_value(svc: &Arc<dyn GitService>, workspace_id: Option<&str>, commit: &str)
    -> Result<Value, String>
{
    let report = svc.diff(workspace_id, Some(commit), None).map_err(|e| e.to_string())?;
    let is_working = report.commit.eq_ignore_ascii_case("WORKING");
    let short_sha: String = report.commit.chars().take(7).collect();
    let parsed = crate::diff::parse_unified_diff(&report.diff);
    serde_json::to_value(serde_json::json!({
        "sha": report.commit,
        "short_sha": short_sha,
        "is_working": is_working,
        "diff_text": report.diff,
        "parsed": parsed,
    }))
    .map_err(|e| e.to_string())
}

/// Open a live stream that re-emits `resolve()` on every `config.*` kernel event.
async fn live_stream(
    kernel: Arc<KernelLog>,
    resolve: impl Fn() -> Value + Send + Sync + 'static,
) -> ServiceResult<ServiceStream<JsonResponse>> {
    let initial = resolve();
    let mut rx = kernel.subscribe();
    let stream = async_stream::stream! {
        if let Ok(json) = serde_json::to_string(&initial) {
            yield Ok::<JsonResponse, ConnectError>(JsonResponse { json, ..Default::default() });
        }
        loop {
            match rx.recv().await {
                Ok(_ev) => {
                    while rx.try_recv().is_ok() {}
                    let value = resolve();
                    if let Ok(json) = serde_json::to_string(&value) {
                        yield Ok(JsonResponse { json, ..Default::default() });
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Ok(Response::new(Box::pin(stream)))
}

// ── write helper ─────────────────────────────────────────────────────────────

async fn gated_write(
    gate: &GateFn,
    write_dispatch: &WriteFn,
    op: &str,
    args: Value,
    ctx: RequestContext,
) -> ServiceResult<JsonResponse> {
    gate(&ctx, op)?;
    let value = write_dispatch(op.to_string(), args, ctx).await?;
    Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl GitServiceTrait for GitWire {
    async fn status(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGitStatusReqView,
    ) -> ServiceResult<StatusBuckets> {
        let ws = None::<&str>;
        let report = self
            .svc
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
            .svc
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
        let branches = self
            .svc
            .branches(None)
            .map_err(|e| ConnectError::internal(format!("git.branches: {e}")))?;
        let (ahead, behind) =
            branches.iter().find(|b| b.active).map(|b| (b.ahead, b.behind)).unwrap_or((0, 0));
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
        let value = overview_value(&self.svc, ws, limit)
            .map_err(|e| ConnectError::internal(format!("git.overview: {e}")))?;
        serde_json::from_value::<GitOverviewResponse>(value)
            .map_err(|e| ConnectError::internal(format!("git.overview decode: {e}")))
            .map(Response::new)
    }

    async fn overview_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitOverviewReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty()).map(str::to_string);
        let limit = if request.limit != 0 { request.limit as usize } else { 20 };
        let svc = self.svc.clone();
        live_stream(self.kernel.clone(), move || {
            let ws_ref = ws.as_deref();
            overview_value(&svc, ws_ref, limit).unwrap_or(Value::Null)
        })
        .await
    }

    async fn diff(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitDiffReqView,
    ) -> ServiceResult<GitDiffResponse> {
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty());
        let value = diff_value(&self.svc, ws, &request.commit)
            .map_err(|e| ConnectError::internal(format!("git.diff: {e}")))?;
        serde_json::from_value::<GitDiffResponse>(value)
            .map_err(|e| ConnectError::internal(format!("git.diff decode: {e}")))
            .map(Response::new)
    }

    async fn diff_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitDiffReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let commit = request.commit.to_string();
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty()).map(str::to_string);
        let svc = self.svc.clone();
        live_stream(self.kernel.clone(), move || {
            let ws_ref = ws.as_deref();
            diff_value(&svc, ws_ref, &commit).unwrap_or(Value::Null)
        })
        .await
    }

    async fn annotations(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitAnnotationsReqView,
    ) -> ServiceResult<JsonResponse> {
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty());
        let value = (self.annotations_fn)(ws, &request.file);
        let json = serde_json::to_string(&value)
            .map_err(|e| ConnectError::internal(format!("git.annotations serialize: {e}")))?;
        Ok(Response::new(JsonResponse { json, ..Default::default() }))
    }

    async fn annotations_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitAnnotationsReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let file = request.file.to_string();
        let ws = Some(request.workspace_id).filter(|s| !s.is_empty()).map(str::to_string);
        let af = self.annotations_fn.clone();
        live_stream(self.kernel.clone(), move || {
            let ws_ref = ws.as_deref();
            af(ws_ref, &file)
        })
        .await
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
        gated_write(&self.gate, &self.write_dispatch, "git.push", Value::Object(args), ctx).await
    }

    async fn fetch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitFetchReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({
            "workspace_id": request.workspace_id,
            "remote": request.remote,
        });
        gated_write(&self.gate, &self.write_dispatch, "git.fetch", args, ctx).await
    }

    async fn pull(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitPullReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({
            "workspace_id": request.workspace_id,
            "remote": request.remote,
        });
        gated_write(&self.gate, &self.write_dispatch, "git.pull", args, ctx).await
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
        gated_write(&self.gate, &self.write_dispatch, "git.stage", args, ctx).await
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
        gated_write(&self.gate, &self.write_dispatch, "git.unstage", args, ctx).await
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
        gated_write(&self.gate, &self.write_dispatch, "git.commit", args, ctx).await
    }

    async fn checkout(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitCheckoutReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({
            "workspace_id": request.workspace_id,
            "refname": request.refname,
        });
        gated_write(&self.gate, &self.write_dispatch, "git.checkout", args, ctx).await
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
        gated_write(
            &self.gate,
            &self.write_dispatch,
            "git.branch_create",
            Value::Object(args),
            ctx,
        )
        .await
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
        gated_write(
            &self.gate,
            &self.write_dispatch,
            "git.branch_delete",
            Value::Object(args),
            ctx,
        )
        .await
    }

    async fn init(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGitInitReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = serde_json::json!({ "workspace_id": request.workspace_id });
        gated_write(&self.gate, &self.write_dispatch, "git.init", args, ctx).await
    }
}
