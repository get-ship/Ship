//! AgentService — HTTP surface at `/api/agents/*`.
//!
//! Routes registered by `router::build_ship_routes`. Canonical shape per
//! `.artifacts/daemon-services.md`:
//!
//! - GET    /api/agents                     list (project + user)
//! - GET    /api/agents/{id}                full document (?scope= optional)
//! - PUT    /api/agents/{id}                replace
//! - PATCH  /api/agents/{id}                RFC 6902 ops
//! - DELETE /api/agents/{id}                trash
//! - POST   /api/agents                     create
//! - POST   /api/agents/{id}/rename         { new_id }
//! - POST   /api/agents/{id}/duplicate      { new_id, to_scope? }
//! - POST   /api/agents/{id}/move           { to_scope }
//!
//! Runtime actions (invoke/stop/steer) already live in `runtime_api` and
//! stay there — that module is the runtime-state side of the domain.

pub mod state;
pub mod view_state;


use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

use crate::rest_api::ApiState;
use super::files::{FilesService, Scope};

// ── URL scope parsing ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ScopeQuery {
    /// Optional scope selector. Absent ⇒ project when present, else user.
    /// "project" / "user".
    #[serde(default)]
    pub scope: Option<String>,
}

fn parse_scope(q: &ScopeQuery) -> Result<Option<Scope>, Resp> {
    match q.scope.as_deref() {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(err(
            StatusCode::BAD_REQUEST,
            "bad_scope",
            format!("scope must be 'project' or 'user', got `{other}`"),
        )),
    }
}

fn parse_scope_required(s: &str) -> Result<Scope, Resp> {
    match s {
        "project" => Ok(Scope::Project),
        "user" => Ok(Scope::User),
        other => Err(err(
            StatusCode::BAD_REQUEST,
            "bad_scope",
            format!("scope must be 'project' or 'user', got `{other}`"),
        )),
    }
}

// ── Response helpers ─────────────────────────────────────────────────

type Resp = (StatusCode, Json<serde_json::Value>);

fn ok(value: serde_json::Value) -> Resp {
    (StatusCode::OK, Json(value))
}
fn created(value: serde_json::Value) -> Resp {
    (StatusCode::CREATED, Json(value))
}
fn err(status: StatusCode, code: &str, message: impl Into<String>) -> Resp {
    (status, Json(serde_json::json!({ "ok": false, "error": code, "message": message.into() })))
}

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

// ── Handlers ─────────────────────────────────────────────────────────

/// GET /api/agents
pub async fn list(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let svc = files(&state);
    let items = state::list(&svc);
    Json(serde_json::json!({ "ok": true, "items": items }))
}

/// GET /api/agents/{id}
pub async fn get(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    let (resolved, doc) = state::load_doc(&svc, &id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "id": id,
        "scope": state::ScopeTag::from(resolved),
        "doc": doc,
    })))
}

/// PUT /api/agents/{id}
pub async fn put(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(doc): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let explicit_scope = parse_scope(&q)?;
    let svc = files(&state);
    // Resolve scope: explicit wins; else existing file; else project.
    let target_scope = match explicit_scope {
        Some(s) => s,
        None => match state::find(&svc, &id) { Some((s, _)) => s, None => Scope::Project },
    };
    state::write_doc(&svc, &id, target_scope, &doc, true).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(target_scope),
    })))
}

/// PATCH /api/agents/{id}
/// Body is an RFC 6902 patch document: `[{op,path,value?}, ...]`.
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
    let resolved = state::patch_doc(&svc, &id, scope, &patch).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// DELETE /api/agents/{id}
pub async fn delete(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    let resolved = state::delete_doc(&svc, &id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// POST /api/agents
#[derive(Deserialize)]
pub struct CreateBody {
    pub id: String,
    /// "project" (default) or "user".
    #[serde(default)]
    pub scope: Option<String>,
    /// Initial document. The service writes it as-is — clients responsible
    /// for shape. `doc.agent.id` is overwritten with the request id so the
    /// field can't drift from the filename.
    pub doc: serde_json::Value,
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
    // Force agent.id to match the filename; templates don't need to
    // duplicate the id across body + slot.
    let doc = force_agent_id(body.doc, &body.id);
    state::write_doc(&svc, &body.id, target, &doc, false).map_err(map_err)?;
    Ok(created(serde_json::json!({
        "ok": true,
        "id": body.id,
        "scope": state::ScopeTag::from(target),
    })))
}

#[derive(Deserialize)]
pub struct RenameBody { pub new_id: String }

pub async fn rename(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<RenameBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    let resolved = state::rename(&svc, &id, &body.new_id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": body.new_id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct DuplicateBody {
    pub new_id: String,
    /// Optional: duplicate into a different scope. Default = same scope.
    #[serde(default)]
    pub to_scope: Option<String>,
}

pub async fn duplicate(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<DuplicateBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    // Load first so we know which scope we're duplicating FROM.
    let (from_scope, doc) = state::load_doc(&svc, &id, scope).map_err(map_err)?;
    let to = match body.to_scope.as_deref() {
        None | Some("") => from_scope,
        Some(s) => parse_scope_required(s)?,
    };
    let doc = force_agent_id(doc, &body.new_id);
    state::write_doc(&svc, &body.new_id, to, &doc, false).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": body.new_id,
        "scope": state::ScopeTag::from(to),
    })))
}

#[derive(Deserialize)]
pub struct MoveBody { pub to_scope: String }

pub async fn move_scope(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<MoveBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let from_scope = parse_scope(&q)?.ok_or_else(|| err(
        StatusCode::BAD_REQUEST, "missing_scope",
        "source scope is required on move — pass ?scope=project or ?scope=user",
    ))?;
    let to = parse_scope_required(&body.to_scope)?;
    let svc = files(&state);
    state::move_scope(&svc, &id, from_scope, to).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(to),
    })))
}

// ── Helpers ──────────────────────────────────────────────────────────

fn force_agent_id(mut doc: serde_json::Value, id: &str) -> serde_json::Value {
    if let Some(obj) = doc.as_object_mut() {
        let agent_entry = obj
            .entry("agent".to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        if let Some(agent) = agent_entry.as_object_mut() {
            agent.insert("id".into(), serde_json::Value::String(id.into()));
            if !agent.contains_key("name") {
                agent.insert("name".into(), serde_json::Value::String(id.into()));
            }
        }
    }
    doc
}

// Result type alias so axum accepts our error tuples via impl IntoResponse.
// Axum's Json<T> -> Response, so we manually wrap the err path.
impl axum::response::IntoResponse for state::Error {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = map_err(self);
        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct _TypeMarker;
