//! Agents view — minijinja-rendered shells for index + detail pages.
//!
//! Routes:
//!
//! - `GET /api/runtime/view/agents` — card grid of every agent discovered
//!   under `.ship/agents/*.{jsonc,json}`.
//! - `GET /api/runtime/view/agents/detail/{id}` — full config page for a
//!   single agent (Skills, MCP, Permissions, Rules, Providers).
//! - `POST /api/runtime/view/agents/new` — returns a structured
//!   `not_implemented` error. The "New agent" button in the mockup
//!   renders but does not invent an add flow.
//!
//! No SSE stream yet — agent configs change rarely and are file-backed;
//! a plain full-page reload is sufficient until we have a concrete
//! interaction that needs live updates.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::Json;
use serde::Serialize;

use crate::rest_api::ApiState;
use crate::views::resolve_view_dir;
use crate::views::state::{AgentDetail, AgentSummary, list_agents, load_agent};
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

#[derive(Serialize)]
pub struct AgentsIndexCtx {
    pub agents: Vec<AgentSummary>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct AgentDetailCtx {
    pub agent: AgentDetail,
    /// The canonical permission tiers, rendered as a radio-card triad.
    pub tiers: Vec<TierRow>,
}

#[derive(Serialize, Clone)]
pub struct TierRow {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub selected: bool,
}

fn build_tiers(selected: &str) -> Vec<TierRow> {
    [
        ("standard", "Standard", "Human confirms risky actions before they run."),
        ("autonomous", "Autonomous", "Agent acts end-to-end; safety rails still apply."),
        ("read-only", "Read Only", "Agent inspects and reports. No writes."),
    ]
    .iter()
    .map(|(id, label, desc)| TierRow {
        id,
        label,
        description: desc,
        selected: *id == selected,
    })
    .collect()
}

/// GET /api/runtime/view/agents — index shell.
#[utoipa::path(
    get,
    path = "/api/runtime/view/agents",
    responses(
        (status = 200, description = "Rendered agents index view", content_type = "text/html"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn agents_shell(
    State(state): State<ApiState>,
) -> Result<Html<String>, StatusCode> {
    let agents = list_agents(state.ship_dir());
    let total = agents.len();
    let ctx = AgentsIndexCtx { agents, total };

    // Try .ship/views/agents/index.html.j2 first (new canonical location).
    if let Some(html) = try_render_view("agents", "index.html.j2", &ctx) {
        return Ok(Html(html));
    }

    // Fallback to embedded template.
    templates::render("agents/shell.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "agents shell render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// GET /api/runtime/view/agents/detail/{id} — detail shell.
#[utoipa::path(
    get,
    path = "/api/runtime/view/agents/detail/{id}",
    params(("id" = String, Path, description = "Agent id")),
    responses(
        (status = 200, description = "Rendered agent detail view", content_type = "text/html"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Agent not found", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn agent_detail(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some(agent) = load_agent(state.ship_dir(), &id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let tiers = build_tiers(&agent.permissions_tier);
    let ctx = AgentDetailCtx { agent, tiers };

    // Try .ship/views/agents/detail.html.j2 first (new canonical location).
    if let Some(html) = try_render_view("agents", "detail.html.j2", &ctx) {
        return Ok(Html(html));
    }

    // Fallback to embedded template.
    templates::render("agents/detail.html.j2", &ctx)
        .map(Html)
        .map_err(|e| {
            tracing::error!(error = %e, "agent detail render failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// POST /api/runtime/view/agents/new — structured not-implemented response.
#[utoipa::path(
    post,
    path = "/api/runtime/view/agents/new",
    responses(
        (status = 501, description = "Not implemented — agents are created via files under .ship/agents/", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "views"
)]
pub async fn agents_create(
    State(_state): State<ApiState>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "ok": false,
            "error": "not_implemented",
            "message": "Creating agents from the view is not implemented yet. Add a file under .ship/agents/<id>.jsonc."
        })),
    )
}

#[cfg(test)]
#[path = "views_agents_tests.rs"]
mod tests;
