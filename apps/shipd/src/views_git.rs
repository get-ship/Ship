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
use runtime::projections::job::{JobStatus, load_jobs_at};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;
use crate::views_git_data::{
    CommitSummary, StatusFile, load_branch, load_commit_diff, load_log, load_status_buckets,
    load_working_diff, resolve_root,
};
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
pub(crate) struct CompletedJob {
    pub slug: String,
    pub branch: String,
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
    pub clean: bool,
    pub total_changed: usize,
    pub staged: Vec<StatusFile>,
    pub modified: Vec<StatusFile>,
    pub untracked: Vec<StatusFile>,
    pub commits: Vec<CommitSummary>,
    pub selected_commit: Option<DiffView>,
    pub completed_jobs: Vec<CompletedJob>,
    pub error: Option<ErrorView>,
}

fn load_completed_jobs(ship_dir: &Path) -> Vec<CompletedJob> {
    let jobs = match load_jobs_at(ship_dir) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!(error = %e, "load_jobs_at failed for git view");
            std::collections::HashMap::new()
        }
    };
    let mut out: Vec<CompletedJob> = jobs
        .into_values()
        .filter(|j| matches!(j.status, JobStatus::Completed))
        .map(|j| CompletedJob {
            slug: j.slug.clone(),
            branch: j.branch.clone(),
        })
        .collect();
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    out
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

pub(crate) fn build_ctx(params: &GitParams, ship_dir: &Path) -> GitCtx {
    let completed_jobs = load_completed_jobs(ship_dir);
    let root = match resolve_root(ship_dir, params.workspace_id.as_deref()) {
        Ok(r) => r,
        Err(e) => {
            return GitCtx {
                workspace_id: params.workspace_id.clone(),
                branch: None,
                clean: true,
                total_changed: 0,
                staged: vec![],
                modified: vec![],
                untracked: vec![],
                commits: vec![],
                selected_commit: None,
                completed_jobs,
                error: Some(ErrorView {
                    title: "Git unavailable".into(),
                    body: e,
                }),
            };
        }
    };

    let branch = load_branch(&root);
    let buckets = load_status_buckets(&root);
    let commits = load_log(&root);
    let selected_commit = resolve_selected(params, &root);

    GitCtx {
        workspace_id: params.workspace_id.clone(),
        branch,
        clean: buckets.clean,
        total_changed: buckets.total,
        staged: buckets.staged,
        modified: buckets.modified,
        untracked: buckets.untracked,
        commits,
        selected_commit,
        completed_jobs,
        error: None,
    }
}

fn render_shell(params: &GitParams, ship_dir: &Path) -> Option<String> {
    let ctx = build_ctx(params, ship_dir);
    match templates::render("git/shell.html.j2", &ctx) {
        Ok(html) => Some(html),
        Err(e) => {
            tracing::error!(error = %e, "git shell render failed");
            None
        }
    }
}

/// GET /api/runtime/view/git — full shell, minijinja-rendered.
pub async fn git_shell(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<GitParams>,
) -> Result<Html<String>, StatusCode> {
    render_shell(&params, state.ship_dir())
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

        if let Some(html) = render_shell(&params, &ship_dir) {
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
            let Some(html) = render_shell(&params, &ship_dir) else { continue };
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
