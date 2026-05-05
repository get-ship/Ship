//! REST API handlers for the three-scope artifacts platform.
//!
//! Routes:
//!   POST   /api/v1/artifacts           — upload (scope + optional run_id)
//!   GET    /api/v1/artifacts            — list with filters
//!   GET    /api/v1/artifacts/:id        — download bytes
//!   HEAD   /api/v1/artifacts/:id        — metadata headers only
//!   GET    /api/v1/artifacts/:id/metadata — JSON metadata
//!   PATCH  /api/v1/artifacts/:id        — update name / owner_actor / expires_at
//!   DELETE /api/v1/artifacts/:id        — soft delete
//!   POST   /api/v1/artifacts/:id/promote — promote to target scope
//!
//! All data work delegates through `ArtifactsService` (mr-02 trait).
//! Multipart parsing, scope gating, and content-disposition headers stay
//! here — those are HTTP transport concerns.

use axum::{
    Extension, Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;

use crate::access::RuntimeAccess;
use crate::auth::{AuthContext, Scope};
use crate::rest_api::ApiState;
use crate::auth::scopes::{can_delete_artifacts, can_read_artifacts, can_write_artifacts, require_scope};
use crate::services::artifacts::service::{
    ArtifactsService, Error as ArtifactsError, PatchArgs, StoreArgs,
};
use ship_platform::artifacts::{ARTIFACTS_COLLECTION_ID, ArtifactScope};
use ship_platform::collections::query::QueryRequest;

/// Max upload: 50 MB.
pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg }))).into_response()
}

fn err_owned(status: StatusCode, msg: String) -> Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg }))).into_response()
}

fn map_err(e: ArtifactsError) -> Response {
    use ArtifactsError::*;
    match e {
        NotFound(m) => err_owned(StatusCode::NOT_FOUND, m),
        Validation(m) => err_owned(StatusCode::BAD_REQUEST, m),
        Io(m) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, m),
        Internal(m) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, m),
    }
}

fn artifacts(state: &ApiState) -> &dyn ArtifactsService {
    state.services().artifacts()
}

fn parse_scope(scope_str: &str, run_id: Option<&str>, user_id: Option<&str>, project_slug: &str) -> Result<ArtifactScope, String> {
    match scope_str {
        "run" => {
            let rid = run_id.ok_or("run_id required for scope=run")?;
            Ok(ArtifactScope::Run { project_id: project_slug.to_string(), run_id: rid.to_string() })
        }
        "project" => Ok(ArtifactScope::Project { project_id: project_slug.to_string() }),
        "user" => {
            let uid = user_id.ok_or("user_id required for scope=user")?;
            Ok(ArtifactScope::User { user_id: uid.to_string() })
        }
        other => Err(format!("invalid scope '{other}'; must be run|project|user")),
    }
}

// ── POST /api/v1/artifacts ───────────────────────────────────────────────────────

#[derive(Deserialize, Default, utoipa::IntoParams)]
pub struct UploadQuery {
    /// Artifact scope: `run`, `project`, or `user`.
    pub scope: Option<String>,
    /// Required when `scope=run`.
    pub run_id: Option<String>,
    /// Required when `scope=user`.
    pub user_id: Option<String>,
}

#[utoipa::path(
    operation_id = "uploadArtifact",
    post,
    path = "/api/v1/artifacts",
    params(UploadQuery),
    request_body(content_type = "multipart/form-data", description = "File upload with optional collection_id, item_id, expires_at fields"),
    responses(
        (status = 201, description = "Artifact uploaded"),
        (status = 400, description = "Invalid scope or multipart payload", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn upload_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Query(q): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Response {
    let scope_str = q.scope.as_deref().unwrap_or("project");
    if !can_write_artifacts(&auth, scope_str) {
        return err(StatusCode::FORBIDDEN, "insufficient scope for artifact write");
    }

    let project_slug = state.project_slug().unwrap_or_default();
    let scope = match parse_scope(scope_str, q.run_id.as_deref(), q.user_id.as_deref(), &project_slug) {
        Ok(s) => s,
        Err(e) => return err_owned(StatusCode::BAD_REQUEST, e),
    };

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut file_mime: Option<String> = None;
    let mut collection_id: Option<String> = None;
    let mut item_id: Option<String> = None;
    let mut expires_at: Option<String> = None;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return err_owned(StatusCode::BAD_REQUEST, format!("multipart error: {e}")),
        };
        let field_name = field.name().map(str::to_string).unwrap_or_default();
        match field_name.as_str() {
            "file" => {
                let name = field.file_name().map(str::to_string);
                let mime = field.content_type().map(str::to_string);
                let bytes = match field.bytes().await {
                    Ok(b) => b,
                    Err(e) => return err_owned(StatusCode::BAD_REQUEST, format!("read error: {e}")),
                };
                if bytes.len() > MAX_UPLOAD_BYTES {
                    return err(StatusCode::PAYLOAD_TOO_LARGE, "file exceeds size limit");
                }
                file_bytes = Some(bytes.to_vec());
                file_name = name;
                file_mime = mime;
            }
            "collection_id" => { collection_id = field.text().await.ok().filter(|s| !s.is_empty()); }
            "item_id" => { item_id = field.text().await.ok().filter(|s| !s.is_empty()); }
            "expires_at" => { expires_at = field.text().await.ok().filter(|s| !s.is_empty()); }
            _ => { let _ = field.bytes().await; }
        }
    }

    let bytes = match file_bytes {
        Some(b) => b,
        None => return err(StatusCode::BAD_REQUEST, "missing 'file' field"),
    };
    let name = file_name.unwrap_or_else(|| "unnamed".to_string());
    let mime = file_mime
        .or_else(|| mime_from_name(&name))
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // TTL expiry for run-scope when not explicitly provided.
    let computed_expires = if expires_at.is_none() {
        if let ArtifactScope::Run { .. } = &scope {
            let ttl_hours = 24u64; // default; TODO: read from config
            let exp = chrono::Utc::now() + chrono::Duration::hours(ttl_hours as i64);
            Some(exp.to_rfc3339())
        } else {
            None
        }
    } else {
        expires_at
    };

    let args = StoreArgs {
        name,
        mime,
        bytes,
        scope,
        collection_id,
        item_id,
        owner_actor: auth.agent_id.clone(),
        expires_at: computed_expires,
    };
    match artifacts(&state).store(args).await {
        Ok(record) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "ok": true, "data": record })),
        )
            .into_response(),
        Err(e) => map_err(e),
    }
}

// ── GET /api/v1/artifacts/:id ────────────────────────────────────────────────────

#[utoipa::path(
    operation_id = "downloadArtifact",
    get,
    path = "/api/v1/artifacts/{id}",
    params(("id" = String, Path, description = "Artifact id")),
    responses(
        (status = 200, description = "Artifact bytes stream", content_type = "application/octet-stream"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Artifact not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn download_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    // Fetch metadata first to check scope.
    let record = match artifacts(&state).get_record(&id).await {
        Ok(r) => r,
        Err(e) => return map_err(e),
    };
    if !can_read_artifacts(&auth, &record.scope) {
        return err(StatusCode::FORBIDDEN, "insufficient scope for artifact read");
    }
    match artifacts(&state).get(&id).await {
        Ok((rec, bytes)) => {
            let disposition = format!("attachment; filename=\"{}\"", sanitize_filename(&rec.name));
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, rec.mime.as_str())
                .header(header::CONTENT_DISPOSITION, disposition)
                .header(header::CONTENT_LENGTH, bytes.len())
                .header("x-artifact-scope", rec.scope.as_str())
                .header("x-content-hash", rec.content_hash.as_deref().unwrap_or(""))
                .body(Body::from(bytes))
                .unwrap_or_else(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "response build failed"))
        }
        Err(e) => map_err(e),
    }
}

// ── HEAD /api/v1/artifacts/:id ───────────────────────────────────────────────────

#[utoipa::path(
    operation_id = "headArtifact",
    head,
    path = "/api/v1/artifacts/{id}",
    params(("id" = String, Path, description = "Artifact id")),
    responses(
        (status = 200, description = "Metadata-only response; body empty, headers carry size/mime/scope/hash"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Artifact not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn head_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    match artifacts(&state).get_record(&id).await {
        Ok(rec) => {
            if !can_read_artifacts(&auth, &rec.scope) {
                return err(StatusCode::FORBIDDEN, "insufficient scope");
            }
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, rec.mime.as_str())
                .header("x-artifact-scope", rec.scope.as_str())
                .header("x-artifact-size", rec.size.to_string())
                .header("x-content-hash", rec.content_hash.as_deref().unwrap_or(""))
                .body(Body::empty())
                .unwrap_or_else(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "response build failed"))
        }
        Err(e) => map_err(e),
    }
}

// ── GET /api/v1/artifacts/:id/metadata ──────────────────────────────────────────

#[utoipa::path(
    operation_id = "artifactMetadata",
    get,
    path = "/api/v1/artifacts/{id}/metadata",
    params(("id" = String, Path, description = "Artifact id")),
    responses(
        (status = 200, description = "Artifact metadata as JSON"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Artifact not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn artifact_metadata(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    match artifacts(&state).get_record(&id).await {
        Ok(rec) => {
            if !can_read_artifacts(&auth, &rec.scope) {
                return err(StatusCode::FORBIDDEN, "insufficient scope");
            }
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": rec }))).into_response()
        }
        Err(e) => map_err(e),
    }
}

// ── PATCH /api/v1/artifacts/:id ──────────────────────────────────────────────────

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "name": "README.md",
    "owner_actor": "agent-writer",
    "expires_at": "2026-05-22T00:00:00Z"
}))]
pub struct PatchBody {
    pub name: Option<String>,
    pub owner_actor: Option<String>,
    pub expires_at: Option<String>,
}

#[utoipa::path(
    operation_id = "patchArtifact",
    patch,
    path = "/api/v1/artifacts/{id}",
    params(("id" = String, Path, description = "Artifact id")),
    request_body = PatchBody,
    responses(
        (status = 200, description = "Artifact metadata patched"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Artifact not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn patch_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<PatchBody>,
) -> Response {
    let record = match artifacts(&state).get_record(&id).await {
        Ok(r) => r,
        Err(e) => return map_err(e),
    };
    if !can_write_artifacts(&auth, &record.scope) {
        return err(StatusCode::FORBIDDEN, "insufficient scope for artifact write");
    }
    let args = PatchArgs {
        name: body.name,
        owner_actor: body.owner_actor,
        expires_at: body.expires_at,
    };
    match artifacts(&state).patch(&id, args).await {
        Ok(rec) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": rec }))).into_response(),
        Err(e) => map_err(e),
    }
}

// ── DELETE /api/v1/artifacts/:id ─────────────────────────────────────────────────

#[utoipa::path(
    operation_id = "deleteArtifact",
    delete,
    path = "/api/v1/artifacts/{id}",
    params(("id" = String, Path, description = "Artifact id")),
    responses(
        (status = 200, description = "Artifact soft-deleted"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Artifact not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn delete_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    let record = match artifacts(&state).get_record(&id).await {
        Ok(r) => r,
        Err(e) => return map_err(e),
    };
    if !can_delete_artifacts(&auth, &record.scope) {
        return err(StatusCode::FORBIDDEN, "insufficient scope for artifact delete");
    }
    match artifacts(&state).delete(&id).await {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Ok(false) => err_owned(StatusCode::NOT_FOUND, format!("artifact '{id}' not found")),
        Err(e) => map_err(e),
    }
}

// ── POST /api/v1/artifacts/:id/promote ──────────────────────────────────────────

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "target_scope": "project",
    "run_id": "01J3Z5ZQXAM3E4T7W",
    "user_id": "me"
}))]
pub struct PromoteBody {
    pub target_scope: String,
    pub run_id: Option<String>,
    pub user_id: Option<String>,
}

#[utoipa::path(
    operation_id = "promoteArtifact",
    post,
    path = "/api/v1/artifacts/{id}/promote",
    params(("id" = String, Path, description = "Artifact id")),
    request_body = PromoteBody,
    responses(
        (status = 200, description = "Artifact promoted to target scope"),
        (status = 400, description = "Invalid target scope", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Artifact not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn promote_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<PromoteBody>,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::PromoteArtifact) {
        return e.into_response();
    }
    let project_slug = state.project_slug().unwrap_or_default();
    let target = match parse_scope(&body.target_scope, body.run_id.as_deref(), body.user_id.as_deref(), &project_slug) {
        Ok(s) => s,
        Err(e) => return err_owned(StatusCode::BAD_REQUEST, e),
    };
    match artifacts(&state).promote(&id, target).await {
        Ok(rec) => (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": rec }))).into_response(),
        Err(e) => map_err(e),
    }
}

// ── GET /api/v1/artifacts ────────────────────────────────────────────────────────

#[derive(Deserialize, Default, utoipa::IntoParams)]
pub struct ListQuery {
    pub scope: Option<String>,
    pub mime: Option<String>,
    pub collection_id: Option<String>,
    pub item_id: Option<String>,
    pub content_hash: Option<String>,
    pub owner_actor: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort: Option<String>,
}

#[utoipa::path(
    operation_id = "listArtifacts",
    get,
    path = "/api/v1/artifacts",
    params(ListQuery),
    responses(
        (status = 200, description = "Filtered, paginated list of artifacts"),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "artifacts"
)]
pub async fn list_artifacts(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Query(q): Query<ListQuery>,
) -> Response {
    let scope_str = q.scope.as_deref().unwrap_or("project");
    if !can_read_artifacts(&auth, scope_str) {
        return err(StatusCode::FORBIDDEN, "insufficient scope for artifact list");
    }
    let mut filter: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    filter.insert("scope".into(), serde_json::Value::String(scope_str.to_string()));
    if let Some(m) = q.mime { filter.insert("mime".into(), serde_json::Value::String(m)); }
    if let Some(c) = q.collection_id { filter.insert("collection_id".into(), serde_json::Value::String(c)); }
    if let Some(i) = q.item_id { filter.insert("item_id".into(), serde_json::Value::String(i)); }
    if let Some(h) = q.content_hash { filter.insert("content_hash".into(), serde_json::Value::String(h)); }
    if let Some(a) = q.owner_actor { filter.insert("owner_actor".into(), serde_json::Value::String(a)); }
    let mut sort: BTreeMap<String, i8> = BTreeMap::new();
    if let Some(s) = q.sort.as_deref() {
        let (field, dir) = if let Some(rest) = s.strip_prefix('-') { (rest, -1i8) } else { (s, 1i8) };
        if !field.is_empty() { sort.insert(field.to_string(), dir); }
    }
    let req = QueryRequest {
        collection: ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort,
        limit: q.limit.unwrap_or(50),
        offset: q.offset.unwrap_or(0),
    };
    match artifacts(&state).list(req).await {
        Ok(items) => {
            let count = items.len();
            (StatusCode::OK, Json(serde_json::json!({ "ok": true, "data": items, "count": count }))).into_response()
        }
        Err(e) => map_err(e),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '"' | '\\' | '\r' | '\n' => '_',
            c if (c as u32) < 0x20 => '_',
            c => c,
        })
        .collect()
}

fn mime_from_name(name: &str) -> Option<String> {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "csv" => "text/csv",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        _ => return None,
    };
    Some(mime.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_quotes_and_control_chars() {
        assert_eq!(sanitize_filename("a\"b"), "a_b");
        assert_eq!(sanitize_filename("a\nb"), "a_b");
        assert_eq!(sanitize_filename("normal-name.txt"), "normal-name.txt");
    }

    #[test]
    fn mime_from_name_basic() {
        assert_eq!(mime_from_name("a.png").as_deref(), Some("image/png"));
        assert_eq!(mime_from_name("a.TXT").as_deref(), Some("text/plain"));
        assert_eq!(mime_from_name("a.unknown"), None);
        assert_eq!(mime_from_name("noext"), None);
    }
}
