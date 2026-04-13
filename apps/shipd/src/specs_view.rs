//! Specs view — minijinja-rendered shell + Datastar SSE content patches.
//!
//! Two endpoints:
//!
//! - `GET /api/runtime/view/specs` — full-page HTML shell. Two-pane layout
//!   (rail + main). The main pane renders empty-state or the selected
//!   spec's detail editor server-side; the rail bootstraps an SSE
//!   subscription for live updates.
//! - `GET /api/runtime/view/specs/sse` — Datastar SSE stream that patches
//!   `#specs-rail-list` whenever the spec list changes (any `spec.*`
//!   kernel event; jobs update too so launched-status reflects live
//!   progress via `job.*`).
//!
//! All rendering goes through `crate::views::templates`. Mutation routes
//! live in `specs_api.rs` and are untouched by this module.

use axum::extract::{Query, State};
use axum::response::Html;
use runtime::db::specs::{self, Spec};
use runtime::events::ActorConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::views::stream::{ViewSse, ViewStream};
use crate::views::templates;

/// Query string for both shell + SSE endpoints.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct SpecsParams {
    #[serde(default)]
    pub tab: Option<String>,
    #[serde(default)]
    pub selected: Option<String>,
}

impl SpecsParams {
    pub fn active_tab(&self) -> String {
        self.tab
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "draft".to_string())
    }
    pub fn selected_id(&self) -> Option<String> {
        self.selected.clone().filter(|s| !s.is_empty())
    }
}

/// Row shown in the rail list.
#[derive(Serialize)]
struct SpecSummary {
    id: String,
    title: String,
    status: String,
    pipeline: Option<String>,
    tone: &'static str,
    updated_at: String,
}

#[derive(Default, Serialize)]
struct Counts {
    draft: usize,
    ready: usize,
    launched: usize,
    completed: usize,
}

/// Full editor context for the main pane.
#[derive(Serialize)]
struct SpecDetail {
    id: String,
    title: String,
    body: String,
    status: String,
    pipeline: Option<String>,
    tone: &'static str,
    // JS-escaped variants for embedding in `data-signals-*="'...'"` attrs.
    title_js: String,
    body_js: String,
    pipeline_js: String,
    criteria_text_js: String,
    scope_text_js: String,
    vars_text_js: String,
}

#[derive(Serialize)]
struct SpecsCtx {
    active_tab: String,
    selected_id: Option<String>,
    counts: Counts,
    specs: Vec<SpecSummary>,
    selected: Option<SpecDetail>,
    pipelines: Vec<String>,
}

fn tone_for(status: &str) -> &'static str {
    match status {
        "completed" | "ready" => "success",
        "launched" => "warning",
        "failed" => "destructive",
        _ => "muted",
    }
}

fn to_summary(s: &Spec) -> SpecSummary {
    SpecSummary {
        id: s.id.clone(),
        title: if s.title.is_empty() {
            "(untitled)".to_string()
        } else {
            s.title.clone()
        },
        status: s.status.clone(),
        pipeline: s.pipeline.clone(),
        tone: tone_for(&s.status),
        updated_at: s.updated_at.to_rfc3339(),
    }
}

fn js_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn to_detail(s: &Spec) -> SpecDetail {
    let body = specs::load_body(&s.body_path).unwrap_or_default();
    let criteria_text = s.criteria.join("\n");
    let scope_text = s.scope.join("\n");
    let vars_text: String = s
        .vars
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    SpecDetail {
        id: s.id.clone(),
        title: s.title.clone(),
        body: body.clone(),
        status: s.status.clone(),
        pipeline: s.pipeline.clone(),
        tone: tone_for(&s.status),
        title_js: js_escape(&s.title),
        body_js: js_escape(&body),
        pipeline_js: js_escape(s.pipeline.as_deref().unwrap_or("")),
        criteria_text_js: js_escape(&criteria_text),
        scope_text_js: js_escape(&scope_text),
        vars_text_js: js_escape(&vars_text),
    }
}

/// List pipeline names from `.ship/pipelines/*.yaml`.
fn list_pipelines() -> Vec<String> {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let dir = project_dir.join("pipelines");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("yaml") {
                return None;
            }
            p.file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
        .collect();
    out.sort();
    out
}

fn counts_of(all: &[Spec]) -> Counts {
    let mut c = Counts::default();
    for s in all {
        match s.status.as_str() {
            "draft" => c.draft += 1,
            "ready" => c.ready += 1,
            "launched" => c.launched += 1,
            "completed" => c.completed += 1,
            _ => {}
        }
    }
    c
}

fn build_ctx(params: &SpecsParams) -> SpecsCtx {
    let all = specs::list_specs().unwrap_or_default();
    let active_tab = params.active_tab();
    let selected_id = params.selected_id();

    let counts = counts_of(&all);
    let specs: Vec<SpecSummary> = all
        .iter()
        .filter(|s| s.status == active_tab)
        .map(to_summary)
        .collect();

    let selected = selected_id
        .as_deref()
        .and_then(|id| all.iter().find(|s| s.id == id).map(to_detail));

    SpecsCtx {
        active_tab,
        selected_id,
        counts,
        specs,
        selected,
        pipelines: list_pipelines(),
    }
}

/// GET /api/runtime/view/specs — full shell, minijinja-rendered.
pub async fn specs_shell(
    State(_state): State<crate::rest_api::ApiState>,
    Query(params): Query<SpecsParams>,
) -> Html<String> {
    let ctx = build_ctx(&params);
    let html = templates::render("specs/shell.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<pre>shell render error: {e}</pre>"));
    Html(html)
}

/// GET /api/runtime/view/specs/sse — Datastar patches for the rail list
/// (and, when a spec is selected, the main pane) on any `spec.*`/`job.*`
/// kernel event.
pub async fn specs_view_sse(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<SpecsParams>,
) -> ViewSse {
    let (vs, sse) = ViewStream::new();

    let actor_id = format!(
        "specs-sse-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["spec.".to_string(), "job.".to_string()],
    };
    let mailbox = state
        .kernel()
        .lock()
        .await
        .spawn_actor(&actor_id, config)
        .ok()
        .map(|(_store, mb)| mb);
    let guard = crate::runtime_api::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        // Initial render.
        let html = render_rail(&params);
        if vs.patch("#specs-rail-list", html).await.is_err() {
            return;
        }

        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let html = render_rail(&params);
            if vs.patch("#specs-rail-list", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

fn render_rail(params: &SpecsParams) -> String {
    let ctx = build_ctx(params);
    templates::render("specs/partials/rail.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<div id=\"specs-rail-list-inner\"><pre>{e}</pre></div>"))
}

#[cfg(test)]
#[path = "specs_view_tests.rs"]
mod tests;
