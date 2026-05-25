//! `GET /api/v1/view-source/{*path}` — return template + manifest source
//! for the SUI view that matches `path`.
//!
//! Used by the shell's Source devtools tab to inspect the currently
//! mounted view's on-disk source.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::ship_runtime::ApiState;

#[derive(Serialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewSourceResponse {
    /// Dotted view name derived from the filesystem path.
    pub view_name: String,
    /// Absolute filesystem path to the matched template.
    pub template_path: String,
    /// Template file contents (`page.sui`).
    pub template: String,
    /// Filename of the manifest that was found next to the template
    /// (or in an ancestor view root). `None` when no manifest exists.
    pub manifest_filename: Option<String>,
    /// Raw manifest file contents. `None` when no manifest exists.
    pub manifest: Option<String>,
    /// Scope the view was resolved from. Always `sui` post-j2-deletion.
    pub scope: String,
}

#[utoipa::path(
    operation_id = "serveViewSource",
    get,
    path = "/api/v1/view-source/{path}",
    params(("path" = String, Path, description = "View path; matches a SUI view name")),
    responses(
        (status = 200, description = "Template + manifest source", body = ViewSourceResponse),
        (status = 404, description = "No matching view"),
    ),
    tag = "framework"
)]
pub async fn serve_view_source(
    State(_state): State<ApiState>,
    Path(path): Path<String>,
) -> Result<Json<ViewSourceResponse>, StatusCode> {
    try_resolve_sui_view(&path)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

fn try_resolve_sui_view(path: &str) -> Option<ViewSourceResponse> {
    let (name, sub) = match path.split_once('/') {
        Some((n, rest)) => (n, Some(rest)),
        None => (path, None),
    };
    let resolved = deck::resolver::resolve(name, sub).ok()?;
    // Bake-list views return source_path = None because the source is
    // `include_str!`'d at compile time. Synthesize the canonical on-disk
    // path so devtools can show + edit.
    let source_path = match resolved.source_path {
        Some(p) => p,
        None => {
            let cwd = std::env::current_dir().ok()?;
            let ship_dir = ship_substrate::project::resolve_project_ship_dir(&cwd)?;
            let project_root = ship_dir.parent()?.to_path_buf();
            project_root.join("views").join(name).join("page.sui")
        }
    };
    let dir = source_path.parent()?;

    let (manifest_filename, manifest) = ["manifest.jsonc", "manifest.yaml", "manifest.json"]
        .iter()
        .find_map(|fname| {
            let p = dir.join(fname);
            std::fs::read_to_string(&p)
                .ok()
                .map(|body| (Some((*fname).to_string()), Some(body)))
        })
        .unwrap_or((None, None));

    Some(ViewSourceResponse {
        view_name: name.to_string(),
        template_path: source_path.display().to_string(),
        template: resolved.source,
        manifest_filename,
        manifest,
        scope: "sui".to_string(),
    })
}
