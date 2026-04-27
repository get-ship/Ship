//! Compile + serve handlers for `.sail` single-file components.
//!
//! Mirrors `views::components` (which serves `.ship` files), but
//! plugs in the `sail-compiler` crate. Each `.sail` file in the
//! project's `.ship/sail/` directory (or the global fallback)
//! compiles on demand and is served as four artifacts:
//!
//! - `GET /_sail/components/<tag>.js`     — runtime ES module
//! - `GET /_sail/components/<tag>.html`   — initial-paint SSR HTML
//! - `GET /_sail/components/<tag>.d.ts`   — TypeScript declaration
//! - `GET /_sail/components/<tag>.json`   — public JSON schema
//!
//! Cached in-process keyed on the source file's `(mtime, len)`. An
//! editor save invalidates automatically on the next request.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::time::SystemTime;

use sail_compiler::{CompileOpts, CompileResult, compile_source};

#[derive(Debug, Clone)]
struct CacheEntry {
    mtime: SystemTime,
    len: u64,
    js: String,
    ssr: String,
    dts: String,
    schema: String,
}

static CACHE: OnceLock<RwLock<HashMap<String, CacheEntry>>> = OnceLock::new();

fn cache() -> &'static RwLock<HashMap<String, CacheEntry>> {
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// `GET /_sail/demo` — visual showcase of the parallel ship-set-sail
/// ports. Static HTML; references compiled .sail modules at
/// `/_sail/components/<tag>.js` (so author edits hot-reload via the
/// existing pipeline). Requires the .sail sources to be discoverable
/// at `.ship/sail/` — run `just sail-demo` to link them.
pub async fn serve_sail_demo() -> Response {
    const DEMO_HTML: &str =
        include_str!("../../../../packages/ship-set-sail/demo/index.html");
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(DEMO_HTML))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /sail/{tag}` — full HTML page that mounts `<tag>` with the
/// SSR'd initial paint, the runtime ES module, and the HMR client.
pub async fn serve_sail_page(Path(tag): Path<String>) -> Response {
    if !is_valid_tag(&tag) {
        return not_found("invalid tag");
    }
    let path = match resolve_sail_path(&tag) {
        Some(p) => p,
        None => return not_found(&format!("sail component not found: {tag}")),
    };
    let entry = match load_or_compile(&tag, &path) {
        Ok(e) => e,
        Err(e) => return server_error(&format!("compile {tag}: {e}")),
    };
    let html = format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n  <title>{tag}</title>\n</head>\n<body>\n{ssr}\n  <script type=\"module\" src=\"/_sail/components/{tag}.js\"></script>\n  <script src=\"/_sail/hmr-client.js\" defer></script>\n</body>\n</html>\n",
        ssr = entry.ssr,
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(html))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /_sail/components/{file}` — `file` is `<tag>.<ext>` where
/// `ext` selects which artifact to serve.
pub async fn serve_sail(Path(file): Path<String>) -> Response {
    let (tag, ext) = match split_tag_ext(&file) {
        Some(t) => t,
        None => return not_found("expected <tag>.{js,html,d.ts,json}"),
    };
    if !is_valid_tag(&tag) {
        return not_found("invalid tag");
    }
    let path = match resolve_sail_path(&tag) {
        Some(p) => p,
        None => return not_found(&format!("sail component not found: {tag}")),
    };

    let entry = match load_or_compile(&tag, &path) {
        Ok(e) => e,
        Err(e) => return server_error(&format!("compile {tag}: {e}")),
    };

    match ext {
        "js" => respond(&entry.js, "application/javascript; charset=utf-8"),
        "html" => respond(&entry.ssr, "text/html; charset=utf-8"),
        "d.ts" => respond(&entry.dts, "text/plain; charset=utf-8"),
        "json" => respond(&entry.schema, "application/json; charset=utf-8"),
        _ => not_found("unsupported extension"),
    }
}

fn load_or_compile(tag: &str, path: &PathBuf) -> Result<CacheEntry, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("stat: {e}"))?;
    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let len = meta.len();

    if let Ok(c) = cache().read()
        && let Some(entry) = c.get(tag)
        && entry.mtime == mtime
        && entry.len == len
    {
        return Ok(entry.clone());
    }

    let source = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
    let opts = CompileOpts::default();
    let CompileResult { js, ssr, dts, schema, diagnostics: _ } =
        compile_source(&source, tag).map_err(|e| format!("{e:#}"))?;
    let entry = CacheEntry { mtime, len, js, ssr, dts, schema };
    if let Ok(mut c) = cache().write() {
        c.insert(tag.to_string(), entry.clone());
    }
    Ok(entry)
}

/// Locate `<tag>.sail` in the project's `.ship/sail/` (preferred) or
/// the global `~/.ship/sail/`.
pub fn resolve_sail_path(tag: &str) -> Option<PathBuf> {
    let filename = format!("{tag}.sail");
    for dir in [
        runtime::project::get_project_dir(None).ok(),
        runtime::project::get_global_dir().ok(),
    ]
    .into_iter()
    .flatten()
    {
        let p = dir.join("sail").join(&filename);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Split `<tag>.js`, `<tag>.html`, `<tag>.d.ts`, or `<tag>.json` into
/// (tag, ext). Two-piece extension `.d.ts` handled explicitly.
fn split_tag_ext(file: &str) -> Option<(String, &str)> {
    if let Some(stem) = file.strip_suffix(".d.ts") {
        return Some((stem.to_string(), "d.ts"));
    }
    let dot = file.rfind('.')?;
    let stem = &file[..dot];
    let ext = &file[dot + 1..];
    if matches!(ext, "js" | "html" | "json") {
        Some((stem.to_string(), ext))
    } else {
        None
    }
}

fn is_valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && tag.contains('-')
}

fn respond(body: &str, content_type: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body.to_string()))
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
    tracing::warn!(error = %msg, "sail compile/serve failed");
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
    fn split_tag_ext_handles_two_piece_dts() {
        assert_eq!(split_tag_ext("app-x.d.ts"), Some(("app-x".into(), "d.ts")));
        assert_eq!(split_tag_ext("app-x.js"), Some(("app-x".into(), "js")));
        assert_eq!(split_tag_ext("app-x.html"), Some(("app-x".into(), "html")));
        assert_eq!(split_tag_ext("app-x.json"), Some(("app-x".into(), "json")));
        assert!(split_tag_ext("app-x.txt").is_none());
        assert!(split_tag_ext("noext").is_none());
    }

    #[test]
    fn tag_validation_requires_hyphen() {
        assert!(is_valid_tag("app-x"));
        assert!(is_valid_tag("sail-counter"));
        assert!(!is_valid_tag("nohyphen"));
        assert!(!is_valid_tag(""));
        assert!(!is_valid_tag("bad/path"));
        assert!(!is_valid_tag("../etc"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_tag_returns_404() {
        let resp = serve_sail(Path("nohyphen.js".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unsupported_extension_returns_404() {
        let resp = serve_sail(Path("app-x.exe".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn page_invalid_tag_returns_404() {
        let resp = serve_sail_page(Path("nohyphen".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
