//! Agents view — minijinja-rendered shells for index + detail pages.
//!
//! Transitional: the four subroute handlers (`agent_detail`,
//! `agent_settings`, `agent_wire`, `agents_create_view`) are being
//! migrated onto the file-routed framework (`/api/v/agents/*`). This
//! module now delegates to `services::agents::{wire,create,settings_view}`
//! for shaping; once the file-routed templates land the bespoke
//! handlers here will be deleted in a follow-up commit.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Html;
use serde::Serialize;

use crate::rest_api::ApiState;
use crate::services::agents::create::{
    AgentsCreateCtx, build_create_ctx, builtin_permission_presets, builtin_providers,
    builtin_templates,
};
use crate::services::agents::settings_view::{AgentSettingsCtx, build as build_settings, build_tiers};
use crate::services::agents::wire::{AgentWireCtx, build as build_wire};
use crate::views::resolve_view_dir;
use crate::views::state::{AgentSummary, list_agents};
use crate::views::templates;

// Re-exports for existing callers / tests that reach through this module.
pub use crate::services::agents::create::{AgentTemplate, PermissionPreset};
pub use crate::services::agents::settings_view::TierRow;
pub use crate::services::agents::wire::{WireEdge, WireNode, compute_wire_layout};

/// Try rendering a template from `.ship/views/<view>/<file>`.
/// Returns `None` if the view directory or file does not exist, or if
/// rendering fails (in which case the caller falls back to embedded templates).
fn try_render_view<C: Serialize>(view: &str, file: &str, ctx: &C) -> Option<String> {
    let (view_dir, _source) = resolve_view_dir(view)?;
    let source = std::fs::read_to_string(view_dir.join(file)).ok()?;
    let template_name = format!("{view}/{file}");
    let html = templates::render_view_source(&template_name, &source, ctx).ok()?;
    let packs = crate::views::read_view_packs(&view_dir);
    Some(crate::views::inject_packs(html, &packs, false))
}

/// Detail-view context used by the legacy settings-panel template. Kept
/// here as a type alias so the tests file continues to compile.
pub type AgentDetailCtx = AgentSettingsCtx;

#[derive(Serialize)]
pub struct AgentsIndexCtx {
    pub agents: Vec<AgentSummary>,
    pub total: usize,
    pub templates: Vec<AgentTemplate>,
    pub permission_presets: Vec<PermissionPreset>,
    pub available_providers: Vec<String>,
}

/// GET /api/runtime/view/agents — index shell.
pub async fn agents_shell(
    State(state): State<ApiState>,
) -> Result<Html<String>, StatusCode> {
    let agents = list_agents(state.ship_dir());
    let total = agents.len();
    let ctx = AgentsIndexCtx {
        agents,
        total,
        templates: builtin_templates(),
        permission_presets: builtin_permission_presets(),
        available_providers: builtin_providers(),
    };

    if let Some(html) = try_render_view("agents", "index.html.j2", &ctx) {
        return Ok(Html(html));
    }
    templates::render("agents/shell.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "agents shell render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/agents/detail/{id} — canonical detail (wire layout).
pub async fn agent_detail(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_wire(&id, state.ship_dir()).ok_or(StatusCode::NOT_FOUND)?;
    try_render_view("agents", "wire.html.j2", &ctx).map(Html).ok_or(StatusCode::NOT_FOUND)
}

/// GET /api/runtime/view/agents/detail/{id}/settings — panels layout.
pub async fn agent_settings(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_settings(&id, state.ship_dir()).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(html) = try_render_view("agents", "detail.html.j2", &ctx) {
        return Ok(Html(html));
    }
    templates::render("agents/detail.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "agent settings render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/agents/{id}/wire — graph/canvas detail view.
pub async fn agent_wire(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let ctx = build_wire(&id, state.ship_dir()).ok_or(StatusCode::NOT_FOUND)?;
    try_render_view("agents", "wire.html.j2", &ctx).map(Html).ok_or(StatusCode::NOT_FOUND)
}

/// GET /api/runtime/view/agents/create — create-agent wizard shell.
pub async fn agents_create_view(
    State(_state): State<ApiState>,
) -> Result<Html<String>, StatusCode> {
    let ctx: AgentsCreateCtx = build_create_ctx();
    try_render_view("agents", "create.html.j2", &ctx).map(Html).ok_or(StatusCode::NOT_FOUND)
}

#[cfg(test)]
#[path = "views_agents_tests.rs"]
mod tests;
