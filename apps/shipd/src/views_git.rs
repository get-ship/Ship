//! Git view — minijinja-rendered shell + Datastar SSE content patches.
//!
//! Single-column, information-dense layout matching the React `/studio/git`
//! page. Two endpoints:
//!
//! - `GET /api/runtime/view/git` — renders the full shell as a fragment.
//!   Dispatches on `commit`: absent → status + log sections; present →
//!   full-pane diff viewer (pass `commit=WORKING` for the working tree
//!   diff against HEAD).
//! - `GET /api/runtime/view/git/sse` — Datastar SSE stream that re-renders
//!   the shell on a polling cadence (there is no kernel topic for
//!   FS-level VCS changes today) and morphs `#git-root` in place.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::views::resolve_view_dir;
use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;
use crate::services::git::read::{
    load_ahead_behind, load_branch, load_branches, load_commit_diff, load_log,
    load_status_buckets, load_working_diff, DEFAULT_LOG_LIMIT,
};
use crate::services::git::state::{resolve_root, BranchSummary, CommitSummary, StatusFile};
use crate::views_git_annotations::{AnnotationsByLine, load_file_annotations};
use crate::views_git_diff::{ParsedDiff, parse_unified_diff};

/// Query string for both shell + SSE endpoints.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct GitParams {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ErrorView {
    pub title: String,
    pub body: String,
}


#[derive(Serialize)]
pub(crate) struct DiffView {
    pub sha: String,
    pub short_sha: String,
    pub is_working: bool,
    pub diff_text: String,
    pub parsed: ParsedDiff,
}

#[derive(Serialize)]
pub(crate) struct GitCtx {
    pub workspace_id: Option<String>,
    pub branch: Option<String>,
    pub head_short: Option<String>,
    pub clean: bool,
    pub total_changed: usize,
    pub staged: Vec<StatusFile>,
    pub modified: Vec<StatusFile>,
    pub untracked: Vec<StatusFile>,
    pub commits: Vec<CommitSummary>,
    pub branches: Vec<BranchSummary>,
    /// `(ahead, behind)` vs the upstream tracking branch, if one is configured.
    pub ahead: usize,
    pub behind: usize,
    pub selected_commit: Option<DiffView>,
    /// File path the user has selected (for per-file diff focus).
    pub active_file: Option<String>,
    /// Annotations for the active diff file, keyed by `"{sha}:{line}"`.
    pub annotations: AnnotationsByLine,
    pub error: Option<ErrorView>,
}

fn resolve_selected(params: &GitParams, root: &Path) -> Option<DiffView> {
    let raw = params.commit.as_deref()?.trim();
    if raw.is_empty() {
        return None;
    }
    if raw.eq_ignore_ascii_case("WORKING") {
        let diff_text = load_working_diff(root);
        let parsed = parse_unified_diff(&diff_text);
        return Some(DiffView {
            sha: "WORKING".into(),
            short_sha: "working".into(),
            is_working: true,
            diff_text,
            parsed,
        });
    }
    let diff_text = load_commit_diff(root, raw);
    if diff_text.is_empty() {
        return None;
    }
    let parsed = parse_unified_diff(&diff_text);
    let short_sha: String = raw.chars().take(7).collect();
    Some(DiffView {
        sha: raw.to_string(),
        short_sha,
        is_working: false,
        diff_text,
        parsed,
    })
}

/// Sync base — used by the git view handler and by cross-view context
/// injection (`views/context.rs`). Does not load annotations.
pub(crate) fn build_base_ctx(params: &GitParams, ship_dir: &Path) -> GitCtx {
    let root = match resolve_root(ship_dir, params.workspace_id.as_deref()) {
        Ok(r) => r,
        Err(e) => {
            return GitCtx {
                workspace_id: params.workspace_id.clone(),
                branch: None,
                head_short: None,
                clean: true,
                total_changed: 0,
                staged: vec![],
                modified: vec![],
                untracked: vec![],
                commits: vec![],
                branches: vec![],
                ahead: 0,
                behind: 0,
                selected_commit: None,
                active_file: None,
                annotations: Default::default(),

                error: Some(ErrorView {
                    title: "Git unavailable".into(),
                    body: e,
                }),
            };
        }
    };

    let branch = load_branch(&root);
    let head_short = commits_head_short(&root);
    let buckets = load_status_buckets(&root);
    let commits = load_log(&root, None, DEFAULT_LOG_LIMIT);
    let branches = load_branches(&root);
    let (ahead, behind) = load_ahead_behind(&root).unwrap_or((0, 0));
    let selected_commit = resolve_selected(params, &root);
    let active_file = params
        .file
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    GitCtx {
        workspace_id: params.workspace_id.clone(),
        branch,
        head_short,
        clean: buckets.clean,
        total_changed: buckets.total,
        staged: buckets.staged,
        modified: buckets.modified,
        untracked: buckets.untracked,
        commits,
        branches,
        ahead,
        behind,
        selected_commit,
        active_file,
        annotations: Default::default(),
        error: None,
    }
}

fn commits_head_short(root: &Path) -> Option<String> {
    let repo = git2::Repository::open(root).ok()?;
    let head = repo.head().ok()?;
    let oid = head.target()?;
    Some(oid.to_string().chars().take(7).collect())
}

/// Async full context — adds annotations for the active diff file.
pub(crate) async fn build_ctx(
    params: &GitParams,
    ship_dir: &Path,
    state: &crate::rest_api::ApiState,
) -> GitCtx {
    let mut ctx = build_base_ctx(params, ship_dir);

    if let Some(file) = params.file.as_deref().filter(|s| !s.is_empty()) {
        if let Ok(mut guard) = state.data_service().await {
            if let Some(svc) = guard.as_mut() {
                ctx.annotations =
                    load_file_annotations(svc, file, params.workspace_id.as_deref());
            }
        }
    }

    ctx
}

async fn render_shell(
    params: &GitParams,
    ship_dir: &Path,
    state: &crate::rest_api::ApiState,
) -> Option<String> {
    let ctx = build_ctx(params, ship_dir, state).await;
    match templates::render("git/shell.html.j2", &ctx) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(error = %e, "git shell render failed");
            None
        }
    }
}

/// Try rendering a template from `.ship/views/git/<file>`.
/// Returns `None` if the view directory or file does not exist, or if
/// rendering fails (in which case the caller falls back to embedded templates).
fn try_render_view(file: &str, ctx: &GitCtx) -> Option<String> {
    let (view_dir, _source) = resolve_view_dir("git")?;
    let source = std::fs::read_to_string(view_dir.join(file)).ok()?;
    let template_name = format!("git/{file}");
    let html = templates::render_view_source(&template_name, &source, ctx).ok()?;
    let packs = crate::views::read_view_packs(&view_dir);
    Some(crate::views::inject_packs(html, &packs, false))
}

/// GET /api/runtime/view/git — full shell, minijinja-rendered.
pub async fn git_shell(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<GitParams>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_ctx(&params, state.ship_dir(), &state).await;

    // Try .ship/views/git/index.html.j2 first (new canonical location).
    if let Some(html) = try_render_view("index.html.j2", &ctx) {
        return Ok(Html(html));
    }

    // Fallback to embedded template.
    render_shell(&params, state.ship_dir(), &state)
        .await
        .map(Html)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// GET /api/runtime/view/git/sse — periodic patches for `#git-root`.
pub async fn git_sse(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<GitParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();

    tokio::spawn(async move {
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        if let Some(html) = render_shell(&params, &ship_dir, &state).await {
            if vs.patch("#git-root", html).await.is_err() {
                return;
            }
        }

        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if vs.is_closed() {
                break;
            }
            let Some(html) = render_shell(&params, &ship_dir, &state).await else {
                continue;
            };
            if vs.patch("#git-root", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

#[cfg(test)]
#[path = "views_git_tests.rs"]
mod tests;
