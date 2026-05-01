//! Specs REST API — CRUD over the `spec` table and pipeline launch.
//!
//! Routes are mounted under `/api/v1/runtime` in lib.rs.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use runtime::db::specs::{self, Spec};
use runtime::events::EventEnvelope;
use runtime::events::job::{JobCreatedPayload, event_types};
use serde::Deserialize;
use std::collections::HashMap;

use crate::rest_api::{ApiState, MeshResponse};

type Resp = (StatusCode, Json<MeshResponse>);

fn ok(data: serde_json::Value) -> Resp {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

fn err(status: StatusCode, msg: impl Into<String>) -> Resp {
    (status, Json(MeshResponse { ok: false, data: serde_json::json!(msg.into()) }))
}

fn spec_to_json(s: &Spec) -> serde_json::Value {
    serde_json::json!({
        "id": s.id,
        "title": s.title,
        "body_path": s.body_path,
        "status": s.status,
        "pipeline": s.pipeline,
        "vars": s.vars,
        "criteria": s.criteria,
        "scope": s.scope,
        "priority": s.priority,
        "created_at": s.created_at.to_rfc3339(),
        "updated_at": s.updated_at.to_rfc3339(),
    })
}

// ---- Request shapes ----

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "title": "Write README",
    "body": "# Write README\n\nSummarize the project in 3 bullet points.\n",
    "pipeline": "single-turn",
    "status": "draft",
    "vars": { "topic": "ship" },
    "criteria": ["README.md exists", "mentions dispatcher"],
    "scope": ["docs/**"],
    "priority": 5
}))]
pub struct CreateSpecRequest {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub pipeline: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "title": "Write README (v2)",
    "status": "ready"
}))]
pub struct UpdateSpecRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub pipeline: Option<Option<String>>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub vars: Option<HashMap<String, String>>,
    #[serde(default)]
    pub criteria: Option<Vec<String>>,
    #[serde(default)]
    pub scope: Option<Vec<String>>,
    #[serde(default)]
    pub priority: Option<Option<u32>>,
}

// ---- Handlers ----

/// GET /api/v1/runtime/specs
#[utoipa::path(
    operation_id = "listSpecs",
    get,
    path = "/api/v1/specs",
    responses(
        (status = 200, description = "List of specs", body = MeshResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn list_specs() -> impl IntoResponse {
    match specs::list_specs() {
        Ok(list) => {
            let data: Vec<_> = list.iter().map(spec_to_json).collect();
            ok(serde_json::json!({ "specs": data }))
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// GET /api/v1/runtime/specs/{id}
#[utoipa::path(
    operation_id = "getSpec",
    get,
    path = "/api/v1/specs/{id}",
    params(("id" = String, Path, description = "Spec id")),
    responses(
        (status = 200, description = "Spec detail with body", body = MeshResponse),
        (status = 404, description = "Spec not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn get_spec(Path(id): Path<String>) -> impl IntoResponse {
    match specs::get_spec(&id) {
        Ok(Some(s)) => {
            let body = specs::load_body(&s.body_path).unwrap_or_default();
            let mut v = spec_to_json(&s);
            v["body"] = serde_json::Value::String(body);
            ok(v)
        }
        Ok(None) => err(StatusCode::NOT_FOUND, format!("spec '{id}' not found")),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// POST /api/v1/runtime/specs
#[utoipa::path(
    operation_id = "createSpec",
    post,
    path = "/api/v1/specs",
    request_body = CreateSpecRequest,
    responses(
        (status = 200, description = "Created spec", body = MeshResponse),
        (status = 400, description = "Invalid payload", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn create_spec(Json(req): Json<CreateSpecRequest>) -> impl IntoResponse {
    let id = runtime::gen_ulid();
    let body_path = format!(".artifacts/specs/{id}.md");

    if let Err(e) = specs::save_body(&body_path, &req.body) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, format!("save body: {e}"));
    }
    let now = Utc::now();
    let spec = Spec {
        id: id.clone(),
        title: req.title,
        body_path,
        status: req.status.unwrap_or_else(|| "draft".into()),
        pipeline: req.pipeline,
        vars: req.vars,
        criteria: req.criteria,
        scope: req.scope,
        priority: req.priority,
        created_at: now,
        updated_at: now,
    };
    match specs::upsert_spec(&spec) {
        Ok(()) => ok(spec_to_json(&spec)),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// PUT /api/v1/runtime/specs/{id}
#[utoipa::path(
    operation_id = "updateSpec",
    put,
    path = "/api/v1/specs/{id}",
    params(("id" = String, Path, description = "Spec id")),
    request_body = UpdateSpecRequest,
    responses(
        (status = 200, description = "Updated spec", body = MeshResponse),
        (status = 400, description = "Invalid payload", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Spec not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn update_spec(
    Path(id): Path<String>,
    Json(req): Json<UpdateSpecRequest>,
) -> impl IntoResponse {
    let mut spec = match specs::get_spec(&id) {
        Ok(Some(s)) => s,
        Ok(None) => return err(StatusCode::NOT_FOUND, format!("spec '{id}' not found")),
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    if let Some(t) = req.title { spec.title = t; }
    if let Some(p) = req.pipeline { spec.pipeline = p; }
    if let Some(s) = req.status { spec.status = s; }
    if let Some(v) = req.vars { spec.vars = v; }
    if let Some(c) = req.criteria { spec.criteria = c; }
    if let Some(s) = req.scope { spec.scope = s; }
    if let Some(p) = req.priority { spec.priority = p; }
    spec.updated_at = Utc::now();

    if let Some(body) = req.body
        && let Err(e) = specs::save_body(&spec.body_path, &body)
    {
        return err(StatusCode::INTERNAL_SERVER_ERROR, format!("save body: {e}"));
    }
    match specs::upsert_spec(&spec) {
        Ok(()) => ok(spec_to_json(&spec)),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// DELETE /api/v1/runtime/specs/{id}
#[utoipa::path(
    operation_id = "deleteSpec",
    delete,
    path = "/api/v1/specs/{id}",
    params(("id" = String, Path, description = "Spec id")),
    responses(
        (status = 200, description = "Deleted", body = MeshResponse),
        (status = 404, description = "Spec not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn delete_spec(Path(id): Path<String>) -> impl IntoResponse {
    match specs::delete_spec(&id) {
        Ok(()) => ok(serde_json::json!({ "id": id })),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// POST /api/v1/runtime/specs/{id}/launch — create a job from the spec.
#[utoipa::path(
    operation_id = "launchSpec",
    post,
    path = "/api/v1/specs/{id}/launch",
    params(("id" = String, Path, description = "Spec id")),
    responses(
        (status = 200, description = "Job dispatched from spec", body = MeshResponse),
        (status = 400, description = "Invalid pipeline", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Spec not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn launch_spec(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let spec = match specs::get_spec(&id) {
        Ok(Some(s)) => s,
        Ok(None) => return err(StatusCode::NOT_FOUND, format!("spec '{id}' not found")),
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };

    let job_id = runtime::gen_ulid();
    let slug = format!("spec-{}", &spec.id);
    let branch = format!("job/{slug}");

    let baseline_sha = runtime::project::get_project_dir(None)
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        .and_then(|root| runtime::services::orchestrator::git_helpers::resolve_head_sha(&root));
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug,
        agent: "claude".into(),
        branch,
        spec_path: spec.body_path.clone(),
        source_spec_path: Some(spec.body_path.clone()),
        plan_id: None,
        model: None,
        provider: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: spec.pipeline.clone(),
        vars: if spec.vars.is_empty() { None } else { Some(spec.vars.clone()) },
        spec_content: std::fs::read_to_string(&spec.body_path).ok(),
        step_specs: None,
        headless: false,
        baseline_sha,
    };

    let payload = match runtime::services::orchestrator::pipeline_loader::ensure_pipeline(payload) {
        Ok(p) => p,
        Err(e) => return err(StatusCode::BAD_REQUEST, e),
    };

    let envelope = match EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
        .map(|e| e.with_actor_id("studio"))
    {
        Ok(e) => e,
        Err(e) => return err(StatusCode::BAD_REQUEST, e.to_string()),
    };

    if let Err(e) = state.persist_and_route(&envelope).await {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }

    // Mark spec as launched.
    let mut updated = spec;
    updated.status = "launched".into();
    updated.updated_at = Utc::now();
    let _ = specs::upsert_spec(&updated);

    ok(serde_json::json!({ "job_id": job_id, "spec_id": id }))
}
