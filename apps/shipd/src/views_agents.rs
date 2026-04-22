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
use crate::views::state::{AgentDetail, AgentSummary, CreateAgentError, CreateAgentRequest, create_agent, list_agents, load_agent};
use crate::views::templates;

/// Try rendering a template from `.ship/views/<view>/<file>`.
/// Returns `None` if the view directory or file does not exist, or if
/// rendering fails (in which case the caller falls back to embedded templates).
///
/// Also splices any `packs:` script tags declared in the view manifest into
/// the rendered `<head>`. The fragment extractor in view_morph_target
/// preserves module `<script src=...>` from head when it strips to body.
fn try_render_view<C: Serialize>(view: &str, file: &str, ctx: &C) -> Option<String> {
    let (view_dir, _source) = resolve_view_dir(view)?;
    let source = std::fs::read_to_string(view_dir.join(file)).ok()?;
    let template_name = format!("{view}/{file}");
    let html = templates::render_view_source(&template_name, &source, ctx).ok()?;
    let packs = crate::views::read_view_packs(&view_dir);
    Some(crate::views::inject_packs(html, &packs, false))
}

/// Context for the create-agent wizard view. Server-driven: the template
/// reads every list out of this struct, never hardcodes catalogs.
#[derive(Serialize)]
pub struct AgentsCreateCtx {
    pub templates: Vec<AgentTemplate>,
    pub permission_presets: Vec<PermissionPreset>,
    pub available_providers: Vec<String>,
}

#[derive(Serialize)]
pub struct AgentsIndexCtx {
    pub agents: Vec<AgentSummary>,
    pub total: usize,
    /// Starter templates surfaced to the create-wizard (`$mode == 'create'`
    /// subtree in index.html.j2). Driven by the daemon so the view never
    /// hardcodes template catalogs.
    pub templates: Vec<AgentTemplate>,
    /// Permission presets for the wizard's wiring step. Same contract as
    /// `TierRow` in the detail view but without the `selected` flag —
    /// the wizard tracks selection client-side via a signal.
    pub permission_presets: Vec<PermissionPreset>,
    /// Providers the agent may compile to. Today a fixed catalog; will
    /// become configurable per-project.
    pub available_providers: Vec<String>,
}

/// A starter template for the create-wizard. Pre-wires skills + MCP + a
/// permission preset so a user can create a productive agent in seconds.
#[derive(Serialize, Clone)]
pub struct AgentTemplate {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// Avatar glyph initial for the preview tile.
    pub letter: &'static str,
    /// Conic-gradient hue key (matches `.hue-<name>` in the view CSS).
    pub hue: &'static str,
    pub skill_ids: Vec<&'static str>,
    pub permission_preset: &'static str,
}

#[derive(Serialize, Clone)]
pub struct PermissionPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

fn builtin_templates() -> Vec<AgentTemplate> {
    vec![
        AgentTemplate { id: "blank",        label: "Blank",         description: "Start from nothing. Just an identity.",         letter: "?", hue: "slate",  skill_ids: vec![],                                      permission_preset: "standard" },
        AgentTemplate { id: "lane",         label: "Language Lane", description: "Narrow agent scoped to one stack. Sandbox preset.", letter: "L", hue: "red",    skill_ids: vec!["rust-compile", "tdd", "git-hygiene"],  permission_preset: "sandbox"  },
        AgentTemplate { id: "reviewer",     label: "Reviewer",      description: "Read-only critic for PRs and diffs.",           letter: "R", hue: "amber",  skill_ids: vec!["diff-review", "a11y-audit"],          permission_preset: "read-only" },
        AgentTemplate { id: "orchestrator", label: "Orchestrator",  description: "Routes work to specialist agents.",             letter: "O", hue: "violet", skill_ids: vec!["cmdr", "spawn-agent", "ship-coordination", "ship-agents", "write-adr"], permission_preset: "standard" },
        AgentTemplate { id: "writer",       label: "Test Writer",   description: "TDD loop — writes failing tests first.",        letter: "T", hue: "teal",   skill_ids: vec!["tdd", "rust-compile", "git-hygiene"], permission_preset: "standard" },
        AgentTemplate { id: "investigator", label: "Investigator",  description: "Reads, searches, summarises. Never writes.",    letter: "I", hue: "blue",   skill_ids: vec!["docs-first", "a11y-audit"],          permission_preset: "read-only" },
    ]
}

fn builtin_permission_presets() -> Vec<PermissionPreset> {
    vec![
        PermissionPreset { id: "standard",   label: "Standard",   description: "Human confirms risky actions before they run." },
        PermissionPreset { id: "autonomous", label: "Autonomous", description: "Agent acts end-to-end; safety rails still apply." },
        PermissionPreset { id: "read-only",  label: "Read-only",  description: "Agent inspects and reports. No writes." },
        PermissionPreset { id: "sandbox",    label: "Sandbox",    description: "Allow anything inside a worktree; block outside." },
    ]
}

fn builtin_providers() -> Vec<String> {
    vec!["claude", "codex", "cursor", "gemini", "opencode"]
        .into_iter()
        .map(String::from)
        .collect()
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
/// GET /api/runtime/view/agents/detail/{id} — default detail view.
///
/// Wire/canvas layout is the canonical detail surface (matches the
/// design's `detailLayout=wire`). The panels layout lives at
/// `/api/runtime/view/agents/detail/{id}/settings` via `agent_settings`.
pub async fn agent_detail(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some(agent) = load_agent(state.ship_dir(), &id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let (nodes, edges) = compute_wire_layout(&agent);
    let ctx = AgentWireCtx { agent, nodes, edges };
    if let Some(html) = try_render_view("agents", "wire.html.j2", &ctx) {
        return Ok(Html(html));
    }
    Err(StatusCode::NOT_FOUND)
}

/// GET /api/runtime/view/agents/detail/{id}/settings — panels layout
/// (the pre-wire detail view). Matches the design's `detailLayout=settings`.
pub async fn agent_settings(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some(agent) = load_agent(state.ship_dir(), &id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let tiers = build_tiers(&agent.permissions_tier);
    let ctx = AgentDetailCtx { agent, tiers };
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

/// A positioned node on the wire canvas. `kind` drives color/shape in
/// the template; the server owns all layout so the template stays
/// declarative.
#[derive(Serialize)]
pub struct WireNode {
    pub kind: &'static str, // agent | skill | mcp | rule | perm | prov
    pub id: String,
    pub title: String,
    pub sub: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
}

/// A bezier edge between the center agent node and a peripheral.
#[derive(Serialize)]
pub struct WireEdge {
    pub kind: &'static str,
    pub id: String, // matches the peripheral node's id
    pub path_d: String,
    pub dashed: bool,
}

#[derive(Serialize)]
pub struct AgentWireCtx {
    pub agent: AgentDetail,
    pub nodes: Vec<WireNode>,
    pub edges: Vec<WireEdge>,
}

fn compute_wire_layout(agent: &AgentDetail) -> (Vec<WireNode>, Vec<WireEdge>) {
    // Canvas: 1000x600 viewBox. Agent anchored center-left.
    let agent_x = 420.0_f32;
    let agent_y = 240.0_f32;
    let agent_w = 220.0_f32;
    let agent_h = 80.0_f32;

    let mut nodes: Vec<WireNode> = Vec::new();
    nodes.push(WireNode {
        kind: "agent",
        id: agent.id.clone(),
        title: agent.name.clone(),
        sub: agent.id.clone(),
        x: agent_x,
        y: agent_y,
        w: agent_w,
    });

    // Helper: distribute N items down a column between y0..y1.
    fn column(n: usize, y0: f32, y1: f32) -> Vec<f32> {
        if n == 0 { return vec![] }
        if n == 1 { return vec![(y0 + y1) / 2.0] }
        let step = (y1 - y0) / (n as f32 - 1.0).max(1.0);
        (0..n).map(|i| y0 + step * i as f32).collect()
    }

    let node_w: f32 = 160.0;

    // Skills — left column.
    for (i, y) in column(agent.skills.len(), 60.0, 420.0).into_iter().enumerate() {
        let s = &agent.skills[i];
        nodes.push(WireNode {
            kind: "skill",
            id: s.clone(),
            title: s.clone(),
            sub: "project skill".into(),
            x: 70.0,
            y,
            w: node_w,
        });
    }

    // MCP — top row centered above agent, one per server.
    for (i, s) in agent.mcp_servers.iter().enumerate() {
        let x = 280.0 + (i as f32) * 180.0;
        nodes.push(WireNode {
            kind: "mcp",
            id: s.clone(),
            title: s.clone(),
            sub: "mcp server".into(),
            x,
            y: 90.0,
            w: node_w,
        });
    }

    // Rules — bottom-center.
    if !agent.rules.is_empty() {
        nodes.push(WireNode {
            kind: "rule",
            id: "rules".into(),
            title: format!("{} rule{}", agent.rules.len(), if agent.rules.len() == 1 { "" } else { "s" }),
            sub: "always-on + inline".into(),
            x: 470.0,
            y: 470.0,
            w: node_w,
        });
    }

    // Permissions — bottom-left.
    nodes.push(WireNode {
        kind: "perm",
        id: "permissions".into(),
        title: capitalize(&agent.permissions_tier),
        sub: "permission preset".into(),
        x: 220.0,
        y: 470.0,
        w: node_w,
    });

    // Providers — right column.
    for (i, y) in column(agent.providers.len(), 100.0, 420.0).into_iter().enumerate() {
        let p = &agent.providers[i];
        nodes.push(WireNode {
            kind: "prov",
            id: p.clone(),
            title: capitalize(p),
            sub: "compiled target".into(),
            x: 830.0,
            y,
            w: node_w,
        });
    }

    // Edges: from agent to each peripheral. Pick the agent port based on
    // whether the node is left/right/top/bottom.
    let agent_left = (agent_x, agent_y + agent_h / 2.0);
    let agent_right = (agent_x + agent_w, agent_y + agent_h / 2.0);
    let agent_top = (agent_x + agent_w / 2.0, agent_y);
    let agent_bottom = (agent_x + agent_w / 2.0, agent_y + agent_h);

    let mut edges: Vec<WireEdge> = Vec::new();
    for n in nodes.iter().skip(1) {
        let (fx, fy);
        let tx;
        let ty = n.y + 30.0; // node vertical center
        if n.x < agent_x {
            fx = agent_left.0; fy = agent_left.1;
            tx = n.x + node_w; // right edge of left-side peripheral
        } else if n.x > agent_x + agent_w {
            fx = agent_right.0; fy = agent_right.1;
            tx = n.x;
        } else if n.y < agent_y {
            fx = agent_top.0; fy = agent_top.1;
            tx = n.x + node_w / 2.0;
        } else {
            fx = agent_bottom.0; fy = agent_bottom.1;
            tx = n.x + node_w / 2.0;
        }
        let mx = (fx + tx) / 2.0;
        let path_d = format!("M{fx:.0},{fy:.0} C{mx:.0},{fy:.0} {mx:.0},{ty:.0} {tx:.0},{ty:.0}");
        edges.push(WireEdge {
            kind: n.kind,
            id: n.id.clone(),
            path_d,
            dashed: n.kind == "rule",
        });
    }

    (nodes, edges)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(c).collect(),
    }
}

/// GET /api/runtime/view/agents/{id}/wire — graph/canvas detail view.
pub async fn agent_wire(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some(agent) = load_agent(state.ship_dir(), &id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let (nodes, edges) = compute_wire_layout(&agent);
    let ctx = AgentWireCtx { agent, nodes, edges };
    if let Some(html) = try_render_view("agents", "wire.html.j2", &ctx) {
        return Ok(Html(html));
    }
    Err(StatusCode::NOT_FOUND)
}

/// GET /api/runtime/view/agents/create — create-agent wizard shell.
/// Every catalog the wizard needs (templates, permission presets, providers)
/// is in the context. The view template never hardcodes.
pub async fn agents_create_view(
    State(_state): State<ApiState>,
) -> Result<Html<String>, StatusCode> {
    let ctx = AgentsCreateCtx {
        templates: builtin_templates(),
        permission_presets: builtin_permission_presets(),
        available_providers: builtin_providers(),
    };
    if let Some(html) = try_render_view("agents", "create.html.j2", &ctx) {
        return Ok(Html(html));
    }
    Err(StatusCode::NOT_FOUND)
}

/// POST /api/runtime/view/agents/new — write a new agent JSONC.
///
/// Payload shape matches the create-wizard's step-3 JSON. Writes under the
/// project dir when available, global otherwise. Returns a structured
/// response the view can morph into a success or error state.
pub async fn agents_create(
    State(state): State<ApiState>,
    Json(req): Json<CreateAgentRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match create_agent(state.ship_dir(), &req) {
        Ok(result) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "ok": true,
                "id": result.id,
                "path": result.path,
                "scope": result.scope,
                "redirect": format!("/agents/{}", result.id),
            })),
        ),
        Err(CreateAgentError::BadId) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": "bad_id",
                "message": "id must match ^[a-z0-9-]+$",
            })),
        ),
        Err(CreateAgentError::BadProvider(p)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": "bad_provider",
                "message": format!("unknown provider `{p}`"),
            })),
        ),
        Err(CreateAgentError::Exists(path)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "ok": false,
                "error": "exists",
                "message": format!("{} already exists", path.display()),
            })),
        ),
        Err(CreateAgentError::Io(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "ok": false,
                "error": "io",
                "message": e.to_string(),
            })),
        ),
    }
}

#[cfg(test)]
#[path = "views_agents_tests.rs"]
mod tests;
