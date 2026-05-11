//! AgentService — HTTP surface at `/api/v1/agents/*`.
//!
//! Routes registered by `router::build_ship_routes`. Canonical shape per
//! `.artifacts/daemon-services.md`:
//!
//! - GET    /api/v1/agents                     list (project + user)
//! - GET    /api/v1/agents/{id}                full document (?scope= optional)
//! - PUT    /api/v1/agents/{id}                replace
//! - PATCH  /api/v1/agents/{id}                RFC 6902 ops
//! - DELETE /api/v1/agents/{id}                trash
//! - POST   /api/v1/agents                     create
//! - POST   /api/v1/agents/{id}/rename         { new_id }
//! - POST   /api/v1/agents/{id}/duplicate      { new_id, to_scope? }
//! - POST   /api/v1/agents/{id}/move           { to_scope }
//!
//! Runtime actions (invoke/stop/steer) already live in `runtime_api` and
//! stay there — that module is the runtime-state side of the domain.

pub mod create;
pub mod service;
pub mod settings_view;
pub mod state;
pub mod view_state;
pub mod wire;


use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;

use crate::access::RuntimeAccess;
use crate::rest_api::ApiState;
use super::files::Scope;
use service::{CreateArgs, DuplicateArgs, PutArgs};

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

#[allow(dead_code)] // used by other handlers in this module tree
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

// ── Handlers ─────────────────────────────────────────────────────────

/// GET /api/v1/agents
pub async fn list(State(state): State<ApiState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "items": state.services().agents().list() }))
}

/// GET /api/v1/agents/{id}
pub async fn get(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let entry = state.services().agents().get(&id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "id": entry.id,
        "scope": entry.scope,
        "doc": entry.doc,
    })))
}

/// PUT /api/v1/agents/{id}
pub async fn put(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(doc): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let explicit_scope = parse_scope(&q)?;
    let target = state
        .agents()
        .put(PutArgs { id: id.clone(), doc, scope: explicit_scope })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(target),
    })))
}

/// PATCH /api/v1/agents/{id}
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
    let resolved = state.services().agents().patch(&id, scope, &patch).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// DELETE /api/v1/agents/{id}
pub async fn delete(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().agents().delete(&id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// POST /api/v1/agents
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
        None | Some("") => None,
        Some(s) => Some(parse_scope_required(s)?),
    };
    let scope = state
        .agents()
        .create(CreateArgs { id: body.id.clone(), doc: body.doc, scope: target })
        .map_err(map_err)?;
    Ok(created(serde_json::json!({
        "ok": true,
        "id": body.id,
        "scope": state::ScopeTag::from(scope),
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
    let resolved = state.services().agents().rename(&id, &body.new_id, scope).map_err(map_err)?;
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
    let source_scope = parse_scope(&q)?;
    let to = match body.to_scope.as_deref() {
        None | Some("") => None,
        Some(s) => Some(parse_scope_required(s)?),
    };
    let scope = state
        .agents()
        .duplicate(DuplicateArgs {
            source_id: id,
            source_scope,
            new_id: body.new_id.clone(),
            to_scope: to,
        })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": body.new_id,
        "scope": state::ScopeTag::from(scope),
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
    state.services().agents().move_scope(&id, from_scope, to).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(to),
    })))
}

// Result type alias so axum accepts our error tuples via impl IntoResponse.
// Axum's Json<T> -> Response, so we manually wrap the err path.
impl axum::response::IntoResponse for state::Error {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = map_err(self);
        (status, body).into_response()
    }
}

