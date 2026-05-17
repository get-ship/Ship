//! GitService — HTTP surface at `/api/v1/git/*`.
//!
//! Canonical shape per `.artifacts/daemon-services.md` §3.GitService:
//!
//! | GET    /api/v1/git/status          | bucketed working tree         |
//! | GET    /api/v1/git/diff            | ?commit=X&path=Y              |
//! | GET    /api/v1/git/log             | ?branch=X&limit=N             |
//! | GET    /api/v1/git/branches        |                               |
//! | POST   /api/v1/git/stage           | {paths}                       |
//! | POST   /api/v1/git/unstage         | {paths}                       |
//! | POST   /api/v1/git/commit          | {message, author?}            |
//! | POST   /api/v1/git/push            | {remote?, force?}             |
//! | POST   /api/v1/git/fetch           |                               |
//! | POST   /api/v1/git/pull            | ff-only; 409 on non-ff        |
//! | POST   /api/v1/git/checkout        | {ref}                         |
//! | GET    /api/v1/git/commit-draft    |                               |
//! | PUT    /api/v1/git/commit-draft    | agent-written                 |
//! | DELETE /api/v1/git/commit-draft    |                               |
//! | GET    /api/v1/git/sse             | change stream                 |
//!
//! All endpoints operate on the active workspace / project root resolved
//! via `ship_platform::services::git::state::resolve_root`. Optional `?workspace_id=` query param lets
//! callers pin a specific worktree.


use ship_platform::services::git::service::{
    BranchCreateArgs, BranchDeleteArgs, CheckoutArgs, CommitArgs, PushArgs,
};

use std::path::Path;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use std::time::Duration;

use crate::access::RuntimeAccess;
use crate::routes::rest_api::ApiState;
use crate::views::stream::{ViewSse, ViewStream};
use ship_platform::services::git::state::GitError;

type Resp = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, code: &str, message: impl Into<String>) -> Resp {
    (status, Json(serde_json::json!({
        "ok": false, "error": code, "message": message.into(),
    })))
}

fn map_err(e: GitError) -> Resp {
    match e {
        GitError::RepoUnavailable(m) => err(StatusCode::NOT_FOUND, "no_repo", m),
        GitError::BadRef(m) => err(StatusCode::BAD_REQUEST, "bad_ref", m),
        GitError::NothingStaged => err(StatusCode::BAD_REQUEST, "nothing_staged",
            "no staged changes — stage files before committing"),
        GitError::NonFastForward { ahead, behind } => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "ok": false,
                "error": "non_ff",
                "message": "pull would not be fast-forward",
                "ahead": ahead,
                "behind": behind,
            })),
        ),
        GitError::Git(m) => err(StatusCode::INTERNAL_SERVER_ERROR, "git", m),
        GitError::Io(m) => err(StatusCode::INTERNAL_SERVER_ERROR, "io", m),
    }
}

// ── query shapes ─────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
pub struct WorkspaceQuery {
    #[serde(default)]
    pub workspace_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DiffQuery {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct LogQuery {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

// ── body shapes ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PathsBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Deserialize)]
pub struct CommitBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub message: String,
    #[serde(default)]
    pub author: Option<String>,
}

#[derive(Deserialize)]
pub struct PushBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub remote: Option<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize, Default)]
pub struct RemoteBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub remote: Option<String>,
}

#[derive(Deserialize)]
pub struct CheckoutBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(rename = "ref")]
    pub refname: String,
}

#[derive(Deserialize)]
pub struct BranchCreateBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub checkout: bool,
}

#[derive(Deserialize)]
pub struct BranchDeleteBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize, Default)]
pub struct InitBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
}


// ── helpers ──────────────────────────────────────────────────────────────

fn resolve(state: &ApiState, workspace_id: Option<&str>) -> Result<std::path::PathBuf, Resp> {
    ship_platform::services::git::state::resolve_root(state.ship_dir(), workspace_id)
        .map_err(|e| err(StatusCode::NOT_FOUND, "no_repo", e))
}

fn current_branch(root: &Path) -> Option<String> {
    ship_platform::services::git::read::load_branch(root)
}

// ── read handlers ────────────────────────────────────────────────────────

/// GET /api/v1/git/status
pub async fn status_handler(
    State(state): State<ApiState>,
    Query(q): Query<WorkspaceQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let r = state.services().git().status(q.workspace_id.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "branch": r.branch,
        "head": r.head,
        "ahead": r.ahead,
        "behind": r.behind,
        "clean": r.clean,
        "total": r.total,
        "staged": r.staged,
        "modified": r.modified,
        "untracked": r.untracked,
    })))
}

/// GET /api/v1/git/diff?commit=X&path=Y   (commit=WORKING for working-tree)
pub async fn diff_handler(
    State(state): State<ApiState>,
    Query(q): Query<DiffQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let r = state
        .git()
        .diff(q.workspace_id.as_deref(), q.commit.as_deref(), q.path.as_deref())
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "commit": r.commit,
        "path": r.path,
        "diff": r.diff,
    })))
}

/// GET /api/v1/git/log?branch=X&limit=N
pub async fn log_handler(
    State(state): State<ApiState>,
    Query(q): Query<LogQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let limit = q.limit.unwrap_or(ship_platform::services::git::read::DEFAULT_LOG_LIMIT);
    let commits = state
        .git()
        .log(q.workspace_id.as_deref(), q.branch.as_deref(), limit)
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "commits": commits })))
}

/// GET /api/v1/git/branches
pub async fn branches_handler(
    State(state): State<ApiState>,
    Query(q): Query<WorkspaceQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let branches = state.services().git().branches(q.workspace_id.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "branches": branches })))
}

// ── write handlers ───────────────────────────────────────────────────────

/// POST /api/v1/git/stage
pub async fn stage_handler(
    State(state): State<ApiState>,
    Json(body): Json<PathsBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let n = state.services().git().stage(body.workspace_id.as_deref(), &body.paths).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "staged": n })))
}

/// POST /api/v1/git/unstage
pub async fn unstage_handler(
    State(state): State<ApiState>,
    Json(body): Json<PathsBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let n = state.services().git().unstage(body.workspace_id.as_deref(), &body.paths).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "unstaged": n })))
}

/// POST /api/v1/git/commit
pub async fn commit_handler(
    State(state): State<ApiState>,
    Json(body): Json<CommitBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let res = state
        .git()
        .commit(CommitArgs {
            workspace_id: body.workspace_id,
            message: body.message,
            author: body.author,
        })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "sha": res.sha,
        "branch": res.branch,
    })))
}

/// POST /api/v1/git/push
pub async fn push_handler(
    State(state): State<ApiState>,
    Json(body): Json<PushBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let force = body.force;
    let remote = state
        .git()
        .push(PushArgs {
            workspace_id: body.workspace_id,
            remote: body.remote,
            force,
        })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "remote": remote, "force": force })))
}

/// POST /api/v1/git/fetch
pub async fn fetch_handler(
    State(state): State<ApiState>,
    Json(body): Json<RemoteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let remote = state
        .git()
        .fetch(body.workspace_id.as_deref(), body.remote.as_deref())
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "remote": remote })))
}

/// POST /api/v1/git/pull — ff-only
pub async fn pull_handler(
    State(state): State<ApiState>,
    Json(body): Json<RemoteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let sha = state
        .git()
        .pull(body.workspace_id.as_deref(), body.remote.as_deref())
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "head": sha })))
}

/// POST /api/v1/git/checkout
pub async fn checkout_handler(
    State(state): State<ApiState>,
    Json(body): Json<CheckoutBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let refname = body.refname.clone();
    let resolved = state
        .git()
        .checkout(CheckoutArgs {
            workspace_id: body.workspace_id,
            refname: refname.clone(),
        })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "ref": refname, "resolved": resolved,
    })))
}

/// POST /api/v1/git/branch — create branch from HEAD; optional checkout.
pub async fn branch_create_handler(
    State(state): State<ApiState>,
    Json(body): Json<BranchCreateBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let name = state
        .git()
        .branch_create(BranchCreateArgs {
            workspace_id: body.workspace_id,
            name: body.name,
            checkout: body.checkout,
        })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "name": name })))
}

/// DELETE /api/v1/git/branch — delete a local branch (force-unmerged via `force=true`).
pub async fn branch_delete_handler(
    State(state): State<ApiState>,
    Json(body): Json<BranchDeleteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let name = state
        .git()
        .branch_delete(BranchDeleteArgs {
            workspace_id: body.workspace_id,
            name: body.name,
            force: body.force,
        })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "name": name })))
}

/// POST /api/v1/git/init — initialize a repo at the active workspace root.
pub async fn init_handler(
    State(state): State<ApiState>,
    Json(body): Json<InitBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    state.git().init(body.workspace_id.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── change stream ────────────────────────────────────────────────────────

/// GET /api/v1/git/sse — polls status at 5s cadence; patches `#git-service-data`
/// with a JSON blob inside a `<script>` tag so any view can subscribe to the
/// server-side truth without re-running its own libgit2 queries.
pub async fn sse_handler(
    State(state): State<ApiState>,
    Query(q): Query<WorkspaceQuery>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();
    let workspace_id = q.workspace_id.clone();

    tokio::spawn(async move {
        let _hb = vs.heartbeat_every(Duration::from_secs(15));
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            if vs.is_closed() { break; }
            let root = match ship_platform::services::git::state::resolve_root(&ship_dir, workspace_id.as_deref()) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let buckets = ship_platform::services::git::read::load_status_buckets(&root);
            let (ahead, behind) = ship_platform::services::git::read::load_ahead_behind(&root).unwrap_or((0, 0));
            let body = serde_json::json!({
                "branch": ship_platform::services::git::read::load_branch(&root),
                "ahead": ahead,
                "behind": behind,
                "clean": buckets.clean,
                "total": buckets.total,
            });
            let html = format!(
                r#"<script id="git-service-data" type="application/json">{}</script>"#,
                body,
            );
            if vs.patch("#git-service-data", html).await.is_err() { break; }
        }
    });

    sse
}

