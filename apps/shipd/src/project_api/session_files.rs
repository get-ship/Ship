//! Session file endpoints — list, read, write, delete files in .artifacts/.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

use super::{err_response, ok_response, resolve_worktree, safe_resolve};

pub(crate) fn file_type_from_ext(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "html",
        Some("md") => "markdown",
        Some("json") => "json",
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("svg")
        | Some("bmp") | Some("ico") => "image",
        Some("url") => "url",
        _ => "text",
    }
}

pub(super) fn collect_files_recursive(
    base: &std::path::Path,
    dir: &std::path::Path,
    files: &mut Vec<serde_json::Value>,
) -> Result<(), (StatusCode, Json<MeshResponse>)> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to read dir: {e}"),
        )
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(base, &path, files)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let rel_path = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified_at = meta.as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let created_at = meta.as_ref()
            .and_then(|m| m.created().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(modified_at);
        let file_type = file_type_from_ext(&path);
        files.push(serde_json::json!({
            "name": name,
            "path": rel_path,
            "size": size,
            "type": file_type,
            "modifiedAt": modified_at,
            "createdAt": created_at,
        }));
    }
    Ok(())
}

/// GET /api/workspaces/{id}/session-files
#[utoipa::path(
    operation_id = "listSessionFiles",
    get,
    path = "/api/workspaces/{id}/session-files",
    params(("id" = String, Path, description = "Workspace id")),
    responses(
        (status = 200, description = "Files list under .artifacts/", body = MeshResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Workspace not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn list_session_files(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let session_dir = worktree.join(".artifacts");

    if !session_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "files": [] })));
    }

    let mut files = Vec::new();
    collect_files_recursive(&session_dir, &session_dir, &mut files)?;

    Ok(ok_response(serde_json::json!({ "files": files })))
}

/// GET /api/workspaces/{id}/session-files/*path
#[utoipa::path(
    operation_id = "readSessionFile",
    get,
    path = "/api/workspaces/{id}/session-files/{path}",
    params(
        ("id" = String, Path, description = "Workspace id"),
        ("path" = String, Path, description = "Relative path under .artifacts/"),
    ),
    responses(
        (status = 200, description = "File content (data-URI for images, text otherwise)", body = MeshResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "File not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn read_session_file(
    State(state): State<ApiState>,
    Path((id, file_path)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let session_dir = worktree.join(".artifacts");
    let requested = session_dir.join(&file_path);
    let target = safe_resolve(&session_dir, &requested)?;

    let file_type = file_type_from_ext(&target);

    if file_type == "image" {
        // Return binary files as data URI
        let bytes = std::fs::read(&target).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                err_response(StatusCode::NOT_FOUND, "file not found")
            } else {
                err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("failed to read file: {e}"))
            }
        })?;
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let ext = target.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let mime = match ext {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "bmp" => "image/bmp",
            "ico" => "image/x-icon",
            _ => "application/octet-stream",
        };
        let data_uri = format!("data:{mime};base64,{b64}");
        return Ok(ok_response(serde_json::json!({ "content": data_uri })));
    }

    let content = std::fs::read_to_string(&target).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            err_response(StatusCode::NOT_FOUND, "file not found")
        } else {
            err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("failed to read file: {e}"))
        }
    })?;

    Ok(ok_response(serde_json::json!({ "content": content })))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct WriteFileReq {
    pub content: String,
}

/// PUT /api/workspaces/{id}/session-files/*path
#[utoipa::path(
    operation_id = "writeSessionFile",
    put,
    path = "/api/workspaces/{id}/session-files/{path}",
    params(
        ("id" = String, Path, description = "Workspace id"),
        ("path" = String, Path, description = "Relative path under .artifacts/"),
    ),
    request_body = WriteFileReq,
    responses(
        (status = 200, description = "File written", body = MeshResponse),
        (status = 400, description = "Path escapes base directory", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Workspace not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn write_session_file(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, file_path)): Path<(String, String)>,
    Json(body): Json<WriteFileReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let session_dir = worktree.join(".artifacts");
    // session_dir may not exist yet; create it first so canonicalize succeeds.
    std::fs::create_dir_all(&session_dir).map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("failed to create session dir: {e}"))
    })?;
    let requested = session_dir.join(&file_path);
    let target = safe_resolve(&session_dir, &requested)?;

    // Ensure parent directories exist
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to create directory: {e}"),
            )
        })?;
    }

    std::fs::write(&target, &body.content).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write file: {e}"),
        )
    })?;

    Ok(ok_response(serde_json::json!({ "path": file_path })))
}

/// DELETE /api/workspaces/{id}/session-files/*path
#[utoipa::path(
    operation_id = "deleteSessionFile",
    delete,
    path = "/api/workspaces/{id}/session-files/{path}",
    params(
        ("id" = String, Path, description = "Workspace id"),
        ("path" = String, Path, description = "Relative path under .artifacts/"),
    ),
    responses(
        (status = 200, description = "File deleted", body = MeshResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "File not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn delete_session_file(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, file_path)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let session_dir = worktree.join(".artifacts");
    let requested = session_dir.join(&file_path);
    let target = safe_resolve(&session_dir, &requested)?;

    std::fs::remove_file(&target).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            err_response(StatusCode::NOT_FOUND, "file not found")
        } else {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to delete file: {e}"),
            )
        }
    })?;

    Ok(ok_response(serde_json::json!({})))
}
