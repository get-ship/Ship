//! Git view — minijinja-rendered shell + Datastar SSE content patches.
//!
//! Two endpoints:
//!
//! - `GET /api/runtime/view/git` — full-page HTML shell. Delivered once on
//!   load; contains the two-pane layout (rail + main) and bootstraps an
//!   SSE subscription on the main pane.
//! - `GET /api/runtime/view/git/sse` — Datastar SSE stream that re-renders
//!   `#git-main` on a polling cadence. Git state is polled rather than
//!   event-driven — there is no kernel topic for FS-level VCS changes
//!   today.
//!
//! All rendering goes through `crate::views::templates`. Git data loading
//! and porcelain parsing live in `crate::views_git_data`.

use axum::extract::{Query, State};
use axum::response::Html;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;
use crate::views_git_data::{
    CommitDetail, CommitSummary, StatusFile, load_branch, load_commit_detail, load_log,
    load_status, resolve_root,
};

/// Query string for both shell + SSE endpoints.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct GitParams {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub tab: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub token: String,
}

impl GitParams {
    pub fn active_tab(&self) -> String {
        self.tab
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "log".to_string())
    }
}

#[derive(Serialize)]
pub(crate) struct ErrorView {
    pub title: String,
    pub body: String,
}

#[derive(Serialize)]
pub(crate) struct GitCtx {
    pub active_tab: String,
    pub workspace_id: Option<String>,
    pub branch: Option<String>,
    pub status_files: Vec<StatusFile>,
    pub commits: Vec<CommitSummary>,
    pub selected_commit: Option<CommitDetail>,
    pub error: Option<ErrorView>,
    pub daemon_origin: String,
    pub token: String,
}

pub(crate) fn build_ctx(params: &GitParams, ship_dir: &Path, daemon_origin: String) -> GitCtx {
    let root = match resolve_root(ship_dir, params.workspace_id.as_deref()) {
        Ok(r) => r,
        Err(e) => {
            return GitCtx {
                active_tab: params.active_tab(),
                workspace_id: params.workspace_id.clone(),
                branch: None,
                status_files: vec![],
                commits: vec![],
                selected_commit: None,
                error: Some(ErrorView {
                    title: "Git unavailable".into(),
                    body: e,
                }),
                daemon_origin,
                token: params.token.clone(),
            };
        }
    };

    let branch = load_branch(&root);
    let status_files = load_status(&root);
    let commits = load_log(&root);
    let selected_commit = params
        .commit
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(|sha| load_commit_detail(&root, sha));

    GitCtx {
        active_tab: params.active_tab(),
        workspace_id: params.workspace_id.clone(),
        branch,
        status_files,
        commits,
        selected_commit,
        error: None,
        daemon_origin,
        token: params.token.clone(),
    }
}

/// GET /api/runtime/view/git — full shell, minijinja-rendered.
pub async fn git_shell(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<GitParams>,
) -> Html<String> {
    let ctx = build_ctx(&params, state.ship_dir(), state.daemon_origin());
    let html = templates::render("git/shell.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<pre>shell render error: {e}</pre>"));
    Html(html)
}

/// GET /api/runtime/view/git/sse — periodic patches for `#git-main`.
pub async fn git_sse(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<GitParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();
    let origin = state.daemon_origin();

    tokio::spawn(async move {
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        let html = render_content(&params, &ship_dir, origin.clone());
        if vs.patch("#git-main", html).await.is_err() {
            return;
        }

        // Poll the working tree every 5s. No kernel topic exists for VCS
        // writes today — a timed re-render is the simplest correct option.
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if vs.is_closed() {
                break;
            }
            let html = render_content(&params, &ship_dir, origin.clone());
            if vs.patch("#git-main", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

fn render_content(params: &GitParams, ship_dir: &Path, daemon_origin: String) -> String {
    let ctx = build_ctx(params, ship_dir, daemon_origin);
    templates::render("git/content.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<div id=\"git-main\"><pre>{e}</pre></div>"))
}

#[cfg(test)]
#[path = "views_git_tests.rs"]
mod tests;
