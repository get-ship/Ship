//! Skills view — minijinja-rendered IDE-style split: left rail + right pane.
//!
//! Routes:
//!
//! - `GET /api/runtime/view/skills` — full shell with rail (search, filter
//!   chips, tree) on the left and an empty state on the right.
//! - `GET /api/runtime/view/skills/rail` — partial re-render of the rail
//!   (used by the datastar `active_search` pattern driving the search box).
//! - `GET /api/runtime/view/skills/detail/{id}` — detail pane populated
//!   with the selected skill's SKILL.md body (rendered raw in a `<pre>`
//!   for now — server-side markdown rendering needs a new dep) and a
//!   metadata sidebar with Variables / Runs / Used by tabs.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::rest_api::ApiState;
use crate::views::resolve_view_dir;
use crate::views::state::{SkillDetail, SkillKind, SkillSummary, list_skills, load_skill};
use crate::views::templates;

/// Try rendering a template from `.ship/views/<view>/<file>`.
/// Returns `None` if the view directory or file does not exist, or if
/// rendering fails (in which case the caller falls back to embedded templates).
fn try_render_view<C: Serialize>(view: &str, file: &str, ctx: &C) -> Option<String> {
    let (view_dir, _source) = resolve_view_dir(view)?;
    let source = std::fs::read_to_string(view_dir.join(file)).ok()?;
    let template_name = format!("{view}/{file}");
    templates::render_view_source(&template_name, &source, ctx).ok()
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SkillsParams {
    #[serde(default)]
    pub q: Option<String>,
    /// "all" (default), "smart", "docs".
    #[serde(default)]
    pub kind: Option<String>,
    /// Optional tag filter (exact match).
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Serialize)]
pub struct SkillGroup {
    pub source: String,
    pub skills: Vec<SkillSummary>,
}

#[derive(Serialize)]
pub struct KindCount {
    pub id: &'static str,
    pub label: &'static str,
    pub count: usize,
    pub selected: bool,
}

#[derive(Serialize)]
pub struct RailCtx {
    pub q: String,
    pub active_kind: String,
    pub active_tag: String,
    pub groups: Vec<SkillGroup>,
    pub kind_counts: Vec<KindCount>,
    pub tags: Vec<String>,
    pub total: usize,
    pub filtered: usize,
}

#[derive(Serialize)]
pub struct SkillsShellCtx {
    pub rail: RailCtx,
}

#[derive(Serialize)]
pub struct SkillDetailCtx {
    pub skill: SkillDetail,
    pub rail: RailCtx,
}

fn active_kind(params: &SkillsParams) -> String {
    params
        .kind
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "all".to_string())
}

fn kind_matches(kind: &str, s: &SkillSummary) -> bool {
    match kind {
        "smart" => matches!(s.kind, SkillKind::Smart),
        "docs" => matches!(s.kind, SkillKind::Docs),
        _ => true,
    }
}

fn build_rail(ship_dir: &std::path::Path, params: &SkillsParams) -> RailCtx {
    let all = list_skills(ship_dir);
    let total = all.len();
    let smart = all.iter().filter(|s| matches!(s.kind, SkillKind::Smart)).count();
    let docs = all.iter().filter(|s| matches!(s.kind, SkillKind::Docs)).count();
    let q = params.q.clone().unwrap_or_default();
    let needle = q.to_lowercase();
    let active_tag = params.tag.clone().unwrap_or_default();
    let kind = active_kind(params);

    let filtered: Vec<SkillSummary> = all
        .iter()
        .filter(|s| kind_matches(&kind, s))
        .filter(|s| {
            if needle.is_empty() {
                return true;
            }
            let hay = format!("{} {}", s.name, s.description).to_lowercase();
            hay.contains(&needle)
        })
        .filter(|s| active_tag.is_empty() || s.tags.iter().any(|t| t == &active_tag))
        .cloned()
        .collect();

    let mut grouped: BTreeMap<String, Vec<SkillSummary>> = BTreeMap::new();
    for s in &filtered {
        grouped.entry(s.source.clone()).or_default().push(s.clone());
    }
    let groups: Vec<SkillGroup> = grouped
        .into_iter()
        .map(|(source, skills)| SkillGroup { source, skills })
        .collect();

    let mut tag_set = std::collections::BTreeSet::new();
    for s in &all {
        for t in &s.tags {
            tag_set.insert(t.clone());
        }
    }

    let kind_counts = vec![
        KindCount {
            id: "all",
            label: "All",
            count: total,
            selected: kind == "all",
        },
        KindCount {
            id: "smart",
            label: "Smart",
            count: smart,
            selected: kind == "smart",
        },
        KindCount {
            id: "docs",
            label: "Docs",
            count: docs,
            selected: kind == "docs",
        },
    ];

    RailCtx {
        q,
        active_kind: kind,
        active_tag,
        groups,
        kind_counts,
        tags: tag_set.into_iter().collect(),
        total,
        filtered: filtered.len(),
    }
}

/// GET /api/runtime/view/skills — full shell with empty right pane.
pub async fn skills_shell(
    State(state): State<ApiState>,
    Query(params): Query<SkillsParams>,
) -> Result<Html<String>, StatusCode> {
    let rail = build_rail(state.ship_dir(), &params);
    let ctx = SkillsShellCtx { rail };

    // Try .ship/views/skills/index.html.j2 first (new canonical location).
    if let Some(html) = try_render_view("skills", "index.html.j2", &ctx) {
        return Ok(Html(html));
    }

    // Fallback to embedded template.
    templates::render("skills/shell.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "skills shell render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/skills/rail — partial rail (datastar active_search).
pub async fn skills_rail(
    State(state): State<ApiState>,
    Query(params): Query<SkillsParams>,
) -> Result<Html<String>, StatusCode> {
    let rail = build_rail(state.ship_dir(), &params);
    let ctx = SkillsShellCtx { rail };
    templates::render("skills/partials/rail.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "skills rail render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/skills/detail/{id} — full shell with a populated
/// detail pane (middle) and metadata sidebar (right).
pub async fn skill_detail(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(params): Query<SkillsParams>,
) -> Result<Html<String>, StatusCode> {
    let Some(skill) = load_skill(state.ship_dir(), &id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let rail = build_rail(state.ship_dir(), &params);
    let ctx = SkillDetailCtx { skill, rail };
    templates::render("skills/shell.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "skill detail render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[cfg(test)]
#[path = "views_skills_tests.rs"]
mod tests;
