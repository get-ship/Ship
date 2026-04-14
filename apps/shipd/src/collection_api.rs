//! REST API handlers for collections.
//!
//! CRUD + query over structured data. Views and agents consume this API.
//! Write operations require WriteWorkspace scope; reads require ReadWorkspace.

use axum::{Extension, Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::ApiState;
use runtime::collections::{query::QueryRequest, schema, service, store};
use runtime::data::DataService;

/// GET /api/collections — list registered schemas.
pub async fn list_collections(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    match schema::load_schemas(state.ship_dir()) {
        Ok(schemas) => {
            let summaries: Vec<_> = schemas.iter().map(|s| serde_json::json!({
                "id": s.id,
                "name": s.name,
                "field_count": s.fields.len(),
            })).collect();
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": summaries })))
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    }
}

/// GET /api/collections/{collection}/schema — get schema definition.
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
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    }
}

#[derive(Deserialize)]
pub struct CreateItemRequest {
    pub id: Option<String>,
    pub data: serde_json::Value,
}

/// POST /api/collections/{collection}/items — create item.
pub async fn create_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection): Path<String>,
    Json(req): Json<CreateItemRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let schema = match load_schema(state.ship_dir(), &collection) {
        Ok(s) => s,
        Err(e) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    let id = req.id.unwrap_or_else(generate_item_id);
    let created_by = auth.agent_id.as_deref();

    let mut conn = match store::open_docs_db(state.ship_dir()) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    match service::create_item(&mut conn, state.kernel(), &schema, &id, &req.data, created_by).await {
        Ok(item) => (StatusCode::CREATED, Json(serde_json::json!({ "ok": true, "data": item })))
            .into_response(),
        Err(e) => {
            let status = if e.to_string().contains("validation") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(serde_json::json!({ "ok": false, "error": e.to_string() })))
                .into_response()
        }
    }
}

/// GET /api/collections/{collection}/items/{item_id} — get item.
pub async fn get_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path((collection, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    let schema = match load_schema(state.ship_dir(), &collection) {
        Ok(s) => s,
        Err(e) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };
    let mut conn = match store::open_docs_db(state.ship_dir()) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    match service::get_item(&mut conn, &schema, &item_id) {
        Ok(Some(item)) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": item })))
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": format!("item '{item_id}' not found") })),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateItemRequest {
    pub data: serde_json::Value,
}

/// PUT /api/collections/{collection}/items/{item_id} — update item (partial merge).
pub async fn update_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path((collection, item_id)): Path<(String, String)>,
    Json(req): Json<UpdateItemRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let schema = match load_schema(state.ship_dir(), &collection) {
        Ok(s) => s,
        Err(e) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    let mut conn = match store::open_docs_db(state.ship_dir()) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    match service::update_item(&mut conn, state.kernel(), &schema, &item_id, &req.data).await {
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
            (status, Json(serde_json::json!({ "ok": false, "error": e.to_string() })))
                .into_response()
        }
    }
}

/// DELETE /api/collections/{collection}/items/{item_id} — delete item.
pub async fn delete_item(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path((collection, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let mut conn = match store::open_docs_db(state.ship_dir()) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    match service::delete_item(&mut conn, state.kernel(), &collection, &item_id).await {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": format!("item '{item_id}' not found") })),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    }
}

/// POST /api/collections/{collection}/query — query items with filter/sort/limit.
pub async fn query_items(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(collection): Path<String>,
    Json(mut req): Json<QueryRequest>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    // Override collection from path (ignore any collection field in body).
    req.collection = collection.clone();

    let col_schema = match load_schema(state.ship_dir(), &collection) {
        Ok(s) => s,
        Err(e) => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    let mut conn = match store::open_docs_db(state.ship_dir()) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    };

    match service::query_items(&mut conn, &col_schema, &req) {
        Ok(items) => (StatusCode::OK, Json(serde_json::json!({
            "ok": true,
            "data": items,
            "count": items.len(),
        }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        ).into_response(),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

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

fn load_schema(
    ship_dir: &std::path::Path,
    collection: &str,
) -> anyhow::Result<runtime::collections::schema::CollectionSchema> {
    let schemas = schema::load_schemas(ship_dir)?;
    schemas
        .into_iter()
        .find(|s| s.id == collection)
        .ok_or_else(|| anyhow::anyhow!("collection '{}' not found", collection))
}
