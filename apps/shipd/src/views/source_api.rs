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
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    if let Ok(project_ship) = ship_application::project::get_project_dir(None) {
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
    // Try the SUI resolver first — covers the new `views/<name>/page.sui`
    // convention plus the legacy `.ship/views/<name>/index.sui` fallback.
    // Devtools' Source/Schema tabs hit this endpoint for every mounted
    // view; without this branch every SUI view returned 404.
    if let Some(resp) = try_resolve_sui_view(&path) {
        return Ok(Json(resp));
    }

    // Fall through to the legacy `.ship/views/<name>/index.html.j2` lookup.
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

fn try_resolve_sui_view(path: &str) -> Option<ViewSourceResponse> {
    // SUI resolver expects the top-level segment as `name` and the rest
    // as a sub-path. For `view-source/agents` we get path = "agents";
    // for `view-source/agents/abc` we'd get path = "agents/abc". Split.
    let (name, sub) = match path.split_once('/') {
        Some((n, rest)) => (n, Some(rest)),
        None => (path, None),
    };
    let resolved = ship_sui::resolver::resolve(name, sub).ok()?;
    // Bake-list views (agents/git/counter) return source_path = None
    // because the source is `include_str!`'d at compile time. Synthesize
    // the canonical on-disk path so devtools can show + edit. The
    // bake-list mirrors files at `<project_root>/views/<name>/page.sui`.
    let source_path = match resolved.source_path {
        Some(p) => p,
        None => {
            let cwd = std::env::current_dir().ok()?;
            let ship_dir = runtime::project::resolve_project_ship_dir_no_redirect(&cwd)?;
            let project_root = ship_dir.parent()?.to_path_buf();
            project_root.join("views").join(name).join("page.sui")
        }
    };
    let dir = source_path.parent()?;

    // Manifest convention for SUI views: manifest.jsonc next to page.sui.
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
