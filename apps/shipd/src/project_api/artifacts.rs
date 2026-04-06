//! Artifact promotion — copy files between workspace (.ship-session/) and
//! project (~/.ship/artifacts/{slug}/) tiers.

use axum::{Json, extract::Path, http::StatusCode};
use serde::Deserialize;
use std::path::PathBuf;

use super::{err_response, ok_response, resolve_worktree};
use crate::rest_api::MeshResponse;

fn project_artifacts_dir() -> Result<PathBuf, (StatusCode, Json<MeshResponse>)> {
    let global = runtime::project::get_global_dir()
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Use the active project's slug for namespacing
    let ship_dir = runtime::project::get_project_dir(None)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let slug = runtime::project::project_slug_from_ship_dir(&ship_dir);

    let dir = global.join("artifacts").join(slug);
    std::fs::create_dir_all(&dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("create artifacts dir: {e}")))?;
    Ok(dir)
}

/// GET /api/workspaces/{id}/artifacts/status
/// Returns promotion status for all workspace files — which ones have been
/// saved to the project tier.
pub async fn artifacts_status(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let session_dir = worktree.join(".ship-session");
    let project_dir = project_artifacts_dir()?;

    let mut statuses: Vec<serde_json::Value> = Vec::new();

    if session_dir.exists() {
        collect_status(&session_dir, &session_dir, &project_dir, &mut statuses)?;
    }

    Ok(ok_response(serde_json::json!({ "files": statuses })))
}

fn collect_status(
    base: &std::path::Path,
    dir: &std::path::Path,
    project_dir: &std::path::Path,
    out: &mut Vec<serde_json::Value>,
) -> Result<(), (StatusCode, Json<MeshResponse>)> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("read dir: {e}")))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_status(base, &path, project_dir, out)?;
            continue;
        }
        if !path.is_file() { continue; }

        let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().to_string();
        let project_path = project_dir.join(&rel);
        let saved = project_path.exists();

        out.push(serde_json::json!({
            "path": rel,
            "saved": saved,
        }));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct PromoteReq {
    pub path: String,
}

/// POST /api/workspaces/{id}/artifacts/promote
/// Copy a file from workspace artifacts to project artifacts.
pub async fn promote_artifact(
    Path(id): Path<String>,
    Json(body): Json<PromoteReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let source = worktree.join(".ship-session").join(&body.path);
    let project_dir = project_artifacts_dir()?;
    let dest = project_dir.join(&body.path);

    // Validate source exists
    if !source.is_file() {
        return Err(err_response(StatusCode::NOT_FOUND, "source file not found"));
    }

    // Prevent path traversal
    if !source.starts_with(worktree.join(".ship-session")) {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid path"));
    }

    // Create parent dirs in project artifacts
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("create dir: {e}")))?;
    }

    // Copy file
    std::fs::copy(&source, &dest)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("copy: {e}")))?;

    Ok(ok_response(serde_json::json!({
        "path": body.path,
        "promoted": true,
    })))
}

/// GET /api/artifacts
/// List all project-level artifacts (persisted across workspaces).
pub async fn list_project_artifacts(
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let project_dir = project_artifacts_dir()?;

    let mut files: Vec<serde_json::Value> = Vec::new();
    if project_dir.exists() {
        super::session_files::collect_files_recursive(&project_dir, &project_dir, &mut files)?;
    }

    Ok(ok_response(serde_json::json!({ "files": files })))
}
