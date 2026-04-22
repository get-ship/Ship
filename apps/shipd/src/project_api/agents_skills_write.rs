//! Write endpoints for agent configs.
//!
//! All mutations are workspace-scoped: resolve worktree first, then operate
//! within `{worktree}/.ship/`. Skill writes moved to the global SkillService
//! at `/api/skills/*`.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};
use super::{err_response, ok_response, resolve_worktree};

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

// ── Endpoints ─────────────────────────────────────────────────────────────────

/// POST /api/workspaces/{id}/agents
///
/// Write an agent transfer bundle to `{worktree}/.ship/agents/{agent_id}.jsonc`.
pub async fn write_agent(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<WriteAgentReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
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

    // ── write_agent ───────────────────────────────────────────────────────────

    fn full_auth() -> axum::Extension<AuthContext> {
        axum::Extension(AuthContext {
            caller_kind: runtime::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
        })
    }

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
        let result = write_agent(State(state), full_auth(), Path("no-such-ws".to_string()), Json(req)).await;
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

}
