//! Write endpoints for skill files / vars under workspace-scoped `.ship/`.
//!
//! Mutations resolve the worktree first, then delegate to
//! `ship_platform::services::skills::state` (file ops) or
//! `ship_platform::agents::skill_vars` (per-skill vars). No raw `std::fs`
//! calls in this layer; path-safety + JSONC handling live in platform.
//!
//! Phase 16 removed the legacy `POST /api/v1/projects/{id}/agents`
//! handler. It had no callers, its request shape predated the canonical
//! AgentProfile (Phase 8), and writes produced files the canonical
//! loader silently rejected. Re-introduce a canonical-shape variant
//! when there's an actual caller for it.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::{ApiState, MeshResponse};
use super::{err_response, ok_response};

use ship_platform::services::files::{FilesService, Scope as FsScope};
use ship_platform::services::skills::state as skills_state;

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WriteFileReq {
    pub content: String,
}

#[derive(Deserialize)]
pub struct SetVarReq {
    pub value_json: String,
}

// ── Endpoints ─────────────────────────────────────────────────────────────────

/// PUT /api/v1/projects/{id}/skills/{skillId}/files/{*path}
pub async fn write_skill_file(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, skill_id, file_path)): Path<(String, String, String)>,
    Json(body): Json<WriteFileReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = state.resolve_worktree(&id)?;
    let files = FilesService::resolve_roots(
        &ship_substrate::ShipHome::project_ship_dir(&worktree),
    );
    skills_state::write_file(
        &files,
        &skill_id,
        Some(FsScope::Project),
        &file_path,
        body.content.as_bytes(),
    )
    .map_err(skills_err_to_http)?;
    Ok(ok_response(json!({})))
}

/// DELETE /api/v1/projects/{id}/skills/{skillId}/files/{*path}
pub async fn delete_skill_file(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, skill_id, file_path)): Path<(String, String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = state.resolve_worktree(&id)?;
    let files = FilesService::resolve_roots(
        &ship_substrate::ShipHome::project_ship_dir(&worktree),
    );
    skills_state::delete_file(&files, &skill_id, Some(FsScope::Project), &file_path)
        .map_err(skills_err_to_http)?;
    Ok(ok_response(json!({})))
}

/// GET /api/v1/projects/{id}/skills/{skillId}/vars
pub async fn get_skill_vars(
    State(state): State<ApiState>,
    Path((id, skill_id)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = state.resolve_worktree(&id)?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(&worktree);

    validate_skill_id(&skill_id)?;

    let vars = ship_platform::agents::skill_vars::get_skill_vars(&ship_dir, &skill_id)
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to read skill vars: {e}"),
            )
        })?;

    match vars {
        Some(map) => {
            let vars_json: serde_json::Map<String, serde_json::Value> =
                map.into_iter().collect();
            Ok(ok_response(json!({ "vars": vars_json })))
        }
        None => Ok(ok_response(json!({ "vars": null }))),
    }
}

/// PUT /api/v1/projects/{id}/skills/{skillId}/vars/{key}
pub async fn set_skill_var(
    State(state): State<ApiState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
    Path((id, skill_id, key)): Path<(String, String, String)>,
    Json(body): Json<SetVarReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    require_scope(&auth, Scope::WriteWorkspace)?;
    let worktree = state.resolve_worktree(&id)?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(&worktree);

    validate_skill_id(&skill_id)?;

    let value: serde_json::Value =
        serde_json::from_str(&body.value_json).map_err(|e| {
            err_response(
                StatusCode::BAD_REQUEST,
                &format!("value_json is not valid JSON: {e}"),
            )
        })?;

    ship_platform::agents::skill_vars::set_skill_var(&ship_dir, &skill_id, &key, value).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unknown variable") || msg.contains("no assets/vars.json") {
            err_response(StatusCode::NOT_FOUND, &msg)
        } else {
            err_response(StatusCode::INTERNAL_SERVER_ERROR, &msg)
        }
    })?;

    Ok(ok_response(json!({})))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn validate_skill_id(skill_id: &str) -> Result<(), (StatusCode, Json<MeshResponse>)> {
    if skill_id.is_empty() || skill_id.contains('/') || skill_id.contains("..") {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid skill id"));
    }
    Ok(())
}

fn skills_err_to_http(
    e: ship_platform::services::skills::state::Error,
) -> (StatusCode, Json<MeshResponse>) {
    use ship_platform::services::skills::state::Error;
    match e {
        Error::NotFound(msg) => err_response(StatusCode::NOT_FOUND, &msg),
        Error::BadId(msg) | Error::BadPath(msg) => err_response(StatusCode::BAD_REQUEST, &msg),
        Error::AlreadyExists(msg) => err_response(StatusCode::CONFLICT, &msg),
        _ => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn set_var_req_parses_string_value_json() {
        let req: SetVarReq = serde_json::from_str(r#"{"value_json": "\"gitmoji\""}"#).unwrap();
        let val: serde_json::Value = serde_json::from_str(&req.value_json).unwrap();
        assert_eq!(val, json!("gitmoji"));
    }

    #[test]
    fn set_var_req_parses_bool_value_json() {
        let req: SetVarReq = serde_json::from_str(r#"{"value_json": "true"}"#).unwrap();
        let val: serde_json::Value = serde_json::from_str(&req.value_json).unwrap();
        assert_eq!(val, json!(true));
    }
}
