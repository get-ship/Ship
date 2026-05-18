//! Project REST API — exposes workspace-scoped file, git, event, and config operations.
//!
//! These endpoints replace the MCP bridge that Studio previously used for
//! filesystem, git, and event operations on a single local project.
//! Routes are mounted under `/api` in lib.rs.

pub mod agents_skills;
pub mod agents_skills_write;
pub mod jobs_status;
pub mod session_files;

pub use jobs_status::{JobStatusReq, StepDoneReq, job_step_done, update_job_status};

pub use agents_skills::{list_agents, list_skills};
pub use agents_skills_write::{
    SetVarReq, delete_skill_file, get_skill_vars, set_skill_var, write_skill_file,
};
pub use session_files::{
    delete_session_file, list_session_files, read_session_file, write_session_file,
};

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;

use crate::routes::rest_api::{ApiState, MeshResponse};

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
// Production worktree resolution: branch → worktree path, falling back
// to the current project root if the id matches the current branch.
// This logic lives in `ship_runtime::default_worktree_resolver()`, which
// is what handlers actually reach via `state.resolve_worktree(id)`. The
// `worktree_path_for_branch` and `current_branch_at` helpers below
// remain pub(crate) so the default resolver can call them.

/// Read the current branch name at `root` via `git rev-parse`. None on any
/// failure (not a git repo, detached HEAD, git missing).
pub(crate) fn current_branch_at(root: &std::path::Path) -> Option<String> {
    let output = ship_substrate::spawn_bounded::Cmd::new("git")
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
pub(crate) fn worktree_path_for_branch(branch: &str) -> Option<PathBuf> {
    let project_ship = ship_substrate::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    let output = ship_substrate::spawn_bounded::Cmd::new("git")
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
        if let (Some(p), Some(b)) = (path, found_branch)
            && b == branch { return Some(PathBuf::from(p)); }
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
            data: json!(msg),
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

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "event_type": "deployment.completed",
    "entity_id": "write-readme",
    "workspace_id": "default",
    "actor_id": "agent-writer",
    "payload": { "slug": "write-readme", "status": "ok" }
}))]
pub struct EmitEventReq {
    pub event_type: String,
    pub entity_id: String,
    pub workspace_id: Option<String>,
    pub payload: serde_json::Value,
    /// Optional actor attribution. When set, the envelope's actor_id is bound
    /// to this value (cannot be overridden by the payload).
    #[serde(default)]
    pub actor_id: Option<String>,
    /// Optional namespace actor (e.g. "studio") — sets envelope.actor.
    /// Defaults to "ship". Used by Studio's event emitter so subscribers
    /// can route on namespace.
    #[serde(default)]
    pub actor: Option<String>,
    /// Optional elevation flag — only honoured when the auth context permits.
    #[serde(default)]
    pub elevated: Option<bool>,
    /// When true, enforce the agent-domain rules: event_type must contain a
    /// dot and must not start with a reserved platform namespace. Used by
    /// MCP's `event` tool so platform namespaces stay daemon-controlled.
    #[serde(default)]
    pub validate_domain: Option<bool>,
}

/// POST /api/v1/events/emit
#[utoipa::path(
    operation_id = "emitEvent",
    post,
    path = "/api/v1/events/emit",
    request_body = EmitEventReq,
    responses(
        (status = 200, description = "Event persisted and routed", body = MeshResponse),
        (status = 400, description = "Invalid envelope", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn emit_event(
    State(state): State<ApiState>,
    auth: Option<axum::Extension<crate::auth::AuthContext>>,
    Json(body): Json<EmitEventReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {

    if body.validate_domain.unwrap_or(false) {
        if body.event_type.is_empty() {
            return Err(err_response(StatusCode::BAD_REQUEST, "event_type must not be empty"));
        }
        if !body.event_type.contains('.') {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                &format!(
                    "event_type '{}' must be namespaced with a dot (e.g. 'deployment.completed')",
                    body.event_type
                ),
            ));
        }
        for prefix in ship_substrate::events::RESERVED_NAMESPACES {
            if body.event_type.starts_with(prefix) {
                return Err(err_response(
                    StatusCode::BAD_REQUEST,
                    &format!(
                        "event_type '{}' is reserved: '{}' prefix is platform-controlled",
                        body.event_type, prefix
                    ),
                ));
            }
        }
    }

    let mut envelope =
        ship_substrate::events::EventEnvelope::new(&body.event_type, &body.entity_id, &body.payload)
            .map_err(|e| err_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    let actor = body.actor_id.as_deref().unwrap_or("studio");
    envelope = envelope.with_actor_id(actor);
    if let Some(ref ns) = body.actor {
        envelope.actor = ns.clone();
    }
    if let Some(ref ws_id) = body.workspace_id {
        envelope = envelope.with_context(Some(ws_id), None);
    }
    if body.elevated.unwrap_or(false) {
        envelope = envelope.elevate();
    }
    let envelope_id: String = envelope.id.clone();

    // Use auth context if available (token-based), otherwise treat as CLI caller.
    // Studio (localhost browser) is exempted from auth but needs Cli caller kind
    // to emit to system namespaces like studio.*.
    let ctx = if let Some(axum::Extension(ac)) = auth {
        ac.to_daemon_emit_context()
    } else {
        ship_substrate::events::EmitContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            skill_id: None,
            project_id: body.workspace_id.clone(),
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

    Ok(ok_response(json!({ "id": envelope_id })))
}


#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_inline {
    use super::*;
    use axum::extract::{Path, State};

    fn make_test_state() -> (ApiState, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
        let kernel = ship_substrate::events::init_kernel_router(ship_dir.clone()).unwrap();
        let state = std::sync::Arc::new(
            crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel),
        );
        (state, dir)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn nonexistent_workspace_returns_404() {
        let (state, _dir) = make_test_state();
        // Use `list_agents` as a stand-in for a workspace-scoped handler that
        // calls `resolve_worktree` — the git endpoints that previously covered
        // this now live in GitService (`services::git`).
        let response = list_agents(
            State(state),
            Path("nonexistent-branch-xyz".to_string()),
        )
            .await
            .unwrap_err();
        let (status, body) = response;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(!body.0.ok);
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
