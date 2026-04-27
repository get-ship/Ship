//! Specs REST API — CRUD over the `spec` table and pipeline launch.
//!
//! Routes are mounted under `/api/v1/runtime` in lib.rs. All real work
//! delegates through `SpecsService` (mr-02 trait).

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runtime::db::specs::Spec;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

use crate::access::RuntimeAccess;
use crate::rest_api::{ApiState, MeshResponse};
use crate::services::specs::service::{
    CreateArgs, Error as SpecsError, SpecsService, UpdateArgs,
};

type Resp = (StatusCode, Json<MeshResponse>);

fn ok(data: serde_json::Value) -> Resp {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

fn err(status: StatusCode, msg: impl Into<String>) -> Resp {
    (status, Json(MeshResponse { ok: false, data: serde_json::json!(msg.into()) }))
}

fn map_err(e: SpecsError) -> Resp {
    use SpecsError::*;
    match e {
        NotFound(m) => err(StatusCode::NOT_FOUND, m),
        Validation(m) => err(StatusCode::BAD_REQUEST, m),
        Io(m) => err(StatusCode::INTERNAL_SERVER_ERROR, m),
        Internal(m) => err(StatusCode::INTERNAL_SERVER_ERROR, m),
    }
}

fn specs(state: &ApiState) -> &dyn SpecsService {
    state.services().specs()
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
pub async fn list_specs(State(state): State<ApiState>) -> impl IntoResponse {
    match specs(&state).list() {
        Ok(list) => {
            let data: Vec<_> = list.iter().map(spec_to_json).collect();
            ok(serde_json::json!({ "specs": data }))
        }
        Err(e) => map_err(e),
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
pub async fn get_spec(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match specs(&state).get_with_body(&id) {
        Ok((s, body)) => {
            let mut v = spec_to_json(&s);
            v["body"] = serde_json::Value::String(body);
            ok(v)
        }
        Err(e) => map_err(e),
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
pub async fn create_spec(
    State(state): State<ApiState>,
    Json(req): Json<CreateSpecRequest>,
) -> impl IntoResponse {
    let args = CreateArgs {
        title: req.title,
        body: req.body,
        status: req.status,
        pipeline: req.pipeline,
        vars: req.vars,
        criteria: req.criteria,
        scope: req.scope,
        priority: req.priority,
    };
    match specs(&state).create(args) {
        Ok(spec) => ok(spec_to_json(&spec)),
        Err(e) => map_err(e),
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
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSpecRequest>,
) -> impl IntoResponse {
    let args = UpdateArgs {
        title: req.title,
        body: req.body,
        status: req.status,
        pipeline: req.pipeline,
        vars: req.vars,
        criteria: req.criteria,
        scope: req.scope,
        priority: req.priority,
    };
    match specs(&state).update(&id, args) {
        Ok(spec) => ok(spec_to_json(&spec)),
        Err(e) => map_err(e),
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
pub async fn delete_spec(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match specs(&state).delete(&id) {
        Ok(()) => ok(serde_json::json!({ "id": id })),
        Err(e) => map_err(e),
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
    match specs(&state).launch(&id).await {
        Ok(job_id) => ok(serde_json::json!({ "job_id": job_id, "spec_id": id })),
        Err(e) => map_err(e),
    }
}
