//! REST API handlers for the artifacts service.
//!
//! Project-scoped blob upload/download on top of the `artifacts` collection.
//! Metadata is queryable through the standard collections query API; binary
//! bytes are served directly by `GET /api/artifacts/{id}`.
//!
//! The daemon must be attached to a project workspace (via `--workspace`).
//! Requests with no project context return 400.

use axum::{
    Extension, Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::ApiState;
use runtime::artifacts::{ARTIFACTS_COLLECTION_ID, ArtifactService};
use runtime::collections::query::QueryRequest;

/// Max upload size: 50 MB.
pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg }))).into_response()
}

fn err_owned(status: StatusCode, msg: String) -> Response {
    (status, Json(serde_json::json!({ "ok": false, "error": msg }))).into_response()
}

/// Lock the daemon's shared DataService and return a guard.
async fn data_guard(
    state: &ApiState,
) -> Result<tokio::sync::MutexGuard<'_, Option<runtime::data::DataService>>, Response> {
    state
        .data_service()
        .await
        .map_err(|e| err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Resolve the project slug for the attached workspace or return 400.
fn project_slug_or_error(state: &ApiState) -> Result<String, Response> {
    state.project_slug().ok_or_else(|| {
        err(
            StatusCode::BAD_REQUEST,
            "no project workspace attached — start the daemon with --workspace",
        )
    })
}

/// POST /api/artifacts — multipart upload.
///
/// Expected form fields:
/// - `file` (required): the binary file part
/// - `collection_id` (optional): link to a collection
/// - `item_id` (optional): link to an item inside that collection
pub async fn upload_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    mut multipart: Multipart,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let slug = match project_slug_or_error(&state) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut file_mime: Option<String> = None;
    let mut collection_id: Option<String> = None;
    let mut item_id: Option<String> = None;

    loop {
        let field_result = multipart.next_field().await;
        let field = match field_result {
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
                    Err(e) => {
                        return err_owned(
                            StatusCode::BAD_REQUEST,
                            format!("failed to read file bytes: {e}"),
                        );
                    }
                };
                if bytes.len() > MAX_UPLOAD_BYTES {
                    return err_owned(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!("file exceeds {MAX_UPLOAD_BYTES}-byte limit"),
                    );
                }
                file_bytes = Some(bytes.to_vec());
                file_name = name;
                file_mime = mime;
            }
            "collection_id" => {
                collection_id = field.text().await.ok().filter(|s| !s.is_empty());
            }
            "item_id" => {
                item_id = field.text().await.ok().filter(|s| !s.is_empty());
            }
            _ => {
                let _ = field.bytes().await;
            }
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

    let svc = ArtifactService::new(state.ship_dir(), &slug);
    let mut guard = match data_guard(&state).await {
        Ok(g) => g,
        Err(resp) => return resp,
    };
    let data = guard.as_mut().expect("data service initialized");

    let created_by = auth.agent_id.as_deref();
    match svc
        .store(
            data,
            state.kernel(),
            &name,
            &mime,
            &bytes,
            collection_id.as_deref(),
            item_id.as_deref(),
            created_by,
        )
        .await
    {
        Ok(record) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "ok": true, "data": record })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("validation") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            err_owned(status, msg)
        }
    }
}

/// GET /api/artifacts/{id} — download an artifact.
pub async fn download_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    let slug = match project_slug_or_error(&state) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let svc = ArtifactService::new(state.ship_dir(), &slug);
    let mut guard = match data_guard(&state).await {
        Ok(g) => g,
        Err(resp) => return resp,
    };
    let data = guard.as_mut().expect("data service initialized");

    match svc.get(data, &id) {
        Ok(Some((record, bytes))) => {
            let disposition = format!(
                "attachment; filename=\"{}\"",
                sanitize_filename(&record.name)
            );
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, record.mime.as_str())
                .header(header::CONTENT_DISPOSITION, disposition)
                .header(header::CONTENT_LENGTH, bytes.len())
                .body(Body::from(bytes))
                .unwrap_or_else(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "response build failed"))
        }
        Ok(None) => err_owned(StatusCode::NOT_FOUND, format!("artifact '{id}' not found")),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("blob missing") {
                err_owned(StatusCode::NOT_FOUND, msg)
            } else {
                err_owned(StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        }
    }
}

/// DELETE /api/artifacts/{id} — remove blob + metadata.
pub async fn delete_artifact(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::WriteWorkspace) {
        return e.into_response();
    }
    let slug = match project_slug_or_error(&state) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let svc = ArtifactService::new(state.ship_dir(), &slug);
    let mut guard = match data_guard(&state).await {
        Ok(g) => g,
        Err(resp) => return resp,
    };
    let data = guard.as_mut().expect("data service initialized");

    match svc.delete(data, state.kernel(), &id).await {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Ok(false) => err_owned(StatusCode::NOT_FOUND, format!("artifact '{id}' not found")),
        Err(e) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// GET /api/artifacts — list artifacts as metadata.
///
/// Supports query-string filters: `mime`, `collection_id`, `item_id`,
/// `limit`, `offset`, `sort` (field name optionally prefixed with `-`
/// for descending).
#[derive(Deserialize, Default)]
pub struct ListQuery {
    pub mime: Option<String>,
    pub collection_id: Option<String>,
    pub item_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort: Option<String>,
}

pub async fn list_artifacts(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Query(q): Query<ListQuery>,
) -> Response {
    if let Err(e) = require_scope(&auth, Scope::ReadWorkspace) {
        return e.into_response();
    }
    let slug = match project_slug_or_error(&state) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let mut filter: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    if let Some(m) = q.mime {
        filter.insert("mime".into(), serde_json::Value::String(m));
    }
    if let Some(c) = q.collection_id {
        filter.insert("collection_id".into(), serde_json::Value::String(c));
    }
    if let Some(i) = q.item_id {
        filter.insert("item_id".into(), serde_json::Value::String(i));
    }

    let mut sort: BTreeMap<String, i8> = BTreeMap::new();
    if let Some(s) = q.sort.as_deref() {
        let (field, dir) = if let Some(rest) = s.strip_prefix('-') {
            (rest, -1i8)
        } else {
            (s, 1i8)
        };
        if !field.is_empty() {
            sort.insert(field.to_string(), dir);
        }
    }

    let req = QueryRequest {
        collection: ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort,
        limit: q.limit.unwrap_or(50),
        offset: q.offset.unwrap_or(0),
    };

    let svc = ArtifactService::new(state.ship_dir(), &slug);
    let mut guard = match data_guard(&state).await {
        Ok(g) => g,
        Err(resp) => return resp,
    };
    let data = guard.as_mut().expect("data service initialized");

    match svc.list(data, req) {
        Ok(items) => {
            let count = items.len();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ok": true,
                    "data": items,
                    "count": count,
                })),
            )
                .into_response()
        }
        Err(e) => err_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

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
