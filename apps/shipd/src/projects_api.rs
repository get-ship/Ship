//! Project discovery + switching.
//!
//! Ship stores its user-global state in `~/.ship/` by default, with
//! optional project-scoped state in `<project>/.ship/`. Users may
//! have multiple project directories on disk; the daemon keeps a
//! list of known ones in `<global>/projects.json` so the rail
//! switcher can show them.
//!
//! The "active project" concept is lightweight: it's just the
//! current working directory the daemon resolves project-scoped
//! resources against. Switching persists to the settings store as
//! `shell.project.active` — consumers re-query their data when
//! they see `ship:project:change`.
//!
//! Scope for this commit: list + upsert + switch. File-system
//! discovery (auto-find `.ship/` dirs in common roots) is a
//! follow-up.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::rest_api::ApiState;

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
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

fn projects_file(state: &ApiState) -> PathBuf {
    state.ship_dir().join("projects.json")
}

fn project_id_for(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string()
}

fn read_projects(state: &ApiState) -> Vec<Project> {
    let f = projects_file(state);
    std::fs::read_to_string(&f)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_projects(state: &ApiState, list: &[Project]) -> std::io::Result<()> {
    let f = projects_file(state);
    if let Some(parent) = f.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = serde_json::to_string_pretty(list).unwrap_or_else(|_| "[]".into());
    std::fs::write(&f, body)
}

fn err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

/// Ensure the daemon's current project (if any) is in the list. Returns the
/// updated list. Idempotent; safe to call on every list read.
fn ensure_current(state: &ApiState, mut list: Vec<Project>) -> Vec<Project> {
    if let Ok(current) = std::env::current_dir() {
        // Only track if it contains a .ship dir.
        if runtime::ShipHome::project_ship_dir(&current).is_dir() {
            let id = project_id_for(&current);
            if !list.iter().any(|p| p.id == id) {
                list.push(Project {
                    id: id.clone(),
                    name: id,
                    path: current.display().to_string(),
                    label: None,
                });
                let _ = write_projects(state, &list);
            }
        }
    }
    list
}

fn active_id(_state: &ApiState) -> Option<String> {
    runtime::get_setting("shell.project.active")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| project_id_for(&p))
        })
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
pub async fn list_projects(State(state): State<ApiState>) -> axum::response::Response {
    let list = ensure_current(&state, read_projects(&state));
    (
        StatusCode::OK,
        Json(ProjectsResponse {
            projects: list,
            active: active_id(&state),
        }),
    )
        .into_response()
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
    State(state): State<ApiState>,
    Json(req): Json<ProjectAddReq>,
) -> axum::response::Response {
    let path = std::path::PathBuf::from(req.path.trim());
    if !path.is_dir() {
        return err(StatusCode::BAD_REQUEST, format!("not a directory: {}", path.display()));
    }
    // Require .ship dir OR create one quietly.
    let ship_dir = runtime::ShipHome::project_ship_dir(&path);
    if !ship_dir.exists()
        && let Err(e) = std::fs::create_dir_all(&ship_dir) {
            return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    let id = project_id_for(&path);
    let mut list = read_projects(&state);
    if let Some(existing) = list.iter_mut().find(|p| p.id == id) {
        existing.path = path.display().to_string();
        if req.label.is_some() {
            existing.label = req.label;
        }
    } else {
        list.push(Project {
            id: id.clone(),
            name: id,
            path: path.display().to_string(),
            label: req.label,
        });
    }
    if let Err(e) = write_projects(&state, &list) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    (
        StatusCode::OK,
        Json(ProjectsResponse {
            projects: list,
            active: active_id(&state),
        }),
    )
        .into_response()
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
    State(state): State<ApiState>,
    Json(req): Json<ProjectSwitchReq>,
) -> axum::response::Response {
    let list = ensure_current(&state, read_projects(&state));
    if !list.iter().any(|p| p.id == req.id) {
        return err(StatusCode::NOT_FOUND, format!("project '{}' not known", req.id));
    }
    if let Err(e) = runtime::set_setting(
        "shell.project.active",
        serde_json::json!(req.id),
    ) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true, "active": req.id })),
    )
        .into_response()
}
