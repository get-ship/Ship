//! PermissionService — HTTP surface at `/api/v1/permissions/*`.
//!
//! Preset entries in a single `<scope>/permissions.jsonc`. Same shape as
//! McpService (singleton-file-with-entries) but keys live at the file root
//! rather than under a sub-object.
//!
//! - GET    /api/v1/permissions                 list presets
//! - GET    /api/v1/permissions/{id}            one preset
//! - PUT    /api/v1/permissions/{id}            replace
//! - PATCH  /api/v1/permissions/{id}            RFC 6902
//! - DELETE /api/v1/permissions/{id}            remove
//! - POST   /api/v1/permissions                 create { id, preset, scope? }
//! - GET    /api/v1/permissions/tool-patterns   autocomplete catalog
//!
//! The `tool-patterns` endpoint composes: hardcoded provider builtins +
//! curated Bash shapes + wildcards for every MCP server returned by
//! `McpService::list()` (scope-agnostic).

pub mod state;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

use crate::rest_api::ApiState;
use super::files::{FilesService, Scope};

#[derive(Debug, Deserialize)]
pub struct ScopeQuery {
    #[serde(default)]
    pub scope: Option<String>,
}

fn parse_scope(q: &ScopeQuery) -> Result<Option<Scope>, Resp> {
    match q.scope.as_deref() {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(err(
            StatusCode::BAD_REQUEST, "bad_scope",
            format!("scope must be 'project' or 'user', got `{other}`"),
        )),
    }
}

fn parse_scope_required(s: &str) -> Result<Scope, Resp> {
    match s {
        "project" => Ok(Scope::Project),
        "user" => Ok(Scope::User),
        other => Err(err(
            StatusCode::BAD_REQUEST, "bad_scope",
            format!("scope must be 'project' or 'user', got `{other}`"),
        )),
    }
}

type Resp = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, code: &str, message: impl Into<String>) -> Resp {
    (status, Json(serde_json::json!({ "ok": false, "error": code, "message": message.into() })))
}

fn created(value: serde_json::Value) -> Resp { (StatusCode::CREATED, Json(value)) }

fn map_err(e: state::Error) -> Resp {
    use state::Error::*;
    match e {
        NotFound(m)         => err(StatusCode::NOT_FOUND, "not_found", m),
        AlreadyExists(m)    => err(StatusCode::CONFLICT, "exists", m),
        BadId(m)            => err(StatusCode::BAD_REQUEST, "bad_id", m),
        Parse(m)            => err(StatusCode::BAD_REQUEST, "parse", m),
        Patch(m)            => err(StatusCode::BAD_REQUEST, "patch", m),
        Io(m)               => err(StatusCode::INTERNAL_SERVER_ERROR, "io", m),
        ScopeUnavailable(s) => err(StatusCode::BAD_REQUEST, "scope_unavailable",
                                   format!("{s:?} scope not configured")),
    }
}

fn files(state: &ApiState) -> FilesService {
    FilesService::resolve_roots(state.ship_dir())
}

/// GET /api/v1/permissions
pub async fn list(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let svc = files(&state);
    let items = state::list(&svc);
    Json(serde_json::json!({ "ok": true, "items": items }))
}

/// GET /api/v1/permissions/{id}
pub async fn get(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    let (resolved, entry) = state::load_entry(&svc, &id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "id": id,
        "scope": state::ScopeTag::from(resolved),
        "preset": entry,
    })))
}

/// PUT /api/v1/permissions/{id}
pub async fn put(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let explicit_scope = parse_scope(&q)?;
    let svc = files(&state);
    let target_scope = match explicit_scope {
        Some(s) => s,
        None => state::find(&svc, &id).unwrap_or(Scope::Project),
    };
    state::write_entry(&svc, &id, target_scope, &body, true).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(target_scope),
    })))
}

/// PATCH /api/v1/permissions/{id}
pub async fn patch(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(patch_body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let patch: json_patch::Patch = serde_json::from_value(patch_body)
        .map_err(|e| err(StatusCode::BAD_REQUEST, "bad_patch", format!("bad patch shape: {e}")))?;
    let svc = files(&state);
    let resolved = state::patch_entry(&svc, &id, scope, &patch).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// DELETE /api/v1/permissions/{id}
pub async fn delete(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    let resolved = state::delete_entry(&svc, &id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// POST /api/v1/permissions
#[derive(Deserialize)]
pub struct CreateBody {
    pub id: String,
    pub preset: serde_json::Value,
    #[serde(default)]
    pub scope: Option<String>,
}

pub async fn create(
    State(state): State<ApiState>,
    Json(body): Json<CreateBody>,
) -> Result<Resp, Resp> {
    let target = match body.scope.as_deref() {
        None | Some("") => Scope::Project,
        Some(s) => parse_scope_required(s)?,
    };
    let svc = files(&state);
    state::write_entry(&svc, &body.id, target, &body.preset, false).map_err(map_err)?;
    Ok(created(serde_json::json!({
        "ok": true,
        "id": body.id,
        "scope": state::ScopeTag::from(target),
    })))
}

// ── tool-patterns — composed catalog ────────────────────────────────

#[derive(Serialize)]
pub struct ToolPattern {
    pub pattern: String,
    pub label: String,
    /// "provider" | "bash" | "mcp-<server>"
    pub source: String,
}

const PROVIDER_BUILTINS: &[&str] = &[
    "Read", "Write", "Edit", "Glob", "Grep", "Bash",
    "NotebookEdit", "WebFetch", "WebSearch", "Task",
];

const BASH_SHAPES: &[(&str, &str)] = &[
    ("Bash(git *)", "Bash git — all git subcommands"),
    ("Bash(git reset*)", "Bash git — reset (destructive)"),
    ("Bash(git push --force*)", "Bash git — force push (destructive)"),
    ("Bash(npm *)", "Bash npm — all npm subcommands"),
    ("Bash(pnpm *)", "Bash pnpm"),
    ("Bash(cargo *)", "Bash cargo"),
    ("Bash(ship *)", "Bash ship — Ship CLI"),
    ("Bash(just *)", "Bash just"),
    ("Bash(rm -rf *)", "Bash rm — recursive delete (destructive)"),
];

/// GET /api/v1/permissions/tool-patterns
///
/// Provider builtins + curated Bash shapes + wildcard entries for every
/// MCP server known to the daemon. Service-to-service: calls
/// `McpService::list` state function directly.
pub async fn tool_patterns(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let svc = files(&state);
    let mut out: Vec<ToolPattern> = Vec::new();
    for t in PROVIDER_BUILTINS {
        out.push(ToolPattern {
            pattern: (*t).into(),
            label: (*t).into(),
            source: "provider".into(),
        });
    }
    for (p, l) in BASH_SHAPES {
        out.push(ToolPattern {
            pattern: (*p).into(),
            label: (*l).into(),
            source: "bash".into(),
        });
    }
    for entry in super::mcp::state::list(&svc) {
        let pattern = format!("mcp__{}__*", entry.id);
        out.push(ToolPattern {
            pattern,
            label: format!("mcp · {} · all tools", entry.id),
            source: format!("mcp-{}", entry.id),
        });
    }
    Json(serde_json::json!({ "ok": true, "items": out }))
}
