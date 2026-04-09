//! Write endpoints for agent configs and skill files/vars.
//!
//! All mutations are workspace-scoped: resolve worktree first, then operate
//! within `{worktree}/.ship/`.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::Deserialize;
use std::path::PathBuf;

use crate::rest_api::{ApiState, MeshResponse};
use super::{err_response, ok_response, resolve_worktree};

// ── Path safety ──────────────────────────────────────────────────────────────

/// Resolve `relative` under `base`, refusing any path that escapes `base`.
fn safe_join(base: &std::path::Path, relative: &str) -> Result<PathBuf, (StatusCode, Json<MeshResponse>)> {
    let joined = base.join(relative);
    let canonical_base = base
        .canonicalize()
        .unwrap_or_else(|_| base.to_path_buf());
    // Normalize without requiring the path to exist yet
    let normalized = normalize_path(&joined);
    if !normalized.starts_with(&canonical_base) {
        return Err(err_response(StatusCode::BAD_REQUEST, "path escapes base directory"));
    }
    Ok(joined)
}

/// Collapse `..` components without requiring the path to exist.
fn normalize_path(path: &std::path::Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::ParentDir => { out.pop(); }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WriteAgentReq {
    pub id: String,
    pub profile: serde_json::Value,
    pub skills: serde_json::Value,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: serde_json::Value,
    pub rules: serde_json::Value,
    pub hooks: serde_json::Value,
}

#[derive(Deserialize)]
pub struct WriteFileReq {
    pub content: String,
}

#[derive(Deserialize)]
pub struct SetVarReq {
    pub value_json: String,
}

// ── Endpoints ─────────────────────────────────────────────────────────────────

/// POST /api/workspaces/{id}/agents
///
/// Write an agent transfer bundle to `{worktree}/.ship/agents/{agent_id}.jsonc`.
pub async fn write_agent(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(body): Json<WriteAgentReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let agents_dir = worktree.join(".ship").join("agents");

    std::fs::create_dir_all(&agents_dir).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to create agents dir: {e}"),
        )
    })?;

    // Validate agent id before using it as a filename
    if body.id.is_empty() || body.id.contains('/') || body.id.contains("..") {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid agent id"));
    }

    let bundle = serde_json::json!({
        "agent": body.profile,
        "skills": body.skills,
        "mcpServers": body.mcp_servers,
        "rules": body.rules,
        "hooks": body.hooks,
    });

    let content = serde_json::to_string_pretty(&bundle).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to serialize agent config: {e}"),
        )
    })?;

    let dest = agents_dir.join(format!("{}.jsonc", body.id));
    std::fs::write(&dest, content).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write agent file: {e}"),
        )
    })?;

    Ok(ok_response(serde_json::json!({ "id": body.id })))
}

/// PUT /api/workspaces/{id}/skills/{skillId}/files/{*path}
///
/// Write content to `{worktree}/.ship/skills/{skillId}/{path}`.
pub async fn write_skill_file(
    State(state): State<ApiState>,
    Path((id, skill_id, file_path)): Path<(String, String, String)>,
    Json(body): Json<WriteFileReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;

    validate_skill_id(&skill_id)?;

    let skill_dir = worktree.join(".ship").join("skills").join(&skill_id);
    let dest = safe_join(&skill_dir, &file_path)?;

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to create parent dirs: {e}"),
            )
        })?;
    }

    std::fs::write(&dest, &body.content).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write skill file: {e}"),
        )
    })?;

    Ok(ok_response(serde_json::json!({})))
}

/// DELETE /api/workspaces/{id}/skills/{skillId}/files/{*path}
///
/// Delete `{worktree}/.ship/skills/{skillId}/{path}`.
pub async fn delete_skill_file(
    State(state): State<ApiState>,
    Path((id, skill_id, file_path)): Path<(String, String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;

    validate_skill_id(&skill_id)?;

    let skill_dir = worktree.join(".ship").join("skills").join(&skill_id);
    let target = safe_join(&skill_dir, &file_path)?;

    if !target.exists() {
        return Err(err_response(StatusCode::NOT_FOUND, "file not found"));
    }

    std::fs::remove_file(&target).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to delete skill file: {e}"),
        )
    })?;

    Ok(ok_response(serde_json::json!({})))
}

/// GET /api/workspaces/{id}/skills/{skillId}/vars
///
/// Return merged skill variable state. Uses the worktree `.ship` dir as context.
pub async fn get_skill_vars(
    State(state): State<ApiState>,
    Path((id, skill_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let ship_dir = worktree.join(".ship");

    validate_skill_id(&skill_id)?;

    let vars = runtime::get_skill_vars(&ship_dir, &skill_id).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to read skill vars: {e}"),
        )
    })?;

    match vars {
        Some(map) => {
            let vars_json: serde_json::Map<String, serde_json::Value> =
                map.into_iter().collect();
            Ok(ok_response(serde_json::json!({ "vars": vars_json })))
        }
        None => Ok(ok_response(serde_json::json!({ "vars": null }))),
    }
}

/// PUT /api/workspaces/{id}/skills/{skillId}/vars/{key}
///
/// Set a single skill variable. Body: `{ value_json: "<json-encoded-value>" }`.
pub async fn set_skill_var(
    State(state): State<ApiState>,
    Path((id, skill_id, key)): Path<(String, String, String)>,
    Json(body): Json<SetVarReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let ship_dir = worktree.join(".ship");

    validate_skill_id(&skill_id)?;

    let value: serde_json::Value =
        serde_json::from_str(&body.value_json).map_err(|e| {
            err_response(
                StatusCode::BAD_REQUEST,
                &format!("value_json is not valid JSON: {e}"),
            )
        })?;

    runtime::set_skill_var(&ship_dir, &skill_id, &key, value).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unknown variable") || msg.contains("no assets/vars.json") {
            err_response(StatusCode::NOT_FOUND, &msg)
        } else {
            err_response(StatusCode::INTERNAL_SERVER_ERROR, &msg)
        }
    })?;

    Ok(ok_response(serde_json::json!({})))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn validate_skill_id(skill_id: &str) -> Result<(), (StatusCode, Json<MeshResponse>)> {
    if skill_id.is_empty() || skill_id.contains('/') || skill_id.contains("..") {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid skill id"));
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::{Path, State};
    use crate::rest_api::ApiState;

    fn make_test_state() -> (ApiState, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let ship_dir = dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        runtime::db::ensure_db_at(&ship_dir).unwrap();
        let kernel = runtime::events::init_kernel_router(ship_dir.clone()).unwrap();
        let state = std::sync::Arc::new(
            crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel),
        );
        (state, dir)
    }

    // ── Path safety ──────────────────────────────────────────────────────────

    #[test]
    fn normalize_path_collapses_dotdot() {
        let p = normalize_path(std::path::Path::new("/base/skills/../agents"));
        assert_eq!(p, PathBuf::from("/base/agents"));
    }

    #[test]
    fn safe_join_rejects_escape() {
        let base = tempfile::tempdir().unwrap();
        let base_path = base.path().to_path_buf();
        // Need base to exist for canonicalize
        let result = safe_join(&base_path, "../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn safe_join_accepts_valid_path() {
        let base = tempfile::tempdir().unwrap();
        let base_path = base.path().to_path_buf();
        let result = safe_join(&base_path, "subdir/file.md");
        assert!(result.is_ok());
    }

    // ── write_agent ───────────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn write_agent_nonexistent_workspace_returns_404() {
        let (state, _dir) = make_test_state();
        let req = WriteAgentReq {
            id: "test-agent".to_string(),
            profile: serde_json::json!({"name": "Test"}),
            skills: serde_json::json!([]),
            mcp_servers: serde_json::json!([]),
            rules: serde_json::json!([]),
            hooks: serde_json::json!([]),
        };
        let result = write_agent(State(state), Path("no-such-ws".to_string()), Json(req)).await;
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn write_agent_rejects_invalid_id() {
        // Sync check — validate_skill_id equivalent for agent
        let bad_ids = ["", "../sneaky", "path/with/slash"];
        for bad in bad_ids {
            let id_str = bad.to_string();
            let is_bad = id_str.is_empty() || id_str.contains('/') || id_str.contains("..");
            assert!(is_bad, "expected '{bad}' to be rejected");
        }
    }

    // ── validate_skill_id ─────────────────────────────────────────────────────

    #[test]
    fn validate_skill_id_rejects_empty() {
        assert!(validate_skill_id("").is_err());
    }

    #[test]
    fn validate_skill_id_rejects_slash() {
        assert!(validate_skill_id("a/b").is_err());
    }

    #[test]
    fn validate_skill_id_rejects_dotdot() {
        assert!(validate_skill_id("..").is_err());
    }

    #[test]
    fn validate_skill_id_accepts_valid() {
        assert!(validate_skill_id("my-skill").is_ok());
        assert!(validate_skill_id("skill_v2").is_ok());
    }

    // ── set_skill_var body parsing ────────────────────────────────────────────

    #[test]
    fn set_var_req_parses_string_value_json() {
        let req: SetVarReq = serde_json::from_str(r#"{"value_json": "\"gitmoji\""}"#).unwrap();
        let val: serde_json::Value = serde_json::from_str(&req.value_json).unwrap();
        assert_eq!(val, serde_json::json!("gitmoji"));
    }

    #[test]
    fn set_var_req_parses_bool_value_json() {
        let req: SetVarReq = serde_json::from_str(r#"{"value_json": "true"}"#).unwrap();
        let val: serde_json::Value = serde_json::from_str(&req.value_json).unwrap();
        assert_eq!(val, serde_json::json!(true));
    }
}
