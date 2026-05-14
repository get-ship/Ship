//! REST API handlers for collections.
//!
//! CRUD + query over structured data. Views and agents consume this API.
//! Write operations require WriteWorkspace scope; reads require ReadWorkspace.
//!
//! All data work delegates through `CollectionsService` (mr-02 trait).
//! Handlers stay thin: scope gating, body shape parsing, JSON envelope,
//! status-code mapping.

use axum::{Extension, Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::access::RuntimeAccess;
use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::ApiState;
use ship_platform::services::collections::service::{
    CollectionsService, CreateItemArgs, Error as CollectionsError, UpdateItemArgs,
};
use ship_platform::collections::{query::QueryRequest, schema};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn err_owned(status: StatusCode, msg: String) -> axum::response::Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg }))).into_response()
}

fn map_err(e: CollectionsError) -> axum::response::Response {
    use CollectionsError::*;
    match e {
        NotFound(m) => err_owned(StatusCode::NOT_FOUND, m),
        Conflict(m) => err_owned(StatusCode::CONFLICT, m),
        Validation(m) => err_owned(StatusCode::BAD_REQUEST, m),
        Io(m) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, m),
        Internal(m) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, m),
    }
}

fn collections(state: &ApiState) -> &dyn CollectionsService {
    state.services().collections()
}

fn generate_item_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).ok();
    format!("{ts:x}-{:08x}", u32::from_le_bytes(buf))
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/v1/collections — list registered schemas.
#[utoipa::path(
    operation_id = "listCollections",
    get,
    path = "/api/v1/collections",
    responses(
        (status = 200, description = "List of registered collection schemas"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "collections"
)]
pub async fn list_collections(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    match collections(&state).list_user_schemas() {
        Ok(schemas) => {
            let summaries: Vec<_> = schemas.iter().map(|s| serde_json::json!({
                "id": s.id,
                "name": s.name,
                "field_count": s.fields.len(),
            })).collect();
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": summaries })))
                .into_response()
        }
        Err(e) => map_err(e),
    }
}

/// POST /api/v1/collections — create a new user-defined collection schema.
/// Body: full `CollectionSchema` JSON (id, name, fields).
/// Rejects built-in ids and duplicates (409).
pub async fn create_collection(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(new_schema): Json<schema::CollectionSchema>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    match collections(&state).create_schema(new_schema.clone()) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "ok": true, "data": new_schema })),
        )
            .into_response(),
        Err(e) => map_err(e),
    }
}

/// GET /api/v1/collections/{collection}/schema — get schema definition.
#[utoipa::path(
    operation_id = "getSchema",
    get,
    path = "/api/v1/collections/{collection}/schema",
    params(("collection" = String, Path, description = "Collection id")),
    responses(
        (status = 200, description = "Collection schema"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "Collection not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "collections"
)]
pub async fn get_schema(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    match collections(&state).get_schema(&collection) {
        Ok(s) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": s })))
            .into_response(),
        Err(e) => map_err(e),
    }
}

/// PUT /api/v1/collections/{collection}/schema — write the user-defined schema
/// to `.ship/collections/<collection>.jsonc`. Rejects writes to built-in
/// collections (specs, adrs, notes, feedback, annotations) — those are
/// owned by the runtime and shipped as code.
pub async fn update_schema(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection): Path<String>,
    Json(updated): Json<schema::CollectionSchema>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let payload = updated.clone();
    match collections(&state).update_schema(&collection, updated) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "data": payload })),
        )
            .into_response(),
        Err(e) => map_err(e),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "intro",
    "data": {
        "title": "Welcome",
        "body": "Ship runs agents on specs.",
        "tags": ["intro", "pinned"]
    }
}))]
pub struct CreateItemRequest {
    pub id: Option<String>,
    pub data: serde_json::Value,
}

/// POST /api/v1/collections/{collection}/items — create item.
#[utoipa::path(
    operation_id = "createItem",
    post,
    path = "/api/v1/collections/{collection}/items",
    params(("collection" = String, Path, description = "Collection id")),
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created"),
        (status = 400, description = "Validation error", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "Collection not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "collections"
)]
pub async fn create_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection): Path<String>,
    Json(req): Json<CreateItemRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let id = req.id.unwrap_or_else(generate_item_id);
    let args = CreateItemArgs {
        collection: collection.clone(),
        id,
        data: req.data,
        created_by: auth.agent_id.clone(),
    };
    match collections(&state).create_item(args).await {
        Ok(item) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "ok": true, "data": item })),
        )
            .into_response(),
        Err(e) => map_err(e),
    }
}

/// GET /api/v1/collections/{collection}/items/{item_id} — get item.
#[utoipa::path(
    operation_id = "getItem",
    get,
    path = "/api/v1/collections/{collection}/items/{item_id}",
    params(
        ("collection" = String, Path, description = "Collection id"),
        ("item_id" = String, Path, description = "Item id"),
    ),
    responses(
        (status = 200, description = "Item"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "Item not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "collections"
)]
pub async fn get_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path((collection, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    match collections(&state).get_item(&collection, &item_id).await {
        Ok(item) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": item })))
            .into_response(),
        Err(e) => map_err(e),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "data": { "title": "Welcome (v2)" }
}))]
pub struct UpdateItemRequest {
    pub data: serde_json::Value,
}

/// PUT /api/v1/collections/{collection}/items/{item_id} — update item (partial merge).
#[utoipa::path(
    operation_id = "updateItem",
    put,
    path = "/api/v1/collections/{collection}/items/{item_id}",
    params(
        ("collection" = String, Path, description = "Collection id"),
        ("item_id" = String, Path, description = "Item id"),
    ),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, description = "Item updated"),
        (status = 400, description = "Validation error", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "Item not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "collections"
)]
pub async fn update_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path((collection, item_id)): Path<(String, String)>,
    Json(req): Json<UpdateItemRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let args = UpdateItemArgs {
        collection,
        id: item_id,
        data: req.data,
    };
    match collections(&state).update_item(args).await {
        Ok(item) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": item })))
            .into_response(),
        Err(e) => map_err(e),
    }
}

/// DELETE /api/v1/collections/{collection}/items/{item_id} — delete item.
#[utoipa::path(
    operation_id = "deleteItem",
    delete,
    path = "/api/v1/collections/{collection}/items/{item_id}",
    params(
        ("collection" = String, Path, description = "Collection id"),
        ("item_id" = String, Path, description = "Item id"),
    ),
    responses(
        (status = 200, description = "Item deleted"),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "Item not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "collections"
)]
pub async fn delete_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path((collection, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    match collections(&state).delete_item(&collection, &item_id).await {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
            .into_response(),
        Ok(false) => err_owned(StatusCode::NOT_FOUND, format!("item '{item_id}' not found")),
        Err(e) => map_err(e),
    }
}

/// POST /api/v1/collections/{collection}/query — query items with filter/sort/limit.
#[utoipa::path(
    operation_id = "queryItems",
    post,
    path = "/api/v1/collections/{collection}/query",
    params(("collection" = String, Path, description = "Collection id")),
    request_body(content_type = "application/json", description = "Query request: filter, sort, limit, offset"),
    responses(
        (status = 200, description = "Matching items"),
        (status = 400, description = "Invalid query", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 404, description = "Collection not found", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "collections"
)]
pub async fn query_items(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection): Path<String>,
    Json(mut req): Json<QueryRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    if !ship_platform::collections::query::is_safe_identifier(&collection) {
        return (
            StatusCode::BAD_REQUEST,
            Json(crate::wire::ErrorResponse {
                error: "bad_request".into(),
                message: format!("invalid collection name `{collection}`"),
                details: None,
            }),
        ).into_response();
    }
    req.collection = collection.clone();
    match collections(&state).query_items(&collection, req).await {
        Ok(items) => {
            let count = items.len();
            (StatusCode::OK, Json(serde_json::json!({
                "ok": true,
                "data": items,
                "count": count,
            }))).into_response()
        }
        Err(e) => map_err(e),
    }
}
