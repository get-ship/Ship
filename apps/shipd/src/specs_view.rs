//! Specs view — minijinja-rendered shell + Datastar SSE content patches.
//!
//! Two endpoints:
//!
//! - `GET /api/runtime/view/specs` — full-page HTML shell. Two-pane layout
//!   (rail + main). The rail groups specs by status (Draft/Ready/Launched/
//!   Completed), filterable via `?q=`. The main pane renders empty-state
//!   or the selected spec's detail editor server-side.
//! - `GET /api/runtime/view/specs/sse` — Datastar SSE stream that patches
//!   `#specs-rail-list` on any `spec.*`/`job.*` kernel event.
//!
//! All rendering goes through `crate::views::templates`. Mutation routes
//! live in `specs_api.rs` and are untouched by this module.

use axum::extract::{Query, State};
use axum::response::Html;
use chrono::{DateTime, Utc};
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
    pub selected: Option<String>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub token: String,
    // Legacy; still accepted so existing bookmarks don't 404. Unused.
    #[serde(default)]
    pub tab: Option<String>,
}

impl SpecsParams {
    pub fn selected_id(&self) -> Option<String> {
        self.selected.clone().filter(|s| !s.is_empty())
    }
    pub fn query(&self) -> String {
        self.q.clone().unwrap_or_default()
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
    when: String,
}

#[derive(Default, Serialize)]
struct Counts {
    draft: usize,
    ready: usize,
    launched: usize,
    completed: usize,
}

#[derive(Serialize)]
struct AcceptanceItem {
    text: String,
    done: bool,
}

/// Grouped sections rendered in the left rail, in the mockup's order.
#[derive(Serialize)]
struct RailGroup {
    key: &'static str,
    label: &'static str,
    count: usize,
    items: Vec<SpecSummary>,
}

/// Full editor context for the main pane.
#[derive(Serialize)]
struct SpecDetail {
    id: String,
    title: String,
    body: String,
    status: String,
    status_label: String,
    pipeline: Option<String>,
    pipeline_label: String,
    agent: String,
    vars_display: String,
    branch: String,
    tone: &'static str,
    // Rendered from body markdown.
    goal_md: String,
    scope_md: String,
    linked_md: String,
    acceptance: Vec<AcceptanceItem>,
    // Extra sections not in Goal/Acceptance/Scope/Linked — rendered as-is.
    other_md: String,
    // Launch-bar context.
    launch_label: String,
    launch_hint: String,
    // JS-escaped signals for edit-mode inputs.
    title_js: String,
    body_js: String,
    pipeline_js: String,
}

#[derive(Serialize)]
struct SpecsCtx {
    selected_id: Option<String>,
    q: String,
    counts: Counts,
    groups: Vec<RailGroup>,
    total: usize,
    selected: Option<SpecDetail>,
    pipelines: Vec<String>,
    daemon_origin: String,
    token: String,
}

fn tone_for(status: &str) -> &'static str {
    match status {
        "completed" => "success",
        "ready" => "accent",
        "launched" => "warning",
        "failed" => "destructive",
        _ => "muted",
    }
}

fn human_since(ts: DateTime<Utc>) -> String {
    let now = Utc::now();
    let delta = now.signed_duration_since(ts);
    let secs = delta.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3_600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3_600)
    } else if secs < 86_400 * 2 {
        "yesterday".to_string()
    } else {
        format!("{}d ago", secs / 86_400)
    }
}

fn verb_for(status: &str) -> &'static str {
    match status {
        "ready" => "ready",
        "launched" => "launched",
        "completed" => "merged",
        _ => "updated",
    }
}

fn to_summary(s: &Spec) -> SpecSummary {
    let verb = verb_for(&s.status);
    let when = format!("{verb} · {}", human_since(s.updated_at));
    SpecSummary {
        id: s.id.clone(),
        title: if s.title.is_empty() {
            s.id.clone()
        } else {
            s.title.clone()
        },
        status: s.status.clone(),
        pipeline: s.pipeline.clone(),
        tone: tone_for(&s.status),
        when,
    }
}

fn js_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "")
}

/// Split a markdown body into named sections keyed by H2 heading.
/// Returns (sections, prelude). Sections keys are lowercased (`goal`,
/// `acceptance`, `scope`, `linked`). Anything above the first H2 is
/// returned as `prelude`.
fn split_sections(body: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut sections: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut prelude = String::new();
    let mut current_key: Option<String> = None;
    let mut buf = String::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(k) = current_key.take() {
                sections.insert(k, std::mem::take(&mut buf).trim().to_string());
            } else if !buf.is_empty() {
                prelude = std::mem::take(&mut buf).trim().to_string();
            }
            current_key = Some(rest.trim().to_lowercase());
        } else {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if let Some(k) = current_key.take() {
        sections.insert(k, buf.trim().to_string());
    } else if !buf.is_empty() && prelude.is_empty() {
        prelude = buf.trim().to_string();
    }
    (sections, prelude)
}

/// Parse an `## Acceptance` section body into checklist items. Non-checklist
/// lines are ignored. Standard GFM syntax: `- [x] done`, `- [ ] not done`.
fn parse_acceptance(section: &str) -> Vec<AcceptanceItem> {
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix("- ")?;
            let (marker, text) = if let Some(t) = rest.strip_prefix("[x] ") {
                (true, t)
            } else if let Some(t) = rest.strip_prefix("[X] ") {
                (true, t)
            } else if let Some(t) = rest.strip_prefix("[ ] ") {
                (false, t)
            } else {
                return None;
            };
            Some(AcceptanceItem {
                text: text.trim().to_string(),
                done: marker,
            })
        })
        .collect()
}

fn pipeline_step_count(name: &str) -> (usize, Option<String>) {
    use runtime::services::orchestrator::pipeline_loader;
    let payload = runtime::events::job::JobCreatedPayload {
        job_id: String::new(),
        slug: String::new(),
        agent: String::new(),
        branch: String::new(),
        spec_path: String::new(),
        plan_id: None,
        model: None,
        provider: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: Some(name.to_string()),
        vars: None,
        spec_content: None,
    };
    let resolved = pipeline_loader::ensure_pipeline(payload);
    let steps = match resolved.steps.as_ref() {
        Some(p) => &p.steps,
        None => return (0, None),
    };
    let first_agent = steps
        .iter()
        .find(|s| s.agent.is_some())
        .and_then(|s| s.agent.clone());
    (steps.len(), first_agent)
}

fn to_detail(s: &Spec) -> SpecDetail {
    let body = specs::load_body(&s.body_path).unwrap_or_default();
    let (sections, prelude) = split_sections(&body);

    let goal_md = sections.get("goal").cloned().unwrap_or(prelude);
    let acceptance_md = sections.get("acceptance").cloned().unwrap_or_default();
    let scope_md = sections.get("scope").cloned().unwrap_or_default();
    let linked_md = sections.get("linked").cloned().unwrap_or_default();
    let acceptance = parse_acceptance(&acceptance_md);

    // Preserve any extra sections as one appended blob.
    let known = ["goal", "acceptance", "scope", "linked"];
    let mut other = String::new();
    for (k, v) in &sections {
        if !known.contains(&k.as_str()) {
            other.push_str("## ");
            // Title-case the first letter of the section key.
            let mut chars = k.chars();
            if let Some(c) = chars.next() {
                other.extend(c.to_uppercase());
                other.push_str(chars.as_str());
            }
            other.push('\n');
            other.push_str(v);
            other.push_str("\n\n");
        }
    }

    let pipeline_label = s.pipeline.clone().unwrap_or_else(|| "—".to_string());
    let (_steps, first_agent) = match s.pipeline.as_deref() {
        Some(name) => pipeline_step_count(name),
        None => (0, None),
    };
    let agent = first_agent.clone().unwrap_or_else(|| "—".to_string());

    let vars_display = if s.vars.is_empty() {
        "—".to_string()
    } else {
        s.vars
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    };

    let launch_label = match (&s.pipeline, &first_agent) {
        (Some(p), Some(a)) => format!("Launch {p} → {a}"),
        (Some(p), None) => format!("Launch {p}"),
        _ => "Launch".to_string(),
    };
    let launch_hint = match &first_agent {
        Some(agent_name) => format!(
            "Will create worktree job/{}-… and dispatch {} with this spec.",
            s.pipeline.as_deref().unwrap_or("spec"),
            agent_name
        ),
        None => "Select a pipeline above to enable launch.".to_string(),
    };

    SpecDetail {
        id: s.id.clone(),
        title: s.title.clone(),
        body: body.clone(),
        status: s.status.clone(),
        status_label: s.status[..1].to_uppercase() + &s.status[1..],
        pipeline: s.pipeline.clone(),
        pipeline_label,
        agent,
        vars_display,
        branch: "auto".into(),
        tone: tone_for(&s.status),
        goal_md,
        scope_md,
        linked_md,
        acceptance,
        other_md: other.trim().to_string(),
        launch_label,
        launch_hint,
        title_js: js_escape(&s.title),
        body_js: js_escape(&body),
        pipeline_js: js_escape(s.pipeline.as_deref().unwrap_or("")),
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

fn matches_query(s: &Spec, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let needle = q.to_lowercase();
    s.title.to_lowercase().contains(&needle) || s.id.to_lowercase().contains(&needle)
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

fn groups_of(all: &[Spec], q: &str) -> Vec<RailGroup> {
    let order: [(&'static str, &'static str); 4] = [
        ("draft", "Draft"),
        ("ready", "Ready"),
        ("launched", "Launched"),
        ("completed", "Completed"),
    ];
    order
        .iter()
        .map(|(key, label)| {
            let items: Vec<SpecSummary> = all
                .iter()
                .filter(|s| s.status == *key && matches_query(s, q))
                .map(to_summary)
                .collect();
            let count = all.iter().filter(|s| s.status == *key).count();
            RailGroup {
                key,
                label,
                count,
                items,
            }
        })
        .collect()
}

fn build_ctx(params: &SpecsParams, daemon_origin: String) -> SpecsCtx {
    let all = specs::list_specs().unwrap_or_default();
    let selected_id = params.selected_id();
    let q = params.query();

    let counts = counts_of(&all);
    let groups = groups_of(&all, &q);
    let total = all.len();

    let selected = selected_id
        .as_deref()
        .and_then(|id| all.iter().find(|s| s.id == id).map(to_detail));

    SpecsCtx {
        selected_id,
        q,
        counts,
        groups,
        total,
        selected,
        pipelines: list_pipelines(),
        daemon_origin,
        token: params.token.clone(),
    }
}

/// GET /api/runtime/view/specs — full shell, minijinja-rendered.
pub async fn specs_shell(
    State(state): State<crate::rest_api::ApiState>,
    Query(params): Query<SpecsParams>,
) -> Html<String> {
    let ctx = build_ctx(&params, state.daemon_origin());
    let html = templates::render("specs/shell.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<pre>shell render error: {e}</pre>"));
    Html(html)
}

/// GET /api/runtime/view/specs/sse — Datastar patches for the rail list on
/// any `spec.*`/`job.*` kernel event.
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
    let origin = state.daemon_origin();

    tokio::spawn(async move {
        let _guard = guard;
        let _hb = vs.heartbeat_every(Duration::from_secs(15));

        // Initial render.
        let html = render_rail(&params, origin.clone());
        if vs.patch("#specs-rail-list", html).await.is_err() {
            return;
        }

        let Some(mut mb) = mailbox else { return };
        while let Some(_ev) = mb.recv().await {
            if vs.is_closed() {
                break;
            }
            let html = render_rail(&params, origin.clone());
            if vs.patch("#specs-rail-list", html).await.is_err() {
                break;
            }
        }
    });

    sse
}

fn render_rail(params: &SpecsParams, daemon_origin: String) -> String {
    let ctx = build_ctx(params, daemon_origin);
    templates::render("specs/partials/rail.html.j2", &ctx)
        .unwrap_or_else(|e| format!("<div id=\"specs-rail-list-inner\"><pre>{e}</pre></div>"))
}

#[cfg(test)]
#[path = "specs_view_tests.rs"]
mod tests;
