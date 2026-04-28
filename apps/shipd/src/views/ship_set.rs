//! Compile + serve handlers for `.set` single-file components.
//!
//! Mirrors `views::components` (which serves `.ship` files), but
//! plugs in the ship-set-compiler crate. Each `.set` file in the
//! project's `.ship/ship-set/` directory (or the global fallback)
//! compiles on demand and is served as four artifacts:
//!
//! - `GET /_ship-set/components/<tag>.js`     — runtime ES module
//! - `GET /_ship-set/components/<tag>.html`   — initial-paint SSR HTML
//! - `GET /_ship-set/components/<tag>.d.ts`   — TypeScript declaration
//! - `GET /_ship-set/components/<tag>.json`   — public JSON schema
//!
//! Cached in-process keyed on the source file's `(mtime, len)`. An
//! editor save invalidates automatically on the next request.
//!
//! TODO(Phase 4): wire up ship-set-compiler once it lands. The
//! compile path (`load_or_compile`, `serve_ship_set_page`) is
//! stubbed out — requests return 503 until the compiler is available.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use std::path::PathBuf;

/// Bundled @ship-set/runtime — the ESM module compiled `.set` components
/// import from. Built by `pnpm -F @ship-set/runtime bundle` (esbuild).
const SHIP_SET_RUNTIME_JS: &str = include_str!("../../assets/ship-set-runtime.js");

/// `GET /_ship-set/runtime.js` — serves the bundled `@ship-set/runtime` module.
/// Compiled `.set` JS imports `from '@ship-set/runtime'`; the page wrapper
/// inlines an importmap that resolves that bare specifier to this URL.
pub async fn serve_ship_set_runtime() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(Body::from(SHIP_SET_RUNTIME_JS))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /ship-set/{tag}` — full HTML page that mounts `<tag>` with the
/// SSR'd initial paint, the runtime ES module, and the HMR client.
///
/// TODO(Phase 4): compile via ship-set-compiler; currently returns 503.
pub async fn serve_ship_set_page(Path(tag): Path<String>) -> Response {
    if !is_valid_tag(&tag) {
        return not_found("invalid tag");
    }
    let _path = match resolve_ship_set_path(&tag) {
        Some(p) => p,
        None => return not_found(&format!("ship-set component not found: {tag}")),
    };
    // TODO(Phase 4): ship-set-compiler will provide compilation here.
    server_unavailable("ship-set-compiler not yet available (Phase 4)")
}

/// `GET /_ship-set/components/{file}` — `file` is `<tag>.<ext>` where
/// `ext` selects which artifact to serve.
///
/// TODO(Phase 4): compile via ship-set-compiler; currently returns 503.
pub async fn serve_ship_set(Path(file): Path<String>) -> Response {
    let (tag, _ext) = match split_tag_ext(&file) {
        Some(t) => t,
        None => return not_found("expected <tag>.{js,html,d.ts,json}"),
    };
    if !is_valid_tag(&tag) {
        return not_found("invalid tag");
    }
    if resolve_ship_set_path(&tag).is_none() {
        return not_found(&format!("ship-set component not found: {tag}"));
    }
    // TODO(Phase 4): ship-set-compiler will provide compilation here.
    server_unavailable("ship-set-compiler not yet available (Phase 4)")
}

/// Locate `<tag>.set` in the project's `.ship/ship-set/` (preferred) or
/// the global `~/.ship/ship-set/`.
pub fn resolve_ship_set_path(tag: &str) -> Option<PathBuf> {
    let filename = format!("{tag}.set");
    for dir in [
        runtime::project::get_project_dir(None).ok(),
        runtime::project::get_global_dir().ok(),
    ]
    .into_iter()
    .flatten()
    {
        let p = dir.join("ship-set").join(&filename);
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

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

fn server_unavailable(msg: &str) -> Response {
    tracing::warn!(error = %msg, "ship-set compile/serve unavailable");
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("service unavailable")))
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
        assert!(is_valid_tag("ship-set-counter"));
        assert!(!is_valid_tag("nohyphen"));
        assert!(!is_valid_tag(""));
        assert!(!is_valid_tag("bad/path"));
        assert!(!is_valid_tag("../etc"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_tag_returns_404() {
        let resp = serve_ship_set(Path("nohyphen.js".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unsupported_extension_returns_404() {
        let resp = serve_ship_set(Path("app-x.exe".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn page_invalid_tag_returns_404() {
        let resp = serve_ship_set_page(Path("nohyphen".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
