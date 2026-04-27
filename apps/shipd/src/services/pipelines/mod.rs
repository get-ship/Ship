//! PipelineService — HTTP surface at `/api/v1/pipelines/*`.
//!
//! Canonical shape per `.artifacts/daemon-services.md` §3:
//!
//! - GET    /api/v1/pipelines                       list (project + user)
//! - GET    /api/v1/pipelines/{name}                raw document
//! - PUT    /api/v1/pipelines/{name}                full replace
//! - PATCH  /api/v1/pipelines/{name}                RFC 6902 ops
//! - DELETE /api/v1/pipelines/{name}                trash
//! - POST   /api/v1/pipelines/{name}/dispatch       kick off a run
//! - POST   /api/v1/pipelines/runs/{slug}/cancel    cancel a run
//! - GET    /api/v1/pipelines/runs                  run history
//! - GET    /api/v1/pipelines/runs/{slug}           single run detail
//! - GET    /api/v1/pipelines/runs/{slug}/log/stream SSE of the agent log
//!
//! Dispatch / cancel / run-detail / log-stream delegate to the
//! kernel-backed handlers in `runtime_api` + `logs_api` — those
//! functions emit the correct events already; duplicating them would
//! fork the write path.

pub mod focus;
pub mod history;
pub mod run_detail;
pub mod service;
pub mod shape;
pub mod state;
pub mod summaries;
pub mod view_state;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Deserialize;

use crate::rest_api::ApiState;
use super::files::Scope;
use service::PutArgs;

// ── URL scope parsing ────────────────────────────────────────────────

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

// ── Response helpers ─────────────────────────────────────────────────

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
        Parse(m)            => err(StatusCode::BAD_REQUEST, "parse", m),
        Patch(m)            => err(StatusCode::BAD_REQUEST, "patch", m),
        Io(m)               => err(StatusCode::INTERNAL_SERVER_ERROR, "io", m),
        ScopeUnavailable(s) => err(StatusCode::BAD_REQUEST, "scope_unavailable",
                                   format!("{s:?} scope not configured")),
    }
}

// ── Definition CRUD ──────────────────────────────────────────────────

/// GET /api/v1/pipelines
pub async fn list(State(state): State<ApiState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "items": state.pipelines().list() }))
}

/// GET /api/v1/pipelines/{name}
pub async fn get(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let entry = state.pipelines().get(&name, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "name": entry.name,
        "scope": entry.scope,
        "format": entry.format,
        "doc": entry.doc,
    })))
}

/// PUT /api/v1/pipelines/{name}
pub async fn put(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(doc): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let explicit_scope = parse_scope(&q)?;
    let target = state
        .pipelines()
        .put(PutArgs { name: name.clone(), doc, scope: explicit_scope })
        .map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "name": name,
        "scope": state::ScopeTag::from(target),
    })))
}

/// PATCH /api/v1/pipelines/{name}
pub async fn patch(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(q): Query<ScopeQuery>,
    Json(patch_body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let patch: json_patch::Patch = serde_json::from_value(patch_body).map_err(|e| {
        err(StatusCode::BAD_REQUEST, "bad_patch", format!("bad patch shape: {e}"))
    })?;
    let resolved = state.pipelines().patch(&name, scope, &patch).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "name": name,
        "scope": state::ScopeTag::from(resolved),
    })))
}

/// DELETE /api/v1/pipelines/{name}
pub async fn delete(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let scope = parse_scope(&q)?;
    let resolved = state.pipelines().delete(&name, scope).map_err(map_err)?;
    Ok(Json(serde_json::json!({
        "ok": true, "name": name,
        "scope": state::ScopeTag::from(resolved),
    })))
}

// ── Dispatch / cancel / run detail — thin wrappers over runtime_api ──

/// POST /api/v1/pipelines/{name}/dispatch
///
/// Accepts the same body shape as `runtime_api::DispatchRequest` minus
/// `pipeline` (the name is taken from the URL).
pub async fn dispatch(
    state: State<ApiState>,
    auth: axum::Extension<crate::auth::AuthContext>,
    Path(name): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let mut merged = body;
    if let Some(obj) = merged.as_object_mut() {
        obj.insert("pipeline".to_string(), serde_json::Value::String(name));
    } else {
        merged = serde_json::json!({ "pipeline": name });
    }
    let req: crate::runtime_api::DispatchRequest = match serde_json::from_value(merged) {
        Ok(r) => r,
        Err(e) => return err(
            StatusCode::BAD_REQUEST, "bad_body",
            format!("dispatch body: {e}"),
        ).into_response(),
    };
    crate::runtime_api::dispatch_pipeline(state, auth, Json(req)).await.into_response()
}

/// POST /api/v1/pipelines/runs/{slug}/cancel — delegates to the job-level
/// handler which already emits the right event.
pub async fn cancel_run(
    state: State<ApiState>,
    auth: axum::Extension<crate::auth::AuthContext>,
    Path(slug): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let req: crate::runtime_api::CancelJobRequest =
        serde_json::from_value(body).unwrap_or(crate::runtime_api::CancelJobRequest {
            reason: "operator-cancelled".to_string(),
        });
    crate::runtime_api::cancel_job(state, auth, Path(slug), Json(req))
        .await
        .into_response()
}

// ── Run listing / detail ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RunsQuery {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// GET /api/v1/pipelines/runs — paginated history filterable by pipeline
/// name, branch, and a size cap (default 100, max 500).
pub async fn list_runs(
    State(state): State<ApiState>,
    Query(q): Query<RunsQuery>,
) -> Result<Json<serde_json::Value>, Resp> {
    let jobs = runtime::projections::job::load_jobs_at(state.ship_dir())
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "io", e.to_string()))?;
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let mut items: Vec<&runtime::projections::job::JobRecord> = jobs
        .values()
        .filter(|j| match q.name.as_deref() {
            Some(n) if !n.is_empty() => {
                crate::views::state::pipeline_name_for_job_public(j)
                    .as_deref() == Some(n)
            }
            _ => true,
        })
        .filter(|j| match q.branch.as_deref() {
            Some(b) if !b.is_empty() => j.branch == b,
            _ => true,
        })
        .collect();
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    items.truncate(limit);
    let rendered: Vec<serde_json::Value> = items
        .into_iter()
        .map(|j| serde_json::to_value(j).unwrap_or_default())
        .collect();
    Ok(Json(serde_json::json!({ "ok": true, "items": rendered })))
}

/// GET /api/v1/pipelines/runs/{slug}
pub async fn get_run(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, Resp> {
    let jobs = runtime::projections::job::load_jobs_at(state.ship_dir())
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "io", e.to_string()))?;
    let job = jobs
        .values()
        .find(|j| j.slug == slug)
        .cloned()
        .ok_or_else(|| err(
            StatusCode::NOT_FOUND, "not_found",
            format!("no run with slug `{slug}`"),
        ))?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "run": serde_json::to_value(&job).unwrap_or_default(),
    })))
}
