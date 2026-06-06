//! Static file handler for the shell's `public/` assets.
//!
//! The shell's `index.html` references `/ship-logo.svg`, `/favicon.ico`,
//! and a few `/favicon.*` paths at root. None of these are served by
//! the deck scaffold's `serve_scaffold_asset` handler (which reads
//! from `.deck/hub-scaffold/assets/<file>`), so they 404 in dev.
//! This handler serves a small allowlist of root-relative paths
//! from the shell's source tree so the shell paints without 404s.
//!
//! The allowlist is small on purpose: a permissive static handler
//! is a directory-traversal surface. Only specific filenames and
//! content-types are allowed; everything else returns 404.

use std::path::PathBuf;

use axum::extract::Path;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use deck::DaemonMode;
use ship_substrate::home::worktree_root_for;

/// Map of allowed root-relative static files served by the shell.
/// (filename → content-type). Adding a new entry here is the only
/// way to add a new root-relative static asset; this is the SSOT
/// for what root paths the shell can serve.
fn content_type_for(file: &str) -> Option<&'static str> {
    match file {
        "ship-logo.svg" => Some("image/svg+xml; charset=utf-8"),
        "ship-logo-secondary.svg" => Some("image/svg+xml; charset=utf-8"),
        "favicon.svg" => Some("image/svg+xml; charset=utf-8"),
        "favicon.ico" => Some("image/x-icon"),
        _ => None,
    }
}

/// Resolve a static filename to the on-disk path under
/// `<shell_root>/public/`. Returns None if the filename is not
/// in the allowlist or the file doesn't exist on disk.
fn resolve_static_path(shell_root: &PathBuf, file: &str) -> Option<PathBuf> {
    if content_type_for(file).is_none() {
        return None;
    }
    let p = shell_root.join("public").join(file);
    if p.is_file() { Some(p) } else { None }
}

/// `GET /<file>` — serve the shell's `public/<file>` if it's in
/// the allowlist. Whitelist-based: a file not in
/// `content_type_for` is a 404, even if it exists on disk.
pub async fn serve_shell_static(
    Path(file): Path<String>,
    mode: axum::extract::Extension<DaemonMode>,
) -> Response {
    // Reject path traversal. The allowlist catches most cases but
    // belt-and-suspenders against `/..` or `/foo/bar`.
    if file.contains('/') || file.contains('\\') || file.contains("..") || file.starts_with('.') {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let Some(content_type) = content_type_for(&file) else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let cwd = std::env::current_dir().unwrap_or_default();
    let Some(shell_root) = worktree_root_for(&cwd).map(|r| r.join("apps").join("shell")) else {
        return (StatusCode::NOT_FOUND, "no shell root").into_response();
    };
    let Some(path) = resolve_static_path(&shell_root, &file) else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let body = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "read failed").into_response(),
    };
    let mut resp = (StatusCode::OK, body).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(content_type),
    );
    if matches!(mode.0, DaemonMode::Production) {
        resp.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    } else {
        resp.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-cache"),
        );
    }
    resp
}
