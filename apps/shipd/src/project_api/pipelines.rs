//! Pipeline config endpoints — list and read .ship/pipelines/*.jsonc (or legacy .yaml)

use axum::{Json, extract::Path, http::StatusCode};
use serde::Deserialize;
use serde_json::json;

use super::{err_response, ok_response, resolve_worktree};
use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

/// Recognised pipeline file extensions, preferred first.
const PIPELINE_EXTS: &[&str] = &["jsonc", "yaml"];

/// Parse pipeline content (JSONC or YAML) into a serde_json::Value.
fn parse_pipeline_content(content: &str) -> Result<serde_json::Value, String> {
    let trimmed = content.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        let json = compiler::jsonc::strip_jsonc_comments(content);
        serde_json::from_str(&json).map_err(|e| format!("invalid JSONC: {e}"))
    } else {
        serde_yaml::from_str(content).map_err(|e| format!("invalid YAML: {e}"))
    }
}

/// GET /api/workspaces/{id}/pipelines
#[utoipa::path(
    operation_id = "listPipelines",
    get,
    path = "/api/workspaces/{id}/pipelines",
    params(("id" = String, Path, description = "Workspace id")),
    responses(
        (status = 200, description = "Pipeline definitions discovered in .ship/pipelines/", body = MeshResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Workspace not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn list_pipelines(
    axum::extract::State(state): axum::extract::State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let pipelines_dir = worktree.join(".ship").join("pipelines");

    if !pipelines_dir.exists() {
        return Ok(ok_response(json!({ "pipelines": [] })));
    }

    let entries = std::fs::read_dir(&pipelines_dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("read pipelines: {e}")))?;

    let mut pipelines = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !PIPELINE_EXTS.contains(&ext) {
            continue;
        }
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let content = std::fs::read_to_string(&path).unwrap_or_default();

        let val: serde_json::Value = match parse_pipeline_content(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let steps = val.get("steps").and_then(|v| v.as_array()).map(|arr| {
            arr.iter().filter_map(|s| {
                Some(json!({
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

        pipelines.push(json!({
            "name": name,
            "description": description,
            "steps": steps,
            "raw": content,
        }));
    }

    Ok(ok_response(json!({ "pipelines": pipelines })))
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "content": "name: write-readme\nsteps:\n  - id: draft\n    agent: writer\n    produces: [README.md]\n  - id: review\n    agent: reviewer\n    consumes: [README.md]\n    promotes: [README.md]\n"
}))]
pub struct WritePipelineReq {
    pub content: String,
}

/// PUT /api/workspaces/{id}/pipelines/{name}
#[utoipa::path(
    operation_id = "writePipeline",
    put,
    path = "/api/workspaces/{id}/pipelines/{name}",
    params(
        ("id" = String, Path, description = "Workspace id"),
        ("name" = String, Path, description = "Pipeline name (without extension)"),
    ),
    request_body = WritePipelineReq,
    responses(
        (status = 200, description = "Pipeline definition written", body = MeshResponse),
        (status = 400, description = "Invalid pipeline name or content", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Workspace not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn write_pipeline(
    axum::extract::State(state): axum::extract::State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, name)): Path<(String, String)>,
    Json(body): Json<WritePipelineReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let pipelines_dir = worktree.join(".ship").join("pipelines");

    if name.contains('/') || name.contains("..") || name.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid pipeline name"));
    }

    // Validate content is parseable
    parse_pipeline_content(&body.content)
        .map_err(|e| err_response(StatusCode::BAD_REQUEST, &e))?;

    std::fs::create_dir_all(&pipelines_dir)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("create dir: {e}")))?;

    let path = pipelines_dir.join(format!("{name}.jsonc"));
    std::fs::write(&path, &body.content)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("write: {e}")))?;

    Ok(ok_response(json!({ "name": name })))
}

/// DELETE /api/workspaces/{id}/pipelines/{name}
#[utoipa::path(
    operation_id = "deletePipeline",
    delete,
    path = "/api/workspaces/{id}/pipelines/{name}",
    params(
        ("id" = String, Path, description = "Workspace id"),
        ("name" = String, Path, description = "Pipeline name (without extension)"),
    ),
    responses(
        (status = 200, description = "Pipeline deleted", body = MeshResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Insufficient scope", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Pipeline not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn delete_pipeline(
    axum::extract::State(state): axum::extract::State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, name)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = resolve_worktree(state.ship_dir(), &id)?;

    // Try .jsonc first, then .yaml
    let jsonc_path = worktree.join(".ship").join("pipelines").join(format!("{name}.jsonc"));
    let yaml_path = worktree.join(".ship").join("pipelines").join(format!("{name}.yaml"));
    let path = if jsonc_path.exists() {
        jsonc_path
    } else if yaml_path.exists() {
        yaml_path
    } else {
        return Err(err_response(StatusCode::NOT_FOUND, "pipeline not found"));
    };

    std::fs::remove_file(&path)
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("delete: {e}")))?;

    Ok(ok_response(json!({ "name": name })))
}
