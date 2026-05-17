//! Specs REST API — `/api/v1/specs/*`. CRUD plus pipeline launch.
//!
//! Specs are a first-party collection (`SPECS_ID`); the routes are
//! thin shims over `CollectionsService` plus on-disk body files.
//! `launch` delegates to the generic
//! `services::launcher::launch_collection_item`.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::access::RuntimeAccess;
use crate::routes::rest_api::{ApiState, MeshResponse};
use ship_platform::services::collections::service::{
    CollectionsService, CreateItemArgs, Error as CollectionsError, UpdateItemArgs,
};
use crate::services::launcher::{LaunchError, launch_collection_item};
use crate::services::specs::shape::{
    Spec, build_create_data, build_update_data, default_body_path, load_body, save_body,
    spec_from_item,
};

use ship_platform::collections::builtin::SPECS_ID;

type Resp = (StatusCode, Json<MeshResponse>);

fn ok(data: serde_json::Value) -> Resp {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

fn err(status: StatusCode, msg: impl Into<String>) -> Resp {
    (status, Json(MeshResponse { ok: false, data: serde_json::json!(msg.into()) }))
}

fn map_collections_err(e: CollectionsError) -> Resp {
    use CollectionsError::*;
    match e {
        NotFound(m) => err(StatusCode::NOT_FOUND, m),
        Validation(m) => err(StatusCode::BAD_REQUEST, m),
        Conflict(m) => err(StatusCode::CONFLICT, m),
        Io(m) | Internal(m) => err(StatusCode::INTERNAL_SERVER_ERROR, m),
    }
}

fn map_launch_err(e: LaunchError) -> Resp {
    use LaunchError::*;
    match e {
        NotFound(m) => err(StatusCode::NOT_FOUND, m),
        Validation(m) => err(StatusCode::BAD_REQUEST, m),
        Io(m) | Internal(m) => err(StatusCode::INTERNAL_SERVER_ERROR, m),
    }
}

fn collections(state: &ApiState) -> &dyn CollectionsService {
    state.services().collections()
}

fn spec_to_json(s: &Spec) -> serde_json::Value {
    serde_json::json!({
        "id": s.id,
        "title": s.title,
        "body_path": s.body_path,
        "status": s.status,
        "pipeline": s.pipeline,
        "agent": s.agent,
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
    pub agent: Option<String>,
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
    pub agent: Option<Option<String>>,
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

/// GET /api/v1/specs
#[utoipa::path(
    operation_id = "listSpecs",
    get,
    path = "/api/v1/specs",
    responses(
        (status = 200, description = "List of specs", body = MeshResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn list_specs(State(state): State<ApiState>) -> impl IntoResponse {
    let req = ship_platform::collections::query::QueryRequest {
        collection: SPECS_ID.to_string(),
        filter: Default::default(),
        sort: {
            let mut s = std::collections::BTreeMap::new();
            s.insert("updated_at".into(), -1);
            s
        },
        limit: 1000,
        offset: 0,
    };
    match collections(&state).query_items(SPECS_ID, req).await {
        Ok(items) => {
            let data: Vec<_> = items
                .into_iter()
                .map(|i| spec_to_json(&spec_from_item(i)))
                .collect();
            ok(serde_json::json!({ "specs": data }))
        }
        Err(e) => map_collections_err(e),
    }
}

/// GET /api/v1/specs/{id}
#[utoipa::path(
    operation_id = "getSpec",
    get,
    path = "/api/v1/specs/{id}",
    params(("id" = String, Path, description = "Spec id")),
    responses(
        (status = 200, description = "Spec detail with body", body = MeshResponse),
        (status = 404, description = "Spec not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn get_spec(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match collections(&state).get_item(SPECS_ID, &id).await {
        Ok(item) => {
            let spec = spec_from_item(item);
            let body = load_body(&spec.body_path).unwrap_or_default();
            let mut v = spec_to_json(&spec);
            v["body"] = serde_json::Value::String(body);
            ok(v)
        }
        Err(e) => map_collections_err(e),
    }
}

/// POST /api/v1/specs
#[utoipa::path(
    operation_id = "createSpec",
    post,
    path = "/api/v1/specs",
    request_body = CreateSpecRequest,
    responses(
        (status = 200, description = "Created spec", body = MeshResponse),
        (status = 400, description = "Invalid payload", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn create_spec(
    State(state): State<ApiState>,
    Json(req): Json<CreateSpecRequest>,
) -> impl IntoResponse {
    if req.title.trim().is_empty() {
        return err(StatusCode::BAD_REQUEST, "title is required");
    }
    let id = ship_substrate::gen_ulid();
    let body_path = default_body_path(&id);
    if let Err(e) = save_body(&body_path, &req.body) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, format!("save body: {e}"));
    }
    let data = build_create_data(
        &req.title,
        &body_path,
        req.status.as_deref(),
        req.pipeline.as_deref(),
        req.agent.as_deref(),
        &req.vars,
        &req.criteria,
        &req.scope,
        req.priority,
    );
    let args = CreateItemArgs {
        collection: SPECS_ID.to_string(),
        id,
        data,
        created_by: None,
    };
    match collections(&state).create_item(args).await {
        Ok(item) => ok(spec_to_json(&spec_from_item(item))),
        Err(e) => map_collections_err(e),
    }
}

/// PUT /api/v1/specs/{id}
#[utoipa::path(
    operation_id = "updateSpec",
    put,
    path = "/api/v1/specs/{id}",
    params(("id" = String, Path, description = "Spec id")),
    request_body = UpdateSpecRequest,
    responses(
        (status = 200, description = "Updated spec", body = MeshResponse),
        (status = 400, description = "Invalid payload", body = crate::wire::ErrorResponse),
        (status = 404, description = "Spec not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn update_spec(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSpecRequest>,
) -> impl IntoResponse {
    // Body writes need the existing item's body_path. Fetch it first; an
    // update against a nonexistent id should 404 before disk I/O anyway.
    let body_path = if req.body.is_some() {
        match collections(&state).get_item(SPECS_ID, &id).await {
            Ok(item) => spec_from_item(item).body_path,
            Err(e) => return map_collections_err(e),
        }
    } else {
        String::new()
    };
    if let Some(body) = &req.body {
        let path = if body_path.is_empty() {
            default_body_path(&id)
        } else {
            body_path.clone()
        };
        if let Err(e) = save_body(&path, body) {
            return err(StatusCode::INTERNAL_SERVER_ERROR, format!("save body: {e}"));
        }
    }
    let pipeline = req.pipeline.as_ref().map(|opt| opt.as_deref());
    let agent = req.agent.as_ref().map(|opt| opt.as_deref());
    let updates = build_update_data(
        req.title.as_deref(),
        req.status.as_deref(),
        pipeline,
        agent,
        req.vars.as_ref(),
        req.criteria.as_deref(),
        req.scope.as_deref(),
        req.priority,
    );
    let args = UpdateItemArgs {
        collection: SPECS_ID.to_string(),
        id,
        data: updates,
    };
    match collections(&state).update_item(args).await {
        Ok(item) => ok(spec_to_json(&spec_from_item(item))),
        Err(e) => map_collections_err(e),
    }
}

/// DELETE /api/v1/specs/{id}
#[utoipa::path(
    operation_id = "deleteSpec",
    delete,
    path = "/api/v1/specs/{id}",
    params(("id" = String, Path, description = "Spec id")),
    responses(
        (status = 200, description = "Deleted", body = MeshResponse),
        (status = 404, description = "Spec not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn delete_spec(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match collections(&state).delete_item(SPECS_ID, &id).await {
        Ok(true) => ok(serde_json::json!({ "id": id })),
        Ok(false) => err(StatusCode::NOT_FOUND, format!("spec '{id}' not found")),
        Err(e) => map_collections_err(e),
    }
}

/// POST /api/v1/specs/{id}/launch — create a job from the spec.
#[utoipa::path(
    operation_id = "launchSpec",
    post,
    path = "/api/v1/specs/{id}/launch",
    params(("id" = String, Path, description = "Spec id")),
    responses(
        (status = 200, description = "Job dispatched from spec", body = MeshResponse),
        (status = 400, description = "Invalid pipeline", body = crate::wire::ErrorResponse),
        (status = 404, description = "Spec not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "specs"
)]
pub async fn launch_spec(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let kernel = state.kernel().clone();
    match launch_collection_item(collections(&state), &kernel, SPECS_ID, &id).await {
        Ok(job_id) => ok(serde_json::json!({ "job_id": job_id, "spec_id": id })),
        Err(e) => map_launch_err(e),
    }
}

