//! SkillService — HTTP surface at `/api/v1/skills/*`.
//!
//! Canonical shape per `.artifacts/daemon-services.md` §3.SkillService:
//!
//! - GET    /api/v1/skills                              list (project + user)
//! - GET    /api/v1/skills/{id}                         detail (frontmatter + body + files)
//! - PUT    /api/v1/skills/{id}/frontmatter             replace frontmatter
//! - PUT    /api/v1/skills/{id}/body                    replace body
//! - PUT    /api/v1/skills/{id}/files/{*path}           arbitrary file
//! - DELETE /api/v1/skills/{id}/files/{*path}           trash a file
//! - POST   /api/v1/skills                              create / scaffold
//! - DELETE /api/v1/skills/{id}                         trash
//! - POST   /api/v1/skills/{id}/rename
//! - POST   /api/v1/skills/{id}/duplicate
//! - POST   /api/v1/skills/{id}/move
//! - GET    /api/v1/skills/{id}/vars
//! - PUT    /api/v1/skills/{id}/vars/{key}
//! - DELETE /api/v1/skills/{id}/vars/{key}
//!
//! Vars delegate to `runtime::{get,set,delete}_skill_var` (KV-backed).
//! The skill's filesystem is the source of truth for everything else.

pub mod service;
pub mod state;
pub mod view_state;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;

use crate::access::RuntimeAccess;
use crate::rest_api::ApiState;
use super::files::Scope;
use service::CreateArgs;

// ── Scope query parsing ───────────────────────────────────────────────

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

// ── Response helpers ──────────────────────────────────────────────────

type Resp = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, code: &str, message: impl Into<String>) -> Resp {
    (status, Json(serde_json::json!({ "ok": false, "error": code, "message": message.into() })))
}

fn map_err(e: state::Error) -> Resp {
    use state::Error::*;
    match e {
        NotFound(m)         => err(StatusCode::NOT_FOUND, "not_found", m),
        AlreadyExists(m)    => err(StatusCode::CONFLICT, "exists", m),
        BadId(m)            => err(StatusCode::BAD_REQUEST, "bad_id", m),
        BadPath(m)          => err(StatusCode::BAD_REQUEST, "bad_path", m),
        Parse(m)            => err(StatusCode::BAD_REQUEST, "parse", m),
        Io(m)               => err(StatusCode::INTERNAL_SERVER_ERROR, "io", m),
        ScopeUnavailable(s) => err(StatusCode::BAD_REQUEST, "scope_unavailable",
                                   format!("{s:?} scope not configured")),
    }
}

// ── List / detail ─────────────────────────────────────────────────────

/// GET /api/v1/skills
pub async fn list(State(state): State<ApiState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "items": state.services().skills().list() }))
}

/// GET /api/v1/skills/{id}
pub async fn get(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let detail = state.services().skills().get(&id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "skill": detail })))
}

// ── Frontmatter / body ────────────────────────────────────────────────

/// PUT /api/v1/skills/{id}/frontmatter
pub async fn put_frontmatter(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().write_frontmatter(&id, scope, &body).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct BodyPayload {
    pub body: String,
}

/// PUT /api/v1/skills/{id}/body
pub async fn put_body(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(payload): Json<BodyPayload>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().write_body(&id, scope, &payload.body).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

// ── File tree ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct FilePayload {
    pub content: String,
}

/// PUT /api/v1/skills/{id}/files/{*path}
pub async fn put_file(
    State(state): State<ApiState>,
    Path((id, path)): Path<(String, String)>,
    Query(q): Query<ScopeQuery>,
    Json(payload): Json<FilePayload>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills()
        .write_file(&id, scope, &path, payload.content.as_bytes())
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id, "path": path,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// DELETE /api/v1/skills/{id}/files/{*path}
pub async fn delete_file(
    State(state): State<ApiState>,
    Path((id, path)): Path<(String, String)>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().delete_file(&id, scope, &path).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id, "path": path,
        "scope": state::ScopeTag::from(resolved),
    })))
}

// ── Lifecycle ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateBody {
    pub id: String,
    #[serde(default)]
    pub scope: Option<String>,
    /// Optional initial frontmatter object. Defaults to a minimal
    /// `{name, description}` derived from the id.
    #[serde(default)]
    pub frontmatter: Option<serde_json::Value>,
    /// Optional initial markdown body. Defaults to an empty string.
    #[serde(default)]
    pub body: Option<String>,
}

/// POST /api/v1/skills
pub async fn create(
    State(state): State<ApiState>,
    Json(body): Json<CreateBody>,
) -> Result<Resp, Resp> {
    let target = match body.scope.as_deref() {
        None | Some("") => None,
        Some(s) => Some(parse_scope_required(s)?),
    };
    let scope = state
        .skills()
        .create(CreateArgs {
            id: body.id.clone(),
            scope: target,
            frontmatter: body.frontmatter,
            body: body.body,
        })
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "ok": true,
        "id": body.id,
        "scope": state::ScopeTag::from(scope),
    }))))
}

/// DELETE /api/v1/skills/{id}
pub async fn delete(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().delete(&id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct RenameBody { pub new_id: String }

/// POST /api/v1/skills/{id}/rename
pub async fn rename(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<RenameBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().rename(&id, &body.new_id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": body.new_id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct DuplicateBody { pub new_id: String }

/// POST /api/v1/skills/{id}/duplicate
pub async fn duplicate(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<DuplicateBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().duplicate(&id, &body.new_id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": body.new_id,
        "scope": state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct MoveBody { pub to_scope: String }

/// POST /api/v1/skills/{id}/move
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
    state.services().skills().move_scope(&id, from_scope, to).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": state::ScopeTag::from(to),
    })))
}

// ── Vars (KV-backed, delegates through trait) ─────────────────────────

/// GET /api/v1/skills/{id}/vars
pub async fn get_vars(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, Resp> {
    state.services().skills().validate_id(&id).map_err(map_err)?;
    let vars = state.services().skills().get_vars(&id).map_err(|e| {
        err(StatusCode::INTERNAL_SERVER_ERROR, "io", e.to_string())
    })?;
    match vars {
        Some(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map.into_iter().collect();
            Ok(Json(serde_json::json!({ "ok": true, "id": id, "vars": obj })))
        }
        None => Ok(Json(serde_json::json!({ "ok": true, "id": id, "vars": null }))),
    }
}

#[derive(Deserialize)]
pub struct SetVarBody {
    /// Value to store. Accepted as a JSON value directly — clients can send
    /// strings, numbers, booleans, etc.
    pub value: serde_json::Value,
}

/// PUT /api/v1/skills/{id}/vars/{key}
pub async fn put_var(
    State(state): State<ApiState>,
    Path((id, key)): Path<(String, String)>,
    Json(body): Json<SetVarBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    state.services().skills().validate_id(&id).map_err(map_err)?;
    state.services().skills().set_var(&id, &key, body.value).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unknown variable") || msg.contains("no assets/vars.json") {
            err(StatusCode::NOT_FOUND, "not_found", msg)
        } else {
            err(StatusCode::INTERNAL_SERVER_ERROR, "io", msg)
        }
    })?;
    Ok(Json(serde_json::json!({ "ok": true, "id": id, "key": key })))
}

/// DELETE /api/v1/skills/{id}/vars/{key}
pub async fn delete_var(
    State(state): State<ApiState>,
    Path((id, key)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, Resp> {
    state.services().skills().validate_id(&id).map_err(map_err)?;
    let had = state.services().skills().delete_var(&id, &key).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unknown variable") || msg.contains("no assets/vars.json") {
            err(StatusCode::NOT_FOUND, "not_found", msg)
        } else {
            err(StatusCode::INTERNAL_SERVER_ERROR, "io", msg)
        }
    })?;
    Ok(Json(serde_json::json!({ "ok": true, "id": id, "key": key, "removed": had })))
}
