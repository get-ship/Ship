//! Pipeline config endpoints — list and read .ship/pipelines/*.yaml

use axum::{Json, extract::Path, http::StatusCode};
use serde::Deserialize;

use super::{err_response, ok_response, resolve_worktree};
use crate::rest_api::{ApiState, MeshResponse};

/// GET /api/workspaces/{id}/pipelines
pub async fn list_pipelines(
    axum::extract::State(state): axum::extract::State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let pipelines_dir = worktree.join(".ship").join("pipelines");

    if !pipelines_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "pipelines": [] })));
    }

    let entries = std::fs::read_dir(&pipelines_dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("read pipelines: {e}")))?;

    let mut pipelines = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let content = std::fs::read_to_string(&path).unwrap_or_default();

        // Parse YAML to extract metadata
        let val: serde_json::Value = match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let steps = val.get("steps").and_then(|v| v.as_array()).map(|arr| {
            arr.iter().filter_map(|s| {
                Some(serde_json::json!({
                    "id": s.get("id")?.as_str()?,
                    "type": s.get("type")?.as_str()?,
                    "agent": s.get("agent").and_then(|a| a.as_str()),
                    "after": s.get("after").and_then(|a| a.as_array()).map(|deps| {
                        deps.iter().filter_map(|d| d.as_str().map(String::from)).collect::<Vec<_>>()
                    }).unwrap_or_default(),
                    "model": s.get("model").and_then(|m| m.as_str()),
                }))
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

        pipelines.push(serde_json::json!({
            "name": name,
            "description": description,
            "steps": steps,
            "raw": content,
        }));
    }

    Ok(ok_response(serde_json::json!({ "pipelines": pipelines })))
}

#[derive(Deserialize)]
pub struct WritePipelineReq {
    pub content: String,
}

/// PUT /api/workspaces/{id}/pipelines/{name}
pub async fn write_pipeline(
    axum::extract::State(state): axum::extract::State<ApiState>,
    Path((id, name)): Path<(String, String)>,
    Json(body): Json<WritePipelineReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let pipelines_dir = worktree.join(".ship").join("pipelines");

    // Validate name
    if name.contains('/') || name.contains("..") || name.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid pipeline name"));
    }

    // Validate YAML
    let _: serde_json::Value = serde_yaml::from_str(&body.content)
        .map_err(|e| err_response(StatusCode::BAD_REQUEST, &format!("invalid YAML: {e}")))?;

    std::fs::create_dir_all(&pipelines_dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("create dir: {e}")))?;

    let path = pipelines_dir.join(format!("{name}.yaml"));
    std::fs::write(&path, &body.content)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("write: {e}")))?;

    Ok(ok_response(serde_json::json!({ "name": name })))
}

/// DELETE /api/workspaces/{id}/pipelines/{name}
pub async fn delete_pipeline(
    axum::extract::State(state): axum::extract::State<ApiState>,
    Path((id, name)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let path = worktree.join(".ship").join("pipelines").join(format!("{name}.yaml"));

    if !path.exists() {
        return Err(err_response(StatusCode::NOT_FOUND, "pipeline not found"));
    }

    std::fs::remove_file(&path)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("delete: {e}")))?;

    Ok(ok_response(serde_json::json!({ "name": name })))
}
