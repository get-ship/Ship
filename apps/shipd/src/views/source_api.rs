//! `GET /api/v1/view-source/{*path}` — return template + manifest source
//! for the file-routed view that matches `path`.
//!
//! Used by the shell's Source devtools tab to inspect the currently
//! mounted view's on-disk source.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::rest_api::ApiState;
use crate::views::router::match_path;

#[derive(Serialize, utoipa::ToSchema)]
pub struct ViewSourceResponse {
    /// Dotted view name derived from the filesystem path.
    pub view_name: String,
    /// Absolute filesystem path to the matched template.
    pub template_path: String,
    /// Template file contents (`index.html.j2`).
    pub template: String,
    /// Filename of the manifest that was found next to the template
    /// (or in an ancestor view root). `None` when no manifest exists.
    pub manifest_filename: Option<String>,
    /// Raw manifest file contents. `None` when no manifest exists.
    pub manifest: Option<String>,
    /// Scope the view was resolved from: `project` | `global`. Bundled
    /// first-party views live in the user-global `.ship/views/` today so
    /// surface `global` here; `bundled` is reserved for a later policy
    /// pass (editability tiers).
    pub scope: String,
}

fn view_roots(state: &ApiState) -> Vec<(std::path::PathBuf, &'static str)> {
    let mut roots: Vec<(std::path::PathBuf, &'static str)> = Vec::new();
    if let Ok(project_ship) = runtime::project::get_project_dir(None) {
        let p = project_ship.join("views");
        if project_ship != state.ship_dir() && p.is_dir() {
            roots.push((p, "project"));
        }
    }
    let user = state.ship_dir().join("views");
    if user.is_dir() {
        roots.push((user, "global"));
    }
    roots
}

fn find_manifest(
    template: &std::path::Path,
    roots: &[std::path::PathBuf],
) -> Option<std::path::PathBuf> {
    let mut dir = template.parent();
    while let Some(d) = dir {
        if !roots.iter().any(|r| d.starts_with(r) || d == r.as_path()) {
            break;
        }
        for fname in ["manifest.jsonc", "manifest.yaml", "manifest.json"] {
            let p = d.join(fname);
            if p.is_file() {
                return Some(p);
            }
        }
        dir = d.parent();
    }
    None
}

#[utoipa::path(
    operation_id = "serveViewSource",
    get,
    path = "/api/v1/view-source/{path}",
    params(("path" = String, Path, description = "View path; matches /api/v1/v/{path}")),
    responses(
        (status = 200, description = "Template + manifest source", body = ViewSourceResponse),
        (status = 404, description = "No matching view"),
    ),
    tag = "framework"
)]
pub async fn serve_view_source(
    State(state): State<ApiState>,
    Path(path): Path<String>,
) -> Result<Json<ViewSourceResponse>, StatusCode> {
    let roots = view_roots(&state);
    let (rmatch, scope) = roots
        .iter()
        .find_map(|(r, s)| match_path(r, &path).map(|m| (m, *s)))
        .ok_or(StatusCode::NOT_FOUND)?;

    let template =
        std::fs::read_to_string(&rmatch.template).map_err(|_| StatusCode::NOT_FOUND)?;

    let root_paths: Vec<std::path::PathBuf> =
        roots.iter().map(|(p, _)| p.clone()).collect();
    let manifest_path = find_manifest(&rmatch.template, &root_paths);
    let (manifest_filename, manifest) = match manifest_path {
        Some(p) => {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .map(String::from);
            let body = std::fs::read_to_string(&p).ok();
            (name, body)
        }
        None => (None, None),
    };

    Ok(Json(ViewSourceResponse {
        view_name: rmatch.view_name,
        template_path: rmatch.template.display().to_string(),
        template,
        manifest_filename,
        manifest,
        scope: scope.to_string(),
    }))
}
