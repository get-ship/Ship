//! RuleService — HTTP surface at `/api/v1/rules/*`.
//!
//! Rules are plain markdown documents at `<scope>/rules/<id>.md`. PATCH
//! is intentionally omitted — rules carry no structured payload, so the
//! PUT/POST + full-body flow is the only sane write path.
//!
//! - GET    /api/v1/rules                list (project + user, scope-annotated)
//! - GET    /api/v1/rules/{id}           markdown body
//! - PUT    /api/v1/rules/{id}           replace body
//! - DELETE /api/v1/rules/{id}           trash
//! - POST   /api/v1/rules                create { id, content, scope? }

pub mod state;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;

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
        Io(m)               => err(StatusCode::INTERNAL_SERVER_ERROR, "io", m),
        ScopeUnavailable(s) => err(StatusCode::BAD_REQUEST, "scope_unavailable",
                                   format!("{s:?} scope not configured")),
    }
}

fn files(state: &ApiState) -> FilesService {
    FilesService::resolve_roots(state.ship_dir())
}

/// GET /api/v1/rules
pub async fn list(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let svc = files(&state);
    let items = state::list(&svc);
    Json(serde_json::json!({ "ok": true, "items": items }))
}

/// GET /api/v1/rules/{id}
pub async fn get(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    let (resolved, body) = state::load_body(&svc, &id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "id": id,
        "scope": state::ScopeTag::from(resolved),
        "content": body,
    })))
}

/// PUT /api/v1/rules/{id}
/// Body: `{ "content": "..." }` or the raw string inside the JSON value.
pub async fn put(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let explicit_scope = parse_scope(&q)?;
    let content = extract_content(&body)?;
    let svc = files(&state);
    let target_scope = match explicit_scope {
        Some(s) => s,
        None => match state::find(&svc, &id) { Some((s, _)) => s, None => Scope::Project },
    };
    state::write_body(&svc, &id, target_scope, &content, true).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(target_scope),
    })))
}

/// DELETE /api/v1/rules/{id}
pub async fn delete(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let svc = files(&state);
    let resolved = state::delete_body(&svc, &id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// POST /api/v1/rules
#[derive(Deserialize)]
pub struct CreateBody {
    pub id: String,
    pub content: String,
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
    state::write_body(&svc, &body.id, target, &body.content, false).map_err(map_err)?;
    Ok(created(serde_json::json!({
        "ok": true,
        "id": body.id,
        "scope": state::ScopeTag::from(target),
    })))
}

// Accept either `{"content":"..."}` or a bare JSON string in the body.
fn extract_content(body: &serde_json::Value) -> Result<String, Resp> {
    if let Some(s) = body.as_str() {
        return Ok(s.to_string());
    }
    if let Some(s) = body.get("content").and_then(|v| v.as_str()) {
        return Ok(s.to_string());
    }
    Err(err(
        StatusCode::BAD_REQUEST, "bad_body",
        "expected a string body or { content: string }",
    ))
}
