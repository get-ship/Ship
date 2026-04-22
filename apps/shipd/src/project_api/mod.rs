//! Project REST API — exposes workspace-scoped file, git, event, and config operations.
//!
//! These endpoints replace the MCP bridge that Studio previously used for
//! filesystem, git, and event operations on a single local project.
//! Routes are mounted under `/api` in lib.rs.

pub mod artifacts;
mod agents_skills;
mod agents_skills_write;
mod git;
pub(crate) mod session_files;

pub use agents_skills::{list_agents, list_skills};
pub use agents_skills_write::{
    delete_skill_file, get_skill_vars, set_skill_var, write_agent, write_skill_file,
};
pub use artifacts::{artifacts_status, promote_artifact};
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

/// Resolve a workspace branch to its project root.
///
/// Priority: explicit DB path → checked-out git worktree → CWD walk (if
/// the id matches the daemon's current branch). The DB fallback of
/// `resolve_project_root` returns `ship_dir.parent()` when a workspace
/// has no explicit path — for a global ship_dir that's the user's HOME,
/// which has a `.ship/` dir and would pass the old "exists" check.
/// Writes would then land in `~/.ship/pipelines/` instead of the real
/// project root. We now only trust the DB answer when `path` is set.
pub(crate) fn resolve_worktree(
    ship_dir: &std::path::Path,
    id: &str,
) -> Result<PathBuf, (StatusCode, Json<MeshResponse>)> {
    // 1. DB-backed workspace resolution — only trust explicit paths.
    if let Ok((root, ws)) = runtime::workspace::resolve_project_root(ship_dir, id)
        && ws.path.is_some()
        && (root.join(".ship").exists() || root.join(".git").exists())
    {
        return Ok(root);
    }

    // 2. Workspace name is a branch — resolve to its actual worktree path
    //    via `git worktree list`. This gives artifact scope per worktree:
    //    /home/user/ship/.artifacts/ for main
    //    /home/user/ship-worktrees/fix/.artifacts/ for fix branch
    if let Some(path) = worktree_path_for_branch(id) {
        return Ok(path);
    }

    // 3. The daemon's active project root — only if id matches the
    //    current branch. get_project_dir walks up from CWD looking for
    //    .ship/, so it lands on the real project even if the daemon was
    //    started with a global ship_dir.
    if let Ok(project_ship) = runtime::project::get_project_dir(None)
        && let Some(root) = project_ship.parent()
        && let Some(current_branch) = current_branch_at(root)
        && current_branch == id
    {
        return Ok(root.to_path_buf());
    }

    Err((
        StatusCode::NOT_FOUND,
        Json(MeshResponse {
            ok: false,
            data: serde_json::json!(format!("workspace '{id}' not found")),
        }),
    ))
}

/// Read the current branch name at `root` via `git rev-parse`. None on any
/// failure (not a git repo, detached HEAD, git missing).
fn current_branch_at(root: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() || name == "HEAD" { None } else { Some(name) }
}

/// Look up the worktree path for a branch via `git worktree list --porcelain`.
/// Returns None if the branch isn't a checked-out worktree.
fn worktree_path_for_branch(branch: &str) -> Option<PathBuf> {
    let project_ship = runtime::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() { return None; }

    let text = String::from_utf8_lossy(&output.stdout);
    for block in text.split("\n\n") {
        let mut path: Option<String> = None;
        let mut found_branch: Option<String> = None;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p.trim().to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                found_branch = Some(b.trim().to_string());
            }
        }
        if let (Some(p), Some(b)) = (path, found_branch) {
            if b == branch { return Some(PathBuf::from(p)); }
        }
    }
    None
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

/// Resolve `target` under `base` safely, rejecting any path that escapes `base`
/// via `..` components or symlinks.
///
/// Handles both existing and not-yet-created files: canonicalizes the longest
/// existing ancestor and logically extends with the remaining components.
///
/// If `base` doesn't exist, returns 404 — since nothing can exist under a
/// nonexistent base, this is always the correct answer for reads/deletes, and
/// writers should create the base first.
pub(crate) fn safe_resolve(
    base: &std::path::Path,
    target: &std::path::Path,
) -> Result<std::path::PathBuf, (StatusCode, Json<MeshResponse>)> {
    let canonical_base = std::fs::canonicalize(base)
        .map_err(|_| err_response(StatusCode::NOT_FOUND, "file not found"))?;

    // Walk up until we find an existing ancestor we can canonicalize, then
    // append the non-existent remainder. This prevents symlink-based escape:
    // any symlink inside `base` that points outside it will be resolved by
    // canonicalize() and fail the starts_with check.
    let mut existing = target.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let resolved = loop {
        match std::fs::canonicalize(&existing) {
            Ok(p) => break p,
            Err(_) => match existing.file_name() {
                Some(name) => {
                    tail.push(name.to_os_string());
                    if !existing.pop() {
                        // No more components — target is outside anything that exists.
                        return Err(err_response(StatusCode::BAD_REQUEST, "invalid path"));
                    }
                }
                None => {
                    return Err(err_response(StatusCode::BAD_REQUEST, "invalid path"));
                }
            },
        }
    };
    let mut full = resolved;
    for segment in tail.into_iter().rev() {
        full.push(segment);
    }

    if !full.starts_with(&canonical_base) {
        return Err(err_response(StatusCode::BAD_REQUEST, "path escapes base directory"));
    }

    Ok(full)
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
        .emit(envelope, &ctx)
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
