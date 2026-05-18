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


use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;

use crate::access::RuntimeAccess;
use crate::auth::{AuthContext};
use crate::routes::rest_api::ApiState;
use ship_platform::services::files::Scope;
use ship_platform::services::skills::service::CreateArgs;

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


fn map_err(e: ship_platform::services::skills::state::Error) -> Resp {
    use ship_platform::services::skills::state::Error::*;
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
pub async fn list(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, Resp> {
    Ok(Json(serde_json::json!({ "ok": true, "items": state.services().skills().list() })))
}

/// GET /api/v1/skills/{id}
pub async fn get(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
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
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().write_frontmatter(&id, scope, &body).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": ship_platform::services::skills::state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct BodyPayload {
    pub body: String,
}

/// PUT /api/v1/skills/{id}/body
pub async fn put_body(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(payload): Json<BodyPayload>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().write_body(&id, scope, &payload.body).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": ship_platform::services::skills::state::ScopeTag::from(resolved),
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
    Extension(_auth): Extension<AuthContext>,
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
        "scope": ship_platform::services::skills::state::ScopeTag::from(resolved),
    })))
}

/// DELETE /api/v1/skills/{id}/files/{*path}
pub async fn delete_file(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path((id, path)): Path<(String, String)>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().delete_file(&id, scope, &path).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id, "path": path,
        "scope": ship_platform::services::skills::state::ScopeTag::from(resolved),
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
    Extension(_auth): Extension<AuthContext>,
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
        "scope": ship_platform::services::skills::state::ScopeTag::from(scope),
    }))))
}

/// DELETE /api/v1/skills/{id}
pub async fn delete(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().delete(&id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": id,
        "scope": ship_platform::services::skills::state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct RenameBody { pub new_id: String }

/// POST /api/v1/skills/{id}/rename
pub async fn rename(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<RenameBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().rename(&id, &body.new_id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": body.new_id,
        "scope": ship_platform::services::skills::state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct DuplicateBody { pub new_id: String }

/// POST /api/v1/skills/{id}/duplicate
pub async fn duplicate(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(body): Json<DuplicateBody>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.services().skills().duplicate(&id, &body.new_id, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "id": body.new_id,
        "scope": ship_platform::services::skills::state::ScopeTag::from(resolved),
    })))
}

#[derive(Deserialize)]
pub struct MoveBody { pub to_scope: String }

/// POST /api/v1/skills/{id}/move
pub async fn move_scope(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
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
        "scope": ship_platform::services::skills::state::ScopeTag::from(to),
    })))
}

// ── Vars (KV-backed, delegates through trait) ─────────────────────────

/// GET /api/v1/skills/{id}/vars
pub async fn get_vars(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
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
    Extension(_auth): Extension<AuthContext>,
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
    Extension(_auth): Extension<AuthContext>,
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

// ── Invoke ───────────────────────────────────────────────────────────
//
// Library-skill path — the most common skill type. A library skill is
// reference material (markdown body + companion files) that gets
// attached to a session's context. There's no process to spawn; the
// "action" is wiring the skill's content into a session and recording
// the lifecycle on the event bus so views can render progress.
//
// Flow:
//   1. Resolve + parse SKILL.md (validate frontmatter exists, get id).
//   2. Mint a run_id (ULID).
//   3. Pick a session_id — reuse the most-recently-active agent session
//      for the workspace if one exists, else use the run_id as a
//      synthetic "library invoke" session id. The UI can navigate to
//      `/sessions/<id>` either way.
//   4. Emit lifecycle on the kernel bus:
//        skill.started
//        skill.attached_to_session
//        skill.completed   (attach is the entire body of work)
//   5. Return `{ ok, run_id, skill_id, session_id, workspace_id, mode,
//      reused_session }`.
//
// Operator + mode skill types are not yet implemented — they require
// either an interpreter (operator) or a profile-compile pipeline (mode)
// that the daemon doesn't expose today. The arc note at
// `.artifacts/sui-arc/skills-invoke-real.md` tracks the gaps.

#[derive(Deserialize, Default)]
pub struct InvokeBody {
    /// Optional per-invocation variable overrides. Stored on the
    /// emitted event so consumers can render the invocation context;
    /// not yet threaded into a runtime.
    #[serde(default)]
    pub vars: Option<serde_json::Map<String, serde_json::Value>>,
    /// Optional workspace correlation — defaults to the active
    /// workspace if the caller doesn't pass one.
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// Optional explicit session to attach to. When omitted, the
    /// daemon picks the most-recently-active agent session in the
    /// workspace, falling back to the run_id as a synthetic session id.
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Per-invocation outcome — kept narrow so the JSON return shape and
/// the test surface agree on the contract.
struct InvokeOutcome {
    run_id: String,
    session_id: String,
    workspace_id: Option<String>,
    mode: &'static str,
    reused_session: bool,
}

/// POST /api/v1/skills/{id}/invoke
pub async fn invoke(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
    body: Option<Json<InvokeBody>>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    // Resolve the skill so we 404 before emitting any started event for
    // a skill the user can't actually run. `load` parses the SKILL.md
    // frontmatter for us; we surface a slim summary on `skill.started`.
    let detail = state.services().skills().get(&id, scope).map_err(map_err)?;

    let body = body.map(|Json(b)| b).unwrap_or_default();
    let run_id = ship_substrate::gen_ulid();
    let workspace_id = body.workspace_id.clone().or_else(active_workspace_id);

    let (session_id, reused_session) = resolve_session_id(
        state.actor_sessions(),
        workspace_id.as_deref(),
        body.session_id.as_deref(),
        &run_id,
    );

    let frontmatter_summary = serde_json::json!({
        "id": detail.id,
        "name": detail
            .frontmatter
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&detail.id),
        "description": detail
            .frontmatter
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    });

    // skill.started — observable on SSE; entity_id is the skill id.
    let started_payload = serde_json::json!({
        "skill_id": id,
        "run_id": run_id,
        "mode": "library",
        "vars": body.vars.clone().unwrap_or_default(),
        "workspace_id": workspace_id,
        "session_id": session_id,
        "frontmatter": frontmatter_summary,
    });
    emit_skill_event(
        &state,
        ship_types::event_types::SKILL_STARTED,
        &id,
        &started_payload,
        workspace_id.as_deref(),
        Some(&session_id),
    )
    .await?;

    // skill.attached_to_session — records the (run, session) binding.
    let attached = ship_types::payload::SkillAttachedToSession {
        skill_id: id.clone(),
        run_id: run_id.clone(),
        session_id: session_id.clone(),
        mode: "library".to_string(),
        workspace_id: workspace_id.clone(),
        reused: reused_session,
    };
    emit_skill_event(
        &state,
        ship_types::event_types::SKILL_ATTACHED_TO_SESSION,
        &id,
        &attached,
        workspace_id.as_deref(),
        Some(&session_id),
    )
    .await?;

    // skill.completed — library attach is synchronous, complete here.
    let completed = ship_types::payload::SkillCompleted {
        skill_id: id.clone(),
        duration_ms: Some(0),
    };
    emit_skill_event(
        &state,
        ship_types::event_types::SKILL_COMPLETED,
        &id,
        &completed,
        workspace_id.as_deref(),
        Some(&session_id),
    )
    .await?;

    let outcome = InvokeOutcome {
        run_id,
        session_id,
        workspace_id,
        mode: "library",
        reused_session,
    };

    Ok(Json(serde_json::json!({
        "ok": true,
        "run_id": outcome.run_id,
        "skill_id": id,
        "session_id": outcome.session_id,
        "workspace_id": outcome.workspace_id,
        "mode": outcome.mode,
        "reused_session": outcome.reused_session,
    })))
}

/// Resolve the session a library invoke binds to. Priority:
///   1. caller-supplied `session_id` (explicit attach)
///   2. most-recently-active agent session for the workspace
///   3. synthetic — the run_id itself stands in as the session
fn resolve_session_id(
    actors: &crate::sessions::ActorRegistry,
    workspace_id: Option<&str>,
    explicit: Option<&str>,
    run_id: &str,
) -> (String, bool) {
    if let Some(sid) = explicit {
        if !sid.is_empty() {
            return (sid.to_string(), true);
        }
    }
    if let Some(ws) = workspace_id {
        // list() is already sorted most-recent-first.
        for meta in actors.list() {
            if meta.workspace_id == ws
                && matches!(meta.kind, crate::sessions::SessionKind::Agent)
            {
                return (meta.id, true);
            }
        }
    }
    (run_id.to_string(), false)
}

async fn emit_skill_event<P: serde::Serialize>(
    state: &ApiState,
    event_type: &'static str,
    skill_id: &str,
    payload: &P,
    workspace_id: Option<&str>,
    session_id: Option<&str>,
) -> Result<(), Resp> {
    let envelope = ship_substrate::events::EventEnvelope::new(event_type, skill_id, payload)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "emit", e.to_string()))?;
    let ctx = ship_substrate::events::EmitContext {
        caller_kind: ship_substrate::events::CallerKind::Runtime,
        skill_id: Some(skill_id.to_string()),
        project_id: workspace_id.map(str::to_string),
        session_id: session_id.map(str::to_string),
    };
    state
        .kernel()
        .lock()
        .await
        .emit(envelope, &ctx)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "emit", e.to_string()))?;
    Ok(())
}

fn active_workspace_id() -> Option<String> {
    ship_platform::config::get_setting("shell.project.active")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
}
