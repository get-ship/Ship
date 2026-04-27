//! McpService — HTTP surface at `/api/v1/mcp/*`.
//!
//! Singleton-file-with-entries: each scope has one `mcp.jsonc` with
//! `mcp.servers.<id>` sub-objects. The service exposes those entries as
//! if they were a collection.
//!
//! - GET    /api/v1/mcp                list entries (both scopes, annotated)
//! - GET    /api/v1/mcp/{id}           one entry
//! - PUT    /api/v1/mcp/{id}           replace one entry
//! - PATCH  /api/v1/mcp/{id}           RFC 6902 ops against the entry body
//! - DELETE /api/v1/mcp/{id}           remove entry
//! - POST   /api/v1/mcp                create { id, server, scope? }

pub mod service;
pub mod state;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;

use crate::rest_api::ApiState;
use super::files::Scope;
use service::{CreateArgs, PutArgs};

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

/// GET /api/v1/mcp
pub async fn list(State(state): State<ApiState>) -> Json<serde_json::Value> {
    let items = state.mcp().list();
    Json(serde_json::json!({ "ok": true, "items": items }))
}

/// GET /api/v1/mcp/{id}
pub async fn get(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let entry = state.mcp().get(&id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "id": entry.id,
        "scope": entry.scope,
        "server": entry.server,
    })))
}

/// PUT /api/v1/mcp/{id}
pub async fn put(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let explicit_scope = parse_scope(&q)?;
    let target = state
        .mcp()
        .put(PutArgs { id: id.clone(), server: body, scope: explicit_scope })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(target),
    })))
}

/// PATCH /api/v1/mcp/{id}
pub async fn patch(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(patch_body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let patch: json_patch::Patch = serde_json::from_value(patch_body)
        .map_err(|e| err(StatusCode::BAD_REQUEST, "bad_patch", format!("bad patch shape: {e}")))?;
    let resolved = state.mcp().patch(&id, scope, &patch).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// DELETE /api/v1/mcp/{id}
pub async fn delete(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.mcp().delete(&id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// POST /api/v1/mcp
#[derive(Deserialize)]
pub struct CreateBody {
    pub id: String,
    pub server: serde_json::Value,
    #[serde(default)]
    pub scope: Option<String>,
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
        .mcp()
        .create(CreateArgs { id: body.id.clone(), server: body.server, scope: target })
        .map_err(map_err)?;
    Ok(created(serde_json::json!({
        "ok": true,
        "id": body.id,
        "scope": state::ScopeTag::from(scope),
    })))
}
