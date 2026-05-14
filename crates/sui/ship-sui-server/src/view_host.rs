use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use axum::{
    Extension,
    body::Body,
    extract::OriginalUri,
    http::{StatusCode, header},
    response::Response,
};

use sui_compiler::{bindings, parse, render};

use crate::{DaemonMode, compute_etag, mode_from_ext, view_cache};
use crate::resolver::resolve;

const VIEW_HOST_SHELL: &str = include_str!("../assets/view-host.html");
const SSR_PLACEHOLDER: &str = "<!-- SUI_SSR_CONTENT -->";

static SSR_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub(crate) fn ssr_cache() -> &'static Mutex<HashMap<String, String>> {
    SSR_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn view_name_from_path(path: &str) -> Option<String> {
    path.trim_start_matches('/')
        .splitn(3, '/')
        .nth(1)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn ssr_html_for_view(name: &str) -> String {
    {
        let cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(html) = cache.get(name) {
            return html.clone();
        }
    }
    let html = load_ssr_from_disk(name).unwrap_or_default();
    let mut cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.insert(name.to_string(), html.clone());
    html
}

pub(crate) fn load_ssr_from_disk(name: &str) -> Option<String> {
    let resolved = resolve(name, None).ok()?;
    let source_path = resolved.source_path?;
    let stem = source_path.file_stem()?.to_str()?;
    let stem_clean = stem.trim_end_matches(".html");
    let mirror = sui_compiler::out_layout::resolve_out_dir(&source_path);
    let ssr_path = mirror.join(format!("{stem_clean}.ssr.html"));
    std::fs::read_to_string(&ssr_path).ok()
}

/// Render SSR HTML at request time from the source view. Used as a
/// fallback when no precompiled mirror-tree `<stem>.ssr.html` exists yet
/// — without this, the shell ships an empty `#sui-root` and the client
/// has nothing to morph onto.
pub(crate) fn live_emit_ssr(name: &str) -> Option<String> {
    let resolved = resolve(name, None).ok()?;
    let program = parse::parse(&resolved.source).ok()?;
    let registry = bindings::build(&program);
    let opts = render::RenderOpts { view_name: name.to_string() };
    let split = render::render_split(&program, &registry, &opts).ok()?;
    Some(split.html)
}

pub async fn serve_view_host(
    mode: Option<Extension<DaemonMode>>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    let mode = mode_from_ext(mode);
    // Dev: always re-read from disk (no cache). Prod: use in-memory cache.
    // If no precompiled SSR exists, fall back to live-emit so the shell
    // never ships an empty #sui-root — the client morphs onto real markup.
    let ssr_content = view_name_from_path(uri.path())
        .map(|name| {
            let from_disk = if mode == DaemonMode::Production {
                ssr_html_for_view(&name)
            } else {
                load_ssr_from_disk(&name).unwrap_or_default()
            };
            if from_disk.is_empty() {
                live_emit_ssr(&name).unwrap_or_default()
            } else {
                from_disk
            }
        })
        .unwrap_or_default();
    let body = VIEW_HOST_SHELL.replace(SSR_PLACEHOLDER, &ssr_content);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, view_cache(mode));
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(body.as_bytes()));
    }
    builder.body(Body::from(body)).unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    fn repo_root() -> &'static std::path::Path {
        static ROOT: OnceLock<PathBuf> = OnceLock::new();
        ROOT.get_or_init(|| {
            let mut cur = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            loop {
                let cargo = cur.join("Cargo.toml");
                if cargo.is_file()
                    && std::fs::read_to_string(&cargo)
                        .map(|s| s.contains("[workspace]"))
                        .unwrap_or(false)
                {
                    return cur;
                }
                if !cur.pop() {
                    panic!("could not find repo root from {}", env!("CARGO_MANIFEST_DIR"));
                }
            }
        });
        ROOT.get().unwrap()
    }

    fn make_uri(path: &str) -> OriginalUri {
        OriginalUri(format!("http://localhost{path}").parse().unwrap())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn serves_html_with_correct_headers() {
        let resp = serve_view_host(None, make_uri("/view-host")).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("text/html"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains("<!doctype html>"));
        assert!(body.contains("import { mount } from '@ship/sui-runtime'"));
        assert!(body.contains("/_sui/views/"));
        let imap = body.find("type=\"importmap\"").expect("importmap present");
        let module = body.find("type=\"module\"").expect("module script present");
        assert!(imap < module, "importmap must come before any module script");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn no_ssr_file_leaves_root_empty() {
        let resp = serve_view_host(None, make_uri("/view-host/no-such-view-xyzzy")).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(
            body.contains("<div id=\"sui-root\"></div>"),
            "placeholder must be replaced with empty string when no SSR file exists; got: {body}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ssr_file_spliced_into_root() {
        let _lock = crate::test_lock::lock();
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", tmp.path()); }

        let view_dir = tmp.path().join(".ship/views/ssr-host-splice");
        let dist_dir = view_dir.join("dist");
        std::fs::create_dir_all(&dist_dir).unwrap();
        std::fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();
        let ssr_content = "<p data-test=\"ssr\">pre-rendered</p>";
        std::fs::write(dist_dir.join("index.ssr.html"), ssr_content).unwrap();

        // Evict cache entry in case a prior run cached an empty value.
        {
            let mut cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
            cache.remove("ssr-host-splice");
        }

        let resp = serve_view_host(None, make_uri("/view-host/ssr-host-splice")).await;
        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(
            body.contains(ssr_content),
            "SSR content must be spliced into shell; got: {body}"
        );
        assert!(
            body.contains("<div id=\"sui-root\">"),
            "sui-root div must be present; got: {body}"
        );
    }

    #[test]
    fn view_name_extracted_from_various_paths() {
        assert_eq!(view_name_from_path("/view-host/counter"), Some("counter".to_string()));
        assert_eq!(view_name_from_path("/view-host/counter/sub"), Some("counter".to_string()));
        assert_eq!(view_name_from_path("/sui/counter/sub"), Some("counter".to_string()));
        assert_eq!(view_name_from_path("/view-host"), None);
        assert_eq!(view_name_from_path("/"), None);
    }
}
