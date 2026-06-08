//! Agent and skill config endpoints — serve .ship/agents/ and .ship/skills/ data.

use axum::{Json, extract::{Path, State}, http::StatusCode};

use crate::routes::rest_api::{ApiState, MeshResponse};

use super::{err_response, ok_response};

/// GET /api/v1/projects/{id}/agents
pub async fn list_agents(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = state.resolve_worktree(&id)?;
    let agents_dir = ship_substrate::ShipHome::project_ship_dir(&worktree).join("agents");

    if !agents_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "agents": [] })));
    }

    let entries = std::fs::read_dir(&agents_dir).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to read agents dir: {e}"),
        )
    })?;

    let mut agents = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if !path.is_file() || !matches!(ext, Some("jsonc" | "json")) {
            continue;
        }

        let id_str = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let json_str = ship_substrate::fs_util::strip_jsonc_comments(&raw);
        let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Agent config may nest under "agent" key or be flat
        let agent_obj = parsed.get("agent").unwrap_or(&parsed);
        agents.push(serde_json::json!({
            "id": agent_obj.get("id").and_then(|v| v.as_str()).unwrap_or(&id_str),
            "name": agent_obj.get("name").and_then(|v| v.as_str()).unwrap_or(&id_str),
            "description": agent_obj.get("description").and_then(|v| v.as_str()),
            "skills": parsed.get("skills").cloned().unwrap_or(serde_json::json!([])),
            "harnesses": agent_obj.get("harnesses").cloned().unwrap_or(serde_json::json!([])),
        }));
    }

    Ok(ok_response(serde_json::json!({ "agents": agents })))
}


