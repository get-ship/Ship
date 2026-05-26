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

use crate::access::RuntimeAccess;
use crate::auth::{AuthContext};
use crate::routes::rest_api::ApiState;
use crate::auth::scopes::{can_read_artifacts, can_write_artifacts};
use ship_platform::services::artifacts::service::{
    ArtifactsService, Error as ArtifactsError, StoreArgs,
};
use ship_platform::artifacts::ArtifactScope;

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

#[derive(Deserialize, Default)]
pub struct UploadQuery {
    /// Artifact scope: `run`, `project`, or `user`.
    pub scope: Option<String>,
    /// Required when `scope=run`.
    pub run_id: Option<String>,
    /// Required when `scope=user`.
    pub user_id: Option<String>,
}

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

    // Default TTL for run-scope artifacts when the caller didn't supply
    // an explicit `expires_at`. Per design, run-scope is ephemeral and
    // garbage-collected after the run is done. 24h gives enough overlap
    // for debug attach, promote, and download flows. Project/user-scope
    // artifacts are kept until explicitly deleted.
    const RUN_SCOPE_DEFAULT_TTL_HOURS: i64 = 24;
    let computed_expires = match (expires_at, &scope) {
        (Some(t), _) => Some(t),
        (None, ArtifactScope::Run { .. }) => {
            let exp = chrono::Utc::now() + chrono::Duration::hours(RUN_SCOPE_DEFAULT_TTL_HOURS);
            Some(exp.to_rfc3339())
        }
        (None, _) => None,
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

#[derive(Deserialize)]
pub struct PatchBody {
    pub name: Option<String>,
    pub owner_actor: Option<String>,
    pub expires_at: Option<String>,
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
