//! Compile + serve handlers for `.set` single-file components.
//!
//! Mirrors `views::components` (which serves `.ship` files), but
//! plugs in the `sail-compiler` crate. Each `.set` file in the
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

use sail_compiler::{CompileOpts, CompileResult, compile_file};

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

/// `GET /_sail/demo` — side-by-side visual diff: Lit ship-set on the
/// left, Sail port on the right. Each side is its own iframe so the
/// custom-element registries don't collide.
pub async fn serve_sail_demo() -> Response {
    const DEMO_HTML: &str =
        include_str!("../../../../packages/ship-set-sail/demo/index.html");
    html_response(DEMO_HTML)
}

/// `GET /_sail/demo/sail` — Sail-only showcase. Iframe target on the
/// right side of the side-by-side view; also useful standalone.
pub async fn serve_sail_demo_sail() -> Response {
    const SAIL_HTML: &str =
        include_str!("../../../../packages/ship-set-sail/demo/sail.html");
    html_response(SAIL_HTML)
}

/// `GET /_sail/demo/lit` — Lit-only showcase. Iframe target on the
/// left side of the side-by-side view.
pub async fn serve_sail_demo_lit() -> Response {
    const LIT_HTML: &str =
        include_str!("../../../../packages/ship-set-sail/demo/lit.html");
    html_response(LIT_HTML)
}

fn html_response(body: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// Bundled @sail/runtime — the ESM module compiled `.set` components
/// import from. Built by `pnpm -F @sail/runtime bundle` (esbuild).
const SAIL_RUNTIME_JS: &str = include_str!("../../assets/sail-runtime.js");

/// `GET /_sail/runtime.js` — serves the bundled `@sail/runtime` module.
/// Compiled `.set` JS imports `from '@sail/runtime'`; the page wrapper
/// inlines an importmap that resolves that bare specifier to this URL.
pub async fn serve_sail_runtime() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(Body::from(SAIL_RUNTIME_JS))
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
    // Auto-load any sibling .set components this page references. Scans
    // both the SSR (initial paint) and the compiled JS (runtime templates
    // inside `{#if}` / `{#for}` branches and event-driven inserts) for
    // custom-element tags, deduplicates, and emits a script tag for each
    // component that resolves under `.ship/sail/`. The page just writes
    // `<ship-foo>` anywhere; the daemon wires the module loads.
    let dep_scripts = component_dep_scripts(&tag, &entry.ssr, &entry.js);
    // Importmap: compiled `.set` JS imports `from '@sail/runtime'`. The
    // browser can't resolve bare specifiers without a map. This points
    // every component module at the daemon-served bundle so a single
    // runtime instance is shared across all the page's components.
    let importmap = "  <script type=\"importmap\">\n  { \"imports\": { \"@sail/runtime\": \"/_sail/runtime.js\" } }\n  </script>\n";
    // Shared design tokens + theme bootstrap. Mirrors the inline
    // bootstrap in `apps/shell/index.html` so a `/sail/<tag>` page
    // looks the same as the equivalent view rendered inside the shell.
    // The theme `<script>` runs sync before first paint to avoid the
    // flash-of-wrong-theme.
    let chrome = r#"  <link rel="stylesheet" href="/api/assets/view-tokens.css" />
  <script>
    (() => {
      const VALID = new Set(["ship-light","ship-dark","paper","halo","console","contrast","glass"]);
      const DARK = new Set(["ship-dark","contrast","console"]);
      const PREFERS_DARK = matchMedia("(prefers-color-scheme: dark)").matches;
      const raw = localStorage.getItem("ship-site-theme");
      const id = VALID.has(raw) ? raw : (PREFERS_DARK ? "ship-dark" : "ship-light");
      document.documentElement.dataset.theme = id;
      document.documentElement.classList.toggle("dark", DARK.has(id));
    })();
  </script>
  <style>html,body{margin:0;background:var(--background);color:var(--foreground);font-family:var(--font-body);}</style>
"#;
    let html = format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n  <title>{tag}</title>\n{chrome}{importmap}</head>\n<body>\n{ssr}\n{dep_scripts}  <script type=\"module\" src=\"/_sail/components/{tag}.js\"></script>\n  <script src=\"/_sail/hmr-client.js\" defer></script>\n</body>\n</html>\n",
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

    // compile_file takes the path so the compiler can resolve sibling
    // `<style src="./x.css">` and any future path-relative directives.
    let opts = CompileOpts::default();
    let CompileResult { js, ssr, dts, schema, diagnostics: _ } =
        compile_file(path, &opts).map_err(|e| format!("{e:#}"))?;
    let entry = CacheEntry { mtime, len, js, ssr, dts, schema };
    if let Ok(mut c) = cache().write() {
        c.insert(tag.to_string(), entry.clone());
    }
    Ok(entry)
}

/// Scan SSR HTML and compiled JS for custom-element tag references
/// (any token starting with a lowercase letter, containing at least
/// one hyphen, of length 3+). Emit one `<script type="module">` per
/// unique tag that resolves under `.ship/sail/`. Skips the page tag
/// itself (already loaded by the caller).
fn component_dep_scripts(page_tag: &str, ssr: &str, js: &str) -> String {
    let mut found: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    collect_custom_element_tags(ssr, &mut found);
    collect_custom_element_tags(js, &mut found);
    found.remove(page_tag);
    let mut out = String::new();
    for tag in found {
        if resolve_sail_path(&tag).is_some() {
            out.push_str(&format!(
                "  <script type=\"module\" src=\"/_sail/components/{tag}.js\"></script>\n"
            ));
        }
    }
    out
}

/// Walk a string and append any kebab-case identifier (lowercase
/// letter start, contains a hyphen, ≥3 chars) into `into`. False
/// positives are harmless — they get filtered later by
/// `resolve_sail_path`, which only keeps tags that actually exist.
fn collect_custom_element_tags(src: &str, into: &mut std::collections::BTreeSet<String>) {
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let starts_token = if i == 0 {
            bytes[i].is_ascii_lowercase()
        } else {
            let prev = bytes[i - 1];
            !prev.is_ascii_alphanumeric() && prev != b'-' && prev != b'_'
                && bytes[i].is_ascii_lowercase()
        };
        if starts_token {
            let start = i;
            let mut end = i;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-')
            {
                end += 1;
            }
            if end - start >= 3 {
                let name = &src[start..end];
                if name.contains('-') {
                    into.insert(name.to_string());
                }
            }
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
}

/// Locate `<tag>.set` in the project's `.ship/sail/` (preferred) or
/// the global `~/.ship/sail/`.
pub fn resolve_sail_path(tag: &str) -> Option<PathBuf> {
    let filename = format!("{tag}.set");
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
