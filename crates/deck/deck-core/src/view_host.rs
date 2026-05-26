use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use sui_compiler::{bindings, parse, render};

use crate::{DaemonMode, DeckResponse};
use crate::resolver::resolve;

const VIEW_HOST_SHELL: &str = include_str!("../assets/view-host.html");
const SSR_PLACEHOLDER: &str = "<!-- SUI_SSR_CONTENT -->";
const VIEW_NAME_PLACEHOLDER: &str = "<!-- SUI_VIEW_NAME -->";

static SSR_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub fn ssr_cache() -> &'static Mutex<HashMap<String, String>> {
    SSR_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn view_name_from_path(path: &str) -> Option<String> {
    path.trim_start_matches('/')
        .split('/')
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
///
/// Uses the full `render()` so the body carries the `<script
/// id="__set_init__">` seed island — the client runtime needs it to
/// hydrate initial signal state, and `<data live>` views need its
/// `__subscribe__` URL to open the SSE channel. The precompiled
/// `<stem>.ssr.html` artifact (see `load_ssr_from_disk`) is emitted the
/// same way, so dev live-emit and prod from-disk stay byte-identical.
pub(crate) fn live_emit_ssr(name: &str) -> Option<String> {
    let resolved = resolve(name, None).ok()?;
    let program = parse::parse(&resolved.source).ok()?;
    let registry = bindings::build(&program);
    let opts = render::RenderOpts { view_name: name.to_string() };
    let output = render::render(&program, &registry, &opts).ok()?;
    Some(output.html)
}

pub fn serve_view_host(mode: DaemonMode, path: &str) -> DeckResponse {
    // Dev: always re-read from disk (no cache). Prod: use in-memory cache.
    // If no precompiled SSR exists, fall back to live-emit so the shell
    // never ships an empty #sui-root — the client morphs onto real markup.
    let view_name = view_name_from_path(path);
    let ssr_content = view_name
        .as_deref()
        .map(|name| {
            let from_disk = if mode == DaemonMode::Production {
                ssr_html_for_view(name)
            } else {
                load_ssr_from_disk(name).unwrap_or_default()
            };
            if from_disk.is_empty() {
                live_emit_ssr(name).unwrap_or_default()
            } else {
                from_disk
            }
        })
        .unwrap_or_default();
    let body = VIEW_HOST_SHELL
        .replace(
            VIEW_NAME_PLACEHOLDER,
            &render::escape_attr_value(view_name.as_deref().unwrap_or_default()),
        )
        .replace(SSR_PLACEHOLDER, &ssr_content);
    // Reflect the daemon mode so the live-reload client self-gates: it only
    // subscribes when content="development".
    let body = crate::mode_html::inject_mode_meta(&body, mode);
    DeckResponse::html_view(body, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_html_with_correct_headers() {
        let resp = serve_view_host(DaemonMode::Development, "/view-host");
        assert_eq!(resp.status, 200);
        assert!(resp.content_type.contains("text/html"));
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("<!doctype html>"));
        assert!(body.contains("import { mount } from '@ship/sui-runtime'"));
        assert!(body.contains("/_sui/views/"));
        let imap = body.find("type=\"importmap\"").expect("importmap present");
        let module = body.find("type=\"module\"").expect("module script present");
        assert!(imap < module, "importmap must come before any module script");
    }

    #[test]
    fn no_ssr_file_leaves_root_empty() {
        let resp = serve_view_host(DaemonMode::Development, "/view-host/no-such-view-xyzzy");
        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        // The content placeholder collapses to empty, but the root keeps
        // its `data-sui-root` / `data-view` scoping attributes.
        assert!(
            body.contains(
                "<div id=\"sui-root\" data-sui-root data-view=\"no-such-view-xyzzy\"></div>"
            ),
            "content placeholder must collapse to empty while root attrs stay; got: {body}"
        );
    }

    #[test]
    fn root_carries_view_name_from_path() {
        let resp = serve_view_host(DaemonMode::Development, "/view-host/some-view/sub");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(
            body.contains("data-sui-root data-view=\"some-view\""),
            "root must advertise the resolved view name; got: {body}"
        );
        // The bare host landing (no view in URL) leaves the name empty.
        let resp = serve_view_host(DaemonMode::Development, "/view-host");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(
            body.contains("data-sui-root data-view=\"\""),
            "bare /view-host must leave data-view empty; got: {body}"
        );
    }

    #[test]
    fn ssr_file_spliced_into_root() {
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

        let resp = serve_view_host(DaemonMode::Development, "/view-host/ssr-host-splice");
        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }

        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(
            body.contains(ssr_content),
            "SSR content must be spliced into shell; got: {body}"
        );
        assert!(
            body.contains("<div id=\"sui-root\""),
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
