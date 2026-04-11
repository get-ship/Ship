//! Project REST API — exposes workspace-scoped file, git, event, and config operations.
//!
//! These endpoints replace the MCP bridge that Studio previously used for
//! filesystem, git, and event operations on a single local project.
//! Routes are mounted under `/api` in lib.rs.

pub mod artifacts;
mod agents_skills;
mod agents_skills_write;
mod git;
mod pipelines;
pub(crate) mod session_files;

pub use agents_skills::{list_agents, list_skills};
pub use agents_skills_write::{
    delete_skill_file, get_skill_vars, set_skill_var, write_agent, write_skill_file,
};
pub use artifacts::{artifacts_status, list_project_artifacts, promote_artifact};
pub use pipelines::{list_pipelines, write_pipeline, delete_pipeline};
pub use git::{git_diff, git_log, git_status, project_git_diff, project_git_log, project_git_status};
pub use session_files::{
    delete_session_file, list_session_files, read_session_file, write_session_file,
};

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use std::path::PathBuf;

use crate::auth::{Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

// ---- Shared helpers ----

/// Resolve a workspace branch to its project root. Reconciles first, then
/// falls back to ship_dir parent for non-worktree workspaces.
pub(crate) fn resolve_worktree(
    ship_dir: &std::path::Path,
    id: &str,
) -> Result<PathBuf, (StatusCode, Json<MeshResponse>)> {
    let (root, _ws) = runtime::workspace::resolve_project_root(ship_dir, id)
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(MeshResponse {
                        ok: false,
                        data: serde_json::json!("workspace not found"),
                    }),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(MeshResponse {
                        ok: false,
                        data: serde_json::json!(msg),
                    }),
                )
            }
        })?;

    Ok(root)
}

pub(crate) fn ok_response(data: serde_json::Value) -> (StatusCode, Json<MeshResponse>) {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

pub(crate) fn err_response(
    status: StatusCode,
    msg: &str,
) -> (StatusCode, Json<MeshResponse>) {
    (
        status,
        Json(MeshResponse {
            ok: false,
            data: serde_json::json!(msg),
        }),
    )
}

// ---- Event emit endpoint ----

#[derive(Deserialize)]
pub struct EmitEventReq {
    pub event_type: String,
    pub entity_id: String,
    pub workspace_id: Option<String>,
    pub payload: serde_json::Value,
}

/// POST /api/events/emit
pub async fn emit_event(
    State(state): State<ApiState>,
    auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Json(body): Json<EmitEventReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    if let Some(axum::Extension(ref ac)) = auth {
        require_scope(ac, Scope::EmitEvents)?;
    }
    let mut envelope =
        runtime::events::EventEnvelope::new(&body.event_type, &body.entity_id, &body.payload)
            .map_err(|e| err_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    envelope = envelope.with_actor_id("studio");
    if let Some(ref ws_id) = body.workspace_id {
        envelope = envelope.with_context(Some(ws_id), None);
    }

    // Use auth context if available (token-based), otherwise treat as CLI caller.
    // Studio (localhost browser) is exempted from auth but needs Cli caller kind
    // to emit to system namespaces like studio.*.
    let ctx = if let Some(axum::Extension(ac)) = auth {
        ac.to_daemon_emit_context()
    } else {
        runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Cli,
            skill_id: None,
            workspace_id: body.workspace_id.clone(),
            session_id: None,
        }
    };

    state
        .kernel()
        .lock()
        .await
        .route(envelope, &ctx)
        .await
        .map_err(|e| {
            err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        })?;

    Ok(ok_response(serde_json::json!({})))
}

// ---- Workspace delete endpoint ----

/// DELETE /api/workspaces/{id}
pub async fn delete_workspace(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<crate::auth::AuthContext>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    runtime::workspace::delete_workspace(state.ship_dir(), &id).map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;

    Ok(ok_response(serde_json::json!({ "branch": id })))
}

// ---- Workspace ensure endpoint (replaces activate) ----

/// POST /api/workspaces/{id}/activate
///
/// Creates the workspace if it doesn't exist. No-op if it does.
/// Kept as /activate for API backward compat.
pub async fn activate_workspace(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<crate::auth::AuthContext>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    runtime::workspace::create_workspace(
        state.ship_dir(),
        runtime::workspace::CreateWorkspaceRequest {
            branch: id.clone(),
            ..Default::default()
        },
    ).map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;

    Ok(ok_response(serde_json::json!({ "branch": id })))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_inline {
    use super::*;
    use axum::extract::{Path, State};

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

    #[tokio::test(flavor = "multi_thread")]
    async fn nonexistent_workspace_returns_404() {
        let (state, _dir) = make_test_state();
        let response = git_status(
            State(state),
            Path("nonexistent-branch-xyz".to_string()),
        )
            .await
            .unwrap_err();
        let (status, body) = response;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.0.ok, false);
    }

    #[test]
    fn file_type_detection() {
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("a.html")),
            "html"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("b.md")),
            "markdown"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("c.json")),
            "json"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("d.txt")),
            "text"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("e")),
            "text"
        );
    }
}
