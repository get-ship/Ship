//! REST API handlers for collections.
//!
//! CRUD + query over structured data. Views and agents consume this API.
//! Write operations require WriteWorkspace scope; reads require ReadWorkspace.

use axum::{Extension, Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::ApiState;
use runtime::collections::{builtin, query::QueryRequest, schema, service, store};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn err_owned(status: StatusCode, msg: String) -> axum::response::Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg }))).into_response()
}

/// Get the shared DataService, ensuring collection table exists.
async fn get_svc_and_schema<'a>(
    state: &'a ApiState,
    collection: &str,
) -> Result<
    (tokio::sync::MutexGuard<'a, Option<runtime::data::DataService>>, schema::CollectionSchema),
    axum::response::Response,
> {
    let schema = load_schema(state.ship_dir(), collection)
        .map_err(|e| err_owned(StatusCode::NOT_FOUND, e.to_string()))?;

    let mut guard = state.data_service().await
        .map_err(|e| err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Ensure table exists for this schema.
    {
        let svc = guard.as_mut().unwrap();
        store::ensure_collection_table(svc, &schema)
            .map_err(|e| err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok((guard, schema))
}

/// Load all schemas: built-in (user-visible) + user-defined from .ship/collections/*.jsonc.
/// Merges global (~/.ship/) and project-local schemas; project-local override global on id clash.
/// System schemas (feedback, annotations) are excluded from user-facing listing.
fn load_all_schemas(ship_dir: &std::path::Path) -> anyhow::Result<Vec<schema::CollectionSchema>> {
    let project_dir = runtime::project::get_project_dir(None).ok();
    let user_schemas = schema::load_schemas_merged(
        ship_dir,
        project_dir.as_deref(),
    )?;
    let mut schemas = builtin::user_schemas();
    schemas.extend(user_schemas);
    Ok(schemas)
}

/// Load ALL schemas including system-internal ones — used internally for
/// lookup when we need to operate on any collection (including system ones).
fn load_any_schema(
    ship_dir: &std::path::Path,
    collection: &str,
) -> anyhow::Result<schema::CollectionSchema> {
    let project_dir = runtime::project::get_project_dir(None).ok();
    let user_schemas = schema::load_schemas_merged(ship_dir, project_dir.as_deref())?;
    let mut schemas = builtin::all_schemas();
    schemas.extend(user_schemas);
    schemas
        .into_iter()
        .find(|s| s.id == collection)
        .ok_or_else(|| anyhow::anyhow!("collection '{}' not found", collection))
}

fn load_schema(
    ship_dir: &std::path::Path,
    collection: &str,
) -> anyhow::Result<schema::CollectionSchema> {
    load_any_schema(ship_dir, collection)
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

/// GET /api/collections — list registered schemas.
#[utoipa::path(
    operation_id = "listCollections",
    get,
    path = "/api/collections",
    responses(
        (status = 200, description = "List of registered collection schemas"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
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
    match load_all_schemas(state.ship_dir()) {
        Ok(schemas) => {
            let summaries: Vec<_> = schemas.iter().map(|s| serde_json::json!({
                "id": s.id,
                "name": s.name,
                "field_count": s.fields.len(),
            })).collect();
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": summaries })))
                .into_response()
        }
        Err(e) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// POST /api/collections — create a new user-defined collection schema.
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
    // Reject if it conflicts with any known built-in id (runtime-owned schemas).
    // specs and adrs are user-editable .ship/collections/ files, not runtime built-ins.
    let known_builtin_ids = [
        builtin::NOTES_ID,
        builtin::FEEDBACK_ID,
        builtin::ANNOTATIONS_ID,
    ];
    if known_builtin_ids.contains(&new_schema.id.as_str()) {
        return err_owned(
            StatusCode::CONFLICT,
            format!("'{}' is a reserved built-in collection id", new_schema.id),
        );
    }
    // Validate snake_case id (basic check: non-empty, alphanumeric + underscores).
    if new_schema.id.is_empty()
        || !new_schema.id.chars().all(|c| c.is_alphanumeric() || c == '_')
    {
        return err_owned(
            StatusCode::BAD_REQUEST,
            format!("collection id '{}' must be non-empty alphanumeric/underscore", new_schema.id),
        );
    }
    let schemas_dir = state.ship_dir().join("collections");
    if let Err(e) = std::fs::create_dir_all(&schemas_dir) {
        return err_owned(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("cannot create {}: {e}", schemas_dir.display()),
        );
    }
    let path = schemas_dir.join(format!("{}.jsonc", new_schema.id));
    if path.exists() {
        return err_owned(
            StatusCode::CONFLICT,
            format!("collection '{}' already exists", new_schema.id),
        );
    }
    let json = match serde_json::to_string_pretty(&new_schema) {
        Ok(s) => s,
        Err(e) => return err_owned(StatusCode::BAD_REQUEST, format!("serialize error: {e}")),
    };
    let tmp = path.with_extension("jsonc.tmp");
    if let Err(e) = std::fs::write(&tmp, &json) {
        return err_owned(StatusCode::INTERNAL_SERVER_ERROR, format!("write failed: {e}"));
    }
    if let Err(e) = std::fs::rename(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        return err_owned(StatusCode::INTERNAL_SERVER_ERROR, format!("rename failed: {e}"));
    }
    (StatusCode::CREATED, Json(serde_json::json!({ "ok": true, "data": new_schema })))
        .into_response()
}

/// GET /api/collections/{collection}/schema — get schema definition.
#[utoipa::path(
    operation_id = "getSchema",
    get,
    path = "/api/collections/{collection}/schema",
    params(("collection" = String, Path, description = "Collection id")),
    responses(
        (status = 200, description = "Collection schema"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Collection not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
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
    match load_schema(state.ship_dir(), &collection) {
        Ok(s) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": s })))
            .into_response(),
        Err(e) => err_owned(StatusCode::NOT_FOUND, e.to_string()),
    }
}

/// PUT /api/collections/{collection}/schema — write the user-defined schema
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
    // Reject schema edits for runtime built-ins — they live in Rust, not on disk.
    // specs and adrs are user-editable .ship/collections/ files, so writes to them are allowed.
    {
        let known_builtin_ids = [
            builtin::NOTES_ID,
            builtin::FEEDBACK_ID,
            builtin::ANNOTATIONS_ID,
        ];
        if known_builtin_ids.contains(&collection.as_str()) {
            return err_owned(
                StatusCode::CONFLICT,
                format!(
                    "'{collection}' is a built-in collection; its schema is defined in the runtime and cannot be edited through the API"
                ),
            );
        }
    }
    // Path id must match body id to prevent accidental renames via URL drift.
    if updated.id != collection {
        return err_owned(
            StatusCode::BAD_REQUEST,
            format!(
                "path collection '{collection}' does not match body id '{}' — rename is unsupported",
                updated.id
            ),
        );
    }
    // Validate, serialize, write atomically.
    let schemas_dir = state.ship_dir().join("collections");
    if let Err(e) = std::fs::create_dir_all(&schemas_dir) {
        return err_owned(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("cannot create {}: {e}", schemas_dir.display()),
        );
    }
    let path = schemas_dir.join(format!("{collection}.jsonc"));
    let json = match serde_json::to_string_pretty(&updated) {
        Ok(s) => s,
        Err(e) => {
            return err_owned(
                StatusCode::BAD_REQUEST,
                format!("could not serialize schema: {e}"),
            )
        }
    };
    // Write via a sibling temp file + rename so a partial write can't leave
    // the .jsonc in an invalid state mid-edit.
    let tmp = path.with_extension("jsonc.tmp");
    if let Err(e) = std::fs::write(&tmp, &json) {
        return err_owned(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("write failed: {e}"),
        );
    }
    if let Err(e) = std::fs::rename(&tmp, &path) {
        let _ = std::fs::remove_file(&tmp);
        return err_owned(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("rename failed: {e}"),
        );
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true, "data": updated })),
    )
        .into_response()
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

/// POST /api/collections/{collection}/items — create item.
#[utoipa::path(
    operation_id = "createItem",
    post,
    path = "/api/collections/{collection}/items",
    params(("collection" = String, Path, description = "Collection id")),
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created"),
        (status = 400, description = "Validation error", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Collection not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
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
    let (mut guard, schema) = match get_svc_and_schema(&state, &collection).await {
        Ok(pair) => pair,
        Err(resp) => return resp,
    };
    let svc = guard.as_mut().unwrap();

    let id = req.id.unwrap_or_else(generate_item_id);
    let created_by = auth.agent_id.as_deref();

    match service::create_item(svc, state.kernel(), &schema, &id, &req.data, created_by).await {
        Ok(item) => (StatusCode::CREATED, Json(serde_json::json!({ "ok": true, "data": item })))
            .into_response(),
        Err(e) => {
            let status = if e.to_string().contains("validation") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            err_owned(status, e.to_string())
        }
    }
}

/// GET /api/collections/{collection}/items/{item_id} — get item.
#[utoipa::path(
    operation_id = "getItem",
    get,
    path = "/api/collections/{collection}/items/{item_id}",
    params(
        ("collection" = String, Path, description = "Collection id"),
        ("item_id" = String, Path, description = "Item id"),
    ),
    responses(
        (status = 200, description = "Item"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Item not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
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
    let (mut guard, schema) = match get_svc_and_schema(&state, &collection).await {
        Ok(pair) => pair,
        Err(resp) => return resp,
    };
    let svc = guard.as_mut().unwrap();

    match service::get_item(svc, &schema, &item_id) {
        Ok(Some(item)) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": item })))
            .into_response(),
        Ok(None) => err_owned(StatusCode::NOT_FOUND, format!("item '{item_id}' not found")),
        Err(e) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "data": { "title": "Welcome (v2)" }
}))]
pub struct UpdateItemRequest {
    pub data: serde_json::Value,
}

/// PUT /api/collections/{collection}/items/{item_id} — update item (partial merge).
#[utoipa::path(
    operation_id = "updateItem",
    put,
    path = "/api/collections/{collection}/items/{item_id}",
    params(
        ("collection" = String, Path, description = "Collection id"),
        ("item_id" = String, Path, description = "Item id"),
    ),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, description = "Item updated"),
        (status = 400, description = "Validation error", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Item not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
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
    let (mut guard, schema) = match get_svc_and_schema(&state, &collection).await {
        Ok(pair) => pair,
        Err(resp) => return resp,
    };
    let svc = guard.as_mut().unwrap();

    match service::update_item(svc, state.kernel(), &schema, &item_id, &req.data).await {
        Ok(item) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": item })))
            .into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("validation") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            err_owned(status, e.to_string())
        }
    }
}

/// DELETE /api/collections/{collection}/items/{item_id} — delete item.
#[utoipa::path(
    operation_id = "deleteItem",
    delete,
    path = "/api/collections/{collection}/items/{item_id}",
    params(
        ("collection" = String, Path, description = "Collection id"),
        ("item_id" = String, Path, description = "Item id"),
    ),
    responses(
        (status = 200, description = "Item deleted"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Item not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
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
    let mut guard = match state.data_service().await {
        Ok(g) => g,
        Err(e) => return err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    };
    let svc = guard.as_mut().unwrap();

    match service::delete_item(svc, state.kernel(), &collection, &item_id).await {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
            .into_response(),
        Ok(false) => err_owned(StatusCode::NOT_FOUND, format!("item '{item_id}' not found")),
        Err(e) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// POST /api/collections/{collection}/query — query items with filter/sort/limit.
#[utoipa::path(
    operation_id = "queryItems",
    post,
    path = "/api/collections/{collection}/query",
    params(("collection" = String, Path, description = "Collection id")),
    request_body(content_type = "application/json", description = "Query request: filter, sort, limit, offset"),
    responses(
        (status = 200, description = "Matching items"),
        (status = 400, description = "Invalid query", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Collection not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
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
    req.collection = collection.clone();

    let (mut guard, schema) = match get_svc_and_schema(&state, &collection).await {
        Ok(pair) => pair,
        Err(resp) => return resp,
    };
    let svc = guard.as_mut().unwrap();

    match service::query_items(svc, &schema, &req) {
        Ok(items) => {
            let count = items.len();
            (StatusCode::OK, Json(serde_json::json!({
                "ok": true,
                "data": items,
                "count": count,
            }))).into_response()
        }
        Err(e) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
