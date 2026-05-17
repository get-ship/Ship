//! `GET /api/v1/views/{name}/files` — list every textual file under a
//! view's source directory + its `[id]` subroutes, with content inlined
//! for files under 100kb.
//!
//! The dev-panel Source tab uses this to populate a file explorer +
//! mount any selected file in a single editor. Pre-fix the tab returned
//! only `page.sui` + `manifest.jsonc` stacked.
//!
//! Resolution mirrors `source_api`: look up the view dir via the SUI
//! resolver, then walk top-level + one nested `[id]/` level.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use std::path::PathBuf;

use crate::ship_runtime::ApiState;

const MAX_INLINE_BYTES: u64 = 100 * 1024;

/// Files we surface in the explorer. Anything else is filtered.
const ALLOWED_EXT: &[&str] = &["sui", "ts", "tsx", "css", "jsonc", "json", "yaml", "yml", "md"];

#[derive(Serialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewFile {
    /// Relative path from the view root (e.g. `page.sui`,
    /// `[id]/page.sui`, `controller.ts`). Forward slashes regardless
    /// of platform.
    pub path: String,
    /// Filename only.
    pub name: String,
    /// Lower-case extension (without the dot). Drives editor language.
    pub kind: String,
    pub size: u64,
    /// File body for files under MAX_INLINE_BYTES; None for larger
    /// files (the dev panel can fall back to a separate fetch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewFilesResponse {
    pub view_name: String,
    pub view_dir: String,
    pub files: Vec<ViewFile>,
}

#[utoipa::path(
    operation_id = "listViewFiles",
    get,
    path = "/api/v1/views/{name}/files",
    params(("name" = String, Path, description = "View name")),
    responses(
        (status = 200, description = "Files under the view directory", body = ViewFilesResponse),
        (status = 404, description = "View not found"),
    ),
    tag = "framework"
)]
pub async fn list_view_files(
    State(_state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<ViewFilesResponse>, StatusCode> {
    let view_dir = resolve_view_dir(&name).ok_or(StatusCode::NOT_FOUND)?;
    let mut files = Vec::new();
    collect_dir(&view_dir, "", &mut files);
    // Sort: page.sui first, then alphabetic
    files.sort_by(|a, b| {
        let a_top = a.name == "page.sui" && !a.path.contains('/');
        let b_top = b.name == "page.sui" && !b.path.contains('/');
        match (a_top, b_top) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        }
    });
    Ok(Json(ViewFilesResponse {
        view_name: name,
        view_dir: view_dir.display().to_string(),
        files,
    }))
}

fn resolve_view_dir(name: &str) -> Option<PathBuf> {
    let resolved = ship_sui::resolver::resolve(name, None).ok()?;
    if let Some(p) = resolved.source_path {
        return p.parent().map(PathBuf::from);
    }
    let cwd = std::env::current_dir().ok()?;
    let ship_dir = ship_substrate::project::resolve_project_ship_dir(&cwd)?;
    let project_root = ship_dir.parent()?.to_path_buf();
    let dir = project_root.join("views").join(name);
    if dir.is_dir() { Some(dir) } else { None }
}

fn collect_dir(root: &std::path::Path, rel_prefix: &str, out: &mut Vec<ViewFile>) {
    let dir = if rel_prefix.is_empty() { root.to_path_buf() } else { root.join(rel_prefix) };
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let rel = if rel_prefix.is_empty() {
            file_name.clone()
        } else {
            format!("{rel_prefix}/{file_name}")
        };
        if path.is_dir() {
            // Only walk subroute-shaped dirs ([id], [slug], ...) and
            // only one level deep — that's the convention for nested
            // routes. Avoid descending into node_modules-shaped dirs.
            if file_name.starts_with('[') && file_name.ends_with(']') && rel_prefix.is_empty() {
                collect_dir(root, &rel, out);
            }
            continue;
        }
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if !ALLOWED_EXT.contains(&ext.as_str()) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = meta.len();
        let content = if size <= MAX_INLINE_BYTES {
            std::fs::read_to_string(&path).ok()
        } else {
            None
        };
        out.push(ViewFile {
            path: rel,
            name: file_name,
            kind: ext,
            size,
            content,
        });
    }
}
