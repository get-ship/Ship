//! GitService — HTTP surface at `/api/git/*`.
//!
//! Canonical shape per `.artifacts/daemon-services.md` §3.GitService:
//!
//! | GET    /api/git/status          | bucketed working tree         |
//! | GET    /api/git/diff            | ?commit=X&path=Y              |
//! | GET    /api/git/log             | ?branch=X&limit=N             |
//! | GET    /api/git/branches        |                               |
//! | POST   /api/git/stage           | {paths}                       |
//! | POST   /api/git/unstage         | {paths}                       |
//! | POST   /api/git/commit          | {message, author?}            |
//! | POST   /api/git/push            | {remote?, force?}             |
//! | POST   /api/git/fetch           |                               |
//! | POST   /api/git/pull            | ff-only; 409 on non-ff        |
//! | POST   /api/git/checkout        | {ref}                         |
//! | GET    /api/git/commit-draft    |                               |
//! | PUT    /api/git/commit-draft    | agent-written                 |
//! | DELETE /api/git/commit-draft    |                               |
//! | GET    /api/git/sse             | change stream                 |
//!
//! All endpoints operate on the active workspace / project root resolved
//! via `state::resolve_root`. Optional `?workspace_id=` query param lets
//! callers pin a specific worktree.

pub mod state;
pub mod read;
pub mod write;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

use crate::rest_api::ApiState;
use crate::views::stream::{ViewSse, ViewStream};
use state::GitError;

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
pub struct DraftBody {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub subject: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub by_agent: Option<String>,
    #[serde(default)]
    pub by_pipeline: Option<String>,
}

// ── helpers ──────────────────────────────────────────────────────────────

fn resolve(state: &ApiState, workspace_id: Option<&str>) -> Result<std::path::PathBuf, Resp> {
    state::resolve_root(state.ship_dir(), workspace_id)
        .map_err(|e| err(StatusCode::NOT_FOUND, "no_repo", e))
}

fn current_branch(root: &Path) -> Option<String> {
    read::load_branch(root)
}

// ── read handlers ────────────────────────────────────────────────────────

/// GET /api/git/status
pub async fn status_handler(
    State(state): State<ApiState>,
    Query(q): Query<WorkspaceQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, q.workspace_id.as_deref())?;
    let buckets = read::load_status_buckets(&root);
    let (ahead, behind) = read::load_ahead_behind(&root).unwrap_or((0, 0));
    Ok(Json(serde_json::json!({
        "ok": true,
        "branch": read::load_branch(&root),
        "head": read::load_head_short(&root),
        "ahead": ahead,
        "behind": behind,
        "clean": buckets.clean,
        "total": buckets.total,
        "staged": buckets.staged,
        "modified": buckets.modified,
        "untracked": buckets.untracked,
    })))
}

/// GET /api/git/diff?commit=X&path=Y   (commit=WORKING for working-tree)
pub async fn diff_handler(
    State(state): State<ApiState>,
    Query(q): Query<DiffQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, q.workspace_id.as_deref())?;
    let commit = q.commit.as_deref().unwrap_or("WORKING");
    let text = if commit.eq_ignore_ascii_case("WORKING") {
        read::load_working_diff(&root)
    } else {
        read::load_commit_diff(&root, commit)
    };
    Ok(Json(serde_json::json!({
        "ok": true,
        "commit": commit,
        "path": q.path,
        "diff": text,
    })))
}

/// GET /api/git/log?branch=X&limit=N
pub async fn log_handler(
    State(state): State<ApiState>,
    Query(q): Query<LogQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, q.workspace_id.as_deref())?;
    let limit = q.limit.unwrap_or(read::DEFAULT_LOG_LIMIT).clamp(1, 500);
    let commits = read::load_log(&root, q.branch.as_deref(), limit);
    Ok(Json(serde_json::json!({ "ok": true, "commits": commits })))
}

/// GET /api/git/branches
pub async fn branches_handler(
    State(state): State<ApiState>,
    Query(q): Query<WorkspaceQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, q.workspace_id.as_deref())?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "branches": read::load_branches(&root),
    })))
}

// ── write handlers ───────────────────────────────────────────────────────

/// POST /api/git/stage
pub async fn stage_handler(
    State(state): State<ApiState>,
    Json(body): Json<PathsBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let n = write::stage_paths(&root, &body.paths).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "staged": n })))
}

/// POST /api/git/unstage
pub async fn unstage_handler(
    State(state): State<ApiState>,
    Json(body): Json<PathsBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let n = write::unstage_paths(&root, &body.paths).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "unstaged": n })))
}

/// POST /api/git/commit
pub async fn commit_handler(
    State(state): State<ApiState>,
    Json(body): Json<CommitBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let res = write::commit_staged(&root, &body.message, body.author.as_deref())
        .map_err(map_err)?;
    // Clear any draft for this branch on success.
    if let Some(ref branch) = res.branch {
        let _ = runtime::db::git_drafts::delete_draft(branch);
    }
    Ok(Json(serde_json::json!({
        "ok": true,
        "sha": res.sha,
        "branch": res.branch,
    })))
}

/// POST /api/git/push
pub async fn push_handler(
    State(state): State<ApiState>,
    Json(body): Json<PushBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let remote = write::push_current_branch(&root, body.remote.as_deref(), body.force)
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "remote": remote, "force": body.force })))
}

/// POST /api/git/fetch
pub async fn fetch_handler(
    State(state): State<ApiState>,
    Json(body): Json<RemoteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let remote = write::fetch(&root, body.remote.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "remote": remote })))
}

/// POST /api/git/pull — ff-only
pub async fn pull_handler(
    State(state): State<ApiState>,
    Json(body): Json<RemoteBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let sha = write::pull_ff_only(&root, body.remote.as_deref()).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "head": sha })))
}

/// POST /api/git/checkout
pub async fn checkout_handler(
    State(state): State<ApiState>,
    Json(body): Json<CheckoutBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let resolved = write::checkout(&root, &body.refname).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "ref": body.refname, "resolved": resolved,
    })))
}

// ── commit draft CRUD ────────────────────────────────────────────────────

/// GET /api/git/commit-draft
pub async fn draft_get_handler(
    State(state): State<ApiState>,
    Query(q): Query<WorkspaceQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, q.workspace_id.as_deref())?;
    let Some(branch) = current_branch(&root) else {
        return Ok(Json(serde_json::json!({ "ok": true, "draft": null })));
    };
    let draft = runtime::db::git_drafts::get_draft(&branch)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true, "draft": draft })))
}

/// PUT /api/git/commit-draft
pub async fn draft_put_handler(
    State(state): State<ApiState>,
    Json(body): Json<DraftBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, body.workspace_id.as_deref())?;
    let branch = current_branch(&root)
        .ok_or_else(|| err(StatusCode::CONFLICT, "detached", "detached HEAD — no branch to key draft"))?;
    let d = runtime::db::git_drafts::CommitDraft {
        branch: branch.clone(),
        subject: body.subject,
        body: body.body,
        by_agent: body.by_agent,
        by_pipeline: body.by_pipeline,
        drafted_at: chrono::Utc::now(),
    };
    runtime::db::git_drafts::upsert_draft(&d)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true, "branch": branch })))
}

/// DELETE /api/git/commit-draft
pub async fn draft_delete_handler(
    State(state): State<ApiState>,
    Query(q): Query<WorkspaceQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let root = resolve(&state, q.workspace_id.as_deref())?;
    if let Some(branch) = current_branch(&root) {
        runtime::db::git_drafts::delete_draft(&branch)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db", e.to_string()))?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── change stream ────────────────────────────────────────────────────────

/// GET /api/git/sse — polls status at 5s cadence; patches `#git-service-data`
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
            let root = match state::resolve_root(&ship_dir, workspace_id.as_deref()) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let buckets = read::load_status_buckets(&root);
            let (ahead, behind) = read::load_ahead_behind(&root).unwrap_or((0, 0));
            let body = serde_json::json!({
                "branch": read::load_branch(&root),
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

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
