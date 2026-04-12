//! Git endpoints — status, diff, log for workspace worktrees and project root.

use axum::{Json, extract::{Path, Query, State}, http::StatusCode};
use serde::Deserialize;

use crate::rest_api::{ApiState, MeshResponse};

use super::{err_response, ok_response, resolve_worktree};

fn run_git(
    worktree: &std::path::Path,
    args: &[&str],
) -> Result<String, (StatusCode, Json<MeshResponse>)> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(worktree)
        .output()
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to run git: {e}"),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("git error: {stderr}"),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// GET /api/workspaces/{id}/git/status
pub async fn git_status(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let output = run_git(&worktree, &["status"])?;
    Ok(ok_response(serde_json::json!({ "output": output })))
}

#[derive(Deserialize)]
pub struct DiffQuery {
    pub range: Option<String>,
}

/// GET /api/workspaces/{id}/git/diff
pub async fn git_diff(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(params): Query<DiffQuery>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let output = match &params.range {
        Some(range) => run_git(&worktree, &["diff", range])?,
        None => run_git(&worktree, &["diff"])?,
    };
    Ok(ok_response(serde_json::json!({ "output": output })))
}

#[derive(Deserialize)]
pub struct LogQuery {
    pub limit: Option<u32>,
}

/// GET /api/workspaces/{id}/git/log
pub async fn git_log(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Query(params): Query<LogQuery>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let limit = params.limit.unwrap_or(20);
    let limit_arg = format!("-{limit}");
    let output = run_git(
        &worktree,
        &[
            "log",
            "--format=%H%x1f%s%x1f%an%x1f%ad",
            "--date=short",
            &limit_arg,
        ],
    )?;

    let commits: Vec<serde_json::Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() >= 4 {
                Some(serde_json::json!({
                    "hash": parts[0],
                    "subject": parts[1],
                    "author": parts[2],
                    "date": parts[3],
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(ok_response(serde_json::json!({
        "output": output,
        "commits": commits,
    })))
}

// ── Project-level endpoints (run against project root, not a workspace) ──

fn resolve_project_root(
    ship_dir: &std::path::Path,
) -> Result<std::path::PathBuf, (StatusCode, Json<MeshResponse>)> {
    // Walk CWD up to find a real project's .ship/. Follows git worktree
    // redirects to the main checkout. This is the correct source — NOT
    // ship_dir.parent(), which on the daemon is ~/ (since ship_dir = ~/.ship).
    if let Ok(project_ship) = runtime::project::get_project_dir(None) {
        if let Some(root) = project_ship.parent() {
            return Ok(root.to_path_buf());
        }
    }
    // Fallback for when ship_dir IS a project .ship/ (e.g. tests).
    let root = ship_dir.parent().ok_or_else(|| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, "cannot resolve project root")
    })?;
    Ok(root.to_path_buf())
}

/// GET /api/project/git/status
pub async fn project_git_status(
    State(state): State<ApiState>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let root = resolve_project_root(state.ship_dir())?;
    let output = run_git(&root, &["status"])?;
    Ok(ok_response(serde_json::json!({ "output": output })))
}

/// GET /api/project/git/diff
pub async fn project_git_diff(
    State(state): State<ApiState>,
    Query(params): Query<DiffQuery>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let root = resolve_project_root(state.ship_dir())?;
    let output = match &params.range {
        Some(range) => run_git(&root, &["diff", range])?,
        None => run_git(&root, &["diff"])?,
    };
    Ok(ok_response(serde_json::json!({ "output": output })))
}

/// GET /api/project/git/log
pub async fn project_git_log(
    State(state): State<ApiState>,
    Query(params): Query<LogQuery>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let root = resolve_project_root(state.ship_dir())?;
    let limit = params.limit.unwrap_or(20);
    let limit_arg = format!("-{limit}");
    let output = run_git(
        &root,
        &[
            "log",
            "--format=%H%x1f%s%x1f%an%x1f%ad",
            "--date=short",
            &limit_arg,
        ],
    )?;

    let commits: Vec<serde_json::Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() >= 4 {
                Some(serde_json::json!({
                    "hash": parts[0],
                    "subject": parts[1],
                    "author": parts[2],
                    "date": parts[3],
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(ok_response(serde_json::json!({
        "output": output,
        "commits": commits,
    })))
}
