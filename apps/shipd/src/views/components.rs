//! Component registration + serve handler for `.ship/components/*.ship`.
//!
//! Each `.ship/components/<tag>.ship` file compiles to a self-contained
//! ES module served at `GET /_ship/components/<tag>.js`. The compiled
//! output is cached in-process keyed on the source-file `(path, mtime,
//! size)` triple, so an editor save invalidates automatically on the
//! next request without touching watchers.
//!
//! Resolution mirrors the `templates::collect_siblings` rule:
//! - project `.ship/components/` first
//! - global `~/.ship/components/` as fallback
//!
//! Pages embed components by adding
//! `<script type="module" src="/_ship/components/app-typeahead.js"></script>`
//! to the page head (or relying on the SDK's auto-loader, future work).
//! The script registers `customElements.define(...)` and from then on
//! every `<app-typeahead>` instance on the page is live.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::SystemTime;

use super::sfc::{compile_component, tag_from_stem};

#[derive(Debug, Clone)]
struct CacheEntry {
    /// Source-file mtime + len; together a cheap freshness signature.
    mtime: SystemTime,
    len: u64,
    /// Compiled JS module string.
    js: String,
}

static CACHE: OnceLock<RwLock<HashMap<String, CacheEntry>>> = OnceLock::new();

fn cache() -> &'static RwLock<HashMap<String, CacheEntry>> {
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// `GET /_ship/components/{tag}.js`
///
/// Returns the compiled custom-element module for `tag`. The path
/// param **includes the `.js` suffix** because axum routes on the
/// literal path; we strip the suffix here to recover the tag.
pub async fn serve_component_js(Path(file): Path<String>) -> Response {
    let tag = match file.strip_suffix(".js") {
        Some(t) => t.to_string(),
        None => return not_found("expected .js suffix"),
    };
    if tag_from_stem(&tag).is_none() {
        return not_found("invalid tag");
    }
    let path = match resolve_component_path(&tag) {
        Some(p) => p,
        None => return not_found(&format!("component not found: {tag}")),
    };

    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return server_error(&format!("stat: {e}")),
    };
    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let len = meta.len();

    if let Ok(c) = cache().read()
        && let Some(entry) = c.get(&tag)
        && entry.mtime == mtime
        && entry.len == len
    {
        return js_response(entry.js.clone());
    }

    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return server_error(&format!("read: {e}")),
    };
    let template_name = format!("components/{tag}.ship");
    let js = match compile_component(&tag, &template_name, &source) {
        Ok(s) => s,
        Err(e) => return server_error(&format!("compile {tag}: {e}")),
    };
    if let Ok(mut c) = cache().write() {
        c.insert(
            tag.clone(),
            CacheEntry { mtime, len, js: js.clone() },
        );
    }
    js_response(js)
}

/// Locate `<tag>.ship` in the project's `.ship/components/` (preferred)
/// or the global `~/.ship/components/` directory.
pub fn resolve_component_path(tag: &str) -> Option<std::path::PathBuf> {
    let filename = format!("{tag}.ship");
    for dir in [
        runtime::project::get_project_dir(None).ok(),
        runtime::project::get_global_dir().ok(),
    ]
    .into_iter()
    .flatten()
    {
        let path = dir.join("components").join(&filename);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Eagerly enumerate every `.ship/components/*.ship` file across project
/// and global dirs, returning `(tag, path)` pairs. Used at daemon
/// startup for log-only confirmation; the actual cache is lazy
/// (populated on first request per tag).
pub fn discover_components() -> Vec<(String, std::path::PathBuf)> {
    let mut out: Vec<(String, std::path::PathBuf)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for dir in [
        runtime::project::get_project_dir(None).ok(),
        runtime::project::get_global_dir().ok(),
    ]
    .into_iter()
    .flatten()
    {
        let comp_dir = dir.join("components");
        let Ok(entries) = std::fs::read_dir(&comp_dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            let Some(ext) = path.extension().and_then(|s| s.to_str()) else { continue };
            if ext != "ship" { continue; }
            let Some(tag) = tag_from_stem(stem) else { continue };
            if seen.insert(tag.clone()) {
                out.push((tag, path));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn js_response(body: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        // No cache while we're iterating on the format. mtime check on
        // the daemon side is cheap; the browser's HTTP cache here would
        // only mask hot-reload behavior.
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

fn server_error(msg: &str) -> Response {
    tracing::warn!(error = %msg, "component compile/serve failed");
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("server error")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_empty_when_no_components_dir() {
        // Smoke test — we don't control the test scratch dir's contents,
        // but the function must not panic on a missing dir.
        let _ = discover_components();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn serve_invalid_tag_returns_404() {
        let resp = serve_component_js(Path("bogus.js".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn serve_missing_suffix_returns_404() {
        let resp = serve_component_js(Path("app-typeahead".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
