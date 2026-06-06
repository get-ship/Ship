//! Axum REST handlers for GitService.
//!
//! These handlers take `State<Arc<dyn GitService>>` — they carry no shipd
//! types. Shipd wires them via a sub-router scoped to `state.git_arc()`:
//!
//! ```ignore
//! let git_router = Router::new()
//!     .ship_route(Route::GitStage, ship_git::rest_handlers::stage_handler)
//!     // ... other git routes ...
//!     .with_state(rt.git_arc());
//! router.merge(git_router)
//! ```
//!
//! The SSE change-stream handler (`git_api::sse_handler`) stays in shipd
//! because it depends on shipd's `ViewSse`/`ViewStream` transport types.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;

use crate::service::{
    BranchCreateArgs, BranchDeleteArgs, CheckoutArgs, CommitArgs, GitService, PushArgs,
};
use crate::state::GitError;

// ── response type alias ───────────────────────────────────────────────────────

pub type Resp = (StatusCode, Json<serde_json::Value>);

// ── error helpers ─────────────────────────────────────────────────────────────

pub fn err(status: StatusCode, code: &str, message: impl Into<String>) -> Resp {
    (
        status,
        Json(serde_json::json!({
            "ok": false, "error": code, "message": message.into(),
        })),
    )
}

pub fn map_err(e: GitError) -> Resp {
    match e {
        GitError::RepoUnavailable(m) => err(StatusCode::NOT_FOUND, "no_repo", m),
        GitError::BadRef(m) => err(StatusCode::BAD_REQUEST, "bad_ref", m),
        GitError::NothingStaged => err(
            StatusCode::BAD_REQUEST,
            "nothing_staged",
            "no staged changes — stage files before committing",
        ),
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

// ── query shapes ──────────────────────────────────────────────────────────────

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

// ── body shapes ───────────────────────────────────────────────────────────────

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

// ── write handlers ────────────────────────────────────────────────────────────

/// POST /api/v1/git/stage
pub async fn stage_handler(
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<PathsBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let n = git.stage(body.workspace_id.as_deref(), &body.paths).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "staged": n })))
}

/// POST /api/v1/git/unstage
pub async fn unstage_handler(
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<PathsBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let n = git.unstage(body.workspace_id.as_deref(), &body.paths).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "unstaged": n })))
}

/// POST /api/v1/git/commit
pub async fn commit_handler(
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<CommitBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let res = git
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
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<PushBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let force = body.force;
    let remote = git
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
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<RemoteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let remote =
        git.fetch(body.workspace_id.as_deref(), body.remote.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "remote": remote })))
}

/// POST /api/v1/git/pull — ff-only
pub async fn pull_handler(
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<RemoteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let sha =
        git.pull(body.workspace_id.as_deref(), body.remote.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "head": sha })))
}

/// POST /api/v1/git/checkout
pub async fn checkout_handler(
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<CheckoutBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let refname = body.refname.clone();
    let resolved = git
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
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<BranchCreateBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let name = git
        .branch_create(BranchCreateArgs {
            workspace_id: body.workspace_id,
            name: body.name,
            checkout: body.checkout,
        })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "name": name })))
}

/// DELETE /api/v1/git/branch — delete a local branch.
pub async fn branch_delete_handler(
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<BranchDeleteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let name = git
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
    State(git): State<Arc<dyn GitService>>,
    Json(body): Json<InitBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    git.init(body.workspace_id.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
