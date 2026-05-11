//! Project discovery + switching.
//!
//! The launcher / shell consumes this surface to list known projects
//! and pick one. Project records live in the daemon's `runtime::projects`
//! registry (a `kv_state` namespace), keyed by a stable hash of the
//! canonical path. Storage migrated off `<global>/projects.json` —
//! one substrate (kv) instead of two.
//!
//! Active-project tracking lives in the runtime settings store under
//! `shell.project.active`. Subscribers redraw on `ship:project:change`.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::routes::rest_api::ApiState;

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    /// Optional human-friendly label, distinct from `name` (which
    /// defaults to the directory basename).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ProjectAddReq {
    pub path: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ProjectSwitchReq {
    pub id: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

fn record_to_api(record: ship_platform::projects::ProjectRecord, label: Option<String>) -> Project {
    Project {
        id: record.id,
        name: record.name,
        path: record.path,
        label,
        created_at: record.created_at,
        last_seen_at: record.last_seen_at,
    }
}

fn active_id() -> Option<String> {
    ship_platform::config::get_setting("shell.project.active")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
}

#[utoipa::path(
    operation_id = "listProjects",
    get,
    path = "/api/v1/projects",
    responses(
        (status = 200, description = "Known projects + active id", body = ProjectsResponse),
    ),
    tag = "framework"
)]
pub async fn list_projects(State(_state): State<ApiState>) -> axum::response::Response {
    match ship_platform::projects::list() {
        Ok(records) => {
            let projects = records
                .into_iter()
                .map(|r| record_to_api(r, None))
                .collect();
            (
                StatusCode::OK,
                Json(ProjectsResponse {
                    projects,
                    active: active_id(),
                }),
            )
                .into_response()
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

#[utoipa::path(
    operation_id = "addProject",
    post,
    path = "/api/v1/projects",
    request_body = ProjectAddReq,
    responses(
        (status = 200, description = "Project added", body = ProjectsResponse),
        (status = 400, description = "Invalid path"),
    ),
    tag = "framework"
)]
pub async fn add_project(
    State(_state): State<ApiState>,
    Json(req): Json<ProjectAddReq>,
) -> axum::response::Response {
    let path = std::path::PathBuf::from(req.path.trim());
    if !path.is_dir() {
        return err(
            StatusCode::BAD_REQUEST,
            format!("not a directory: {}", path.display()),
        );
    }
    // Ensure the project's `.ship/` exists so the daemon has a place to
    // store project-scoped config when the user starts authoring.
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(&path);
    if !ship_dir.exists()
        && let Err(e) = std::fs::create_dir_all(&ship_dir)
    {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    if let Err(e) = ship_platform::projects::register(&path, None) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    // Re-list so the response includes the new record + everything else.
    match ship_platform::projects::list() {
        Ok(records) => {
            let projects = records
                .into_iter()
                .map(|r| record_to_api(r, req.label.clone()))
                .collect();
            (
                StatusCode::OK,
                Json(ProjectsResponse {
                    projects,
                    active: active_id(),
                }),
            )
                .into_response()
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

#[utoipa::path(
    operation_id = "switchProject",
    post,
    path = "/api/v1/projects/switch",
    request_body = ProjectSwitchReq,
    responses(
        (status = 200, description = "Active project updated"),
        (status = 404, description = "Unknown project id"),
    ),
    tag = "framework"
)]
pub async fn switch_project(
    State(_state): State<ApiState>,
    Json(req): Json<ProjectSwitchReq>,
) -> axum::response::Response {
    match ship_platform::projects::get(&req.id) {
        Ok(Some(_)) => {}
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                format!("project '{}' not known", req.id),
            );
        }
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
    if let Err(e) =
        ship_platform::config::set_setting("shell.project.active", serde_json::json!(req.id))
    {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    // Bump last_seen on switch so the rail's recency order reflects use.
    let _ = ship_platform::projects::touch(&req.id);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true, "active": req.id })),
    )
        .into_response()
}
