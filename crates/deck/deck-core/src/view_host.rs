use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use sui_compiler::{bindings, parse, render};

use crate::{DaemonMode, DeckResponse};
use crate::resolver::resolve;

const VIEW_HOST_SHELL: &str = include_str!("../assets/view-host.html");
const SSR_PLACEHOLDER: &str = "<!-- SUI_SSR_CONTENT -->";
const VIEW_NAME_PLACEHOLDER: &str = "<!-- SUI_VIEW_NAME -->";

/// A cached SSR body keyed by the `.ssr.html` artifact's mtime. The mtime gates
/// reuse: when the artifact on disk changes (re-stat sees a new mtime), the
/// entry is a MISS and the body is reloaded — so an edited/re-emitted view is
/// never pinned to stale HTML until restart. Mirrors `resolver::render_resolved`.
struct CacheEntry {
    mtime: SystemTime,
    html: String,
}

static SSR_CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();

fn ssr_cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    SSR_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Drop a cached entry for `name`. Test-only — production invalidation is
/// automatic via the mtime gate; no caller needs to evict by hand.
#[cfg(test)]
pub(crate) fn evict_ssr_cache(name: &str) {
    let mut cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.remove(name);
}

fn view_name_from_path(path: &str) -> Option<String> {
    path.trim_start_matches('/')
        .split('/')
        .nth(1)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Resolve the on-disk `<stem>.ssr.html` mirror-tree artifact path for `name`.
/// Returns `None` when the view can't be resolved (no source path) — the caller
/// then falls back to live-emit and caches nothing.
fn ssr_artifact_path(name: &str) -> Option<PathBuf> {
    let resolved = resolve(name, None).ok()?;
    let source_path = resolved.source_path?;
    let stem = source_path.file_stem()?.to_str()?;
    let stem_clean = stem.trim_end_matches(".html");
    let mirror = sui_compiler::out_layout::resolve_out_dir(&source_path);
    Some(mirror.join(format!("{stem_clean}.ssr.html")))
}

/// Production SSR body for `name`, mtime-cached on the `.ssr.html` artifact.
///
/// The cache is keyed on the artifact's modification time: a hit re-stats the
/// file and only reuses the body when the mtime is unchanged, so a re-emitted
/// or edited artifact is picked up without a restart. An absent artifact (no
/// `.ssr.html` on disk yet) or an empty read is treated as a **MISS** and is
/// never cached — that lets the caller fall back to live-emit, and prevents an
/// empty `#sui-root` from being pinned in place until the artifact lands.
fn ssr_html_for_view(name: &str) -> String {
    let Some(ssr_path) = ssr_artifact_path(name) else {
        return String::new();
    };
    let mtime = std::fs::metadata(&ssr_path).and_then(|m| m.modified()).ok();
    if let Some(mtime) = mtime {
        {
            let cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = cache.get(name)
                && entry.mtime == mtime
            {
                return entry.html.clone();
            }
        }
        let html = std::fs::read_to_string(&ssr_path).unwrap_or_default();
        // Empty reads are a MISS: never cache them (the artifact may be mid-write
        // or a placeholder), so a later non-empty read isn't shadowed.
        if html.is_empty() {
            return html;
        }
        let mut cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
        cache.insert(name.to_string(), CacheEntry { mtime, html: html.clone() });
        return html;
    }
    // No artifact on disk → nothing to cache, caller falls back to live-emit.
    String::new()
}

pub(crate) fn load_ssr_from_disk(name: &str) -> Option<String> {
    let ssr_path = ssr_artifact_path(name)?;
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

    // A named-but-unresolvable view is a human-facing 404: serve the rendered
    // not-found page rather than a 200 shell that boots, fetches the JSON
    // payload, gets a 404, and dumps raw `{"error":{"view_not_found":…}}` into
    // a div. The bare `/view-host` landing (no name) keeps serving the shell —
    // its client-side message handles the "no view in URL" case. The
    // programmatic payload endpoint (`/_sui/views/<name>`) is untouched and
    // still answers JSON for API callers.
    if let Some(name) = view_name.as_deref()
        && (!crate::view::is_valid_view_name(name) || resolve(name, None).is_err())
    {
        return crate::error_page::view_not_found(mode, name);
    }
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
    fn unresolvable_view_serves_html_not_found_page() {
        // A named view the daemon can't resolve must now render the styled
        // 404 page (text/html, status 404) instead of a 200 shell that boots
        // only to dump the payload endpoint's raw JSON into a div. Point the
        // resolver at an empty tempdir so the view is deterministically absent.
        let _lock = crate::test_lock::lock();
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", tmp.path()); }

        let resp = serve_view_host(DaemonMode::Development, "/view-host/no-such-view-xyzzy");

        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }

        assert_eq!(resp.status, 404);
        assert!(resp.content_type.contains("text/html"));
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("<!doctype html>"), "must be a rendered page; got: {body}");
        assert!(body.contains("no-such-view-xyzzy"), "must name the missing view; got: {body}");
        assert!(body.contains("href=\"/\""), "must link back to the shell root; got: {body}");
        assert!(
            !body.contains("\"view_not_found\""),
            "must NOT leak the raw JSON error shape; got: {body}"
        );
    }

    #[test]
    fn resolvable_view_serves_shell_with_root_name() {
        // A view that resolves on disk still serves the 200 shell, with the
        // root advertising the view name. The bare `/view-host` landing (no
        // view in URL) keeps serving the shell with an empty name.
        let _lock = crate::test_lock::lock();
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", tmp.path()); }

        let view_dir = tmp.path().join(".ship/views/some-view");
        std::fs::create_dir_all(&view_dir).unwrap();
        std::fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();

        let resp = serve_view_host(DaemonMode::Development, "/view-host/some-view/sub");
        let bare = serve_view_host(DaemonMode::Development, "/view-host");

        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }

        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(
            body.contains("data-sui-root data-view=\"some-view\""),
            "root must advertise the resolved view name; got: {body}"
        );
        // The bare host landing (no view in URL) leaves the name empty.
        assert_eq!(bare.status, 200);
        let body = std::str::from_utf8(&bare.body).unwrap();
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
        evict_ssr_cache("ssr-host-splice");

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
    fn prod_ssr_cache_invalidates_when_artifact_changes() {
        // The stale-forever bug: in Production the SSR body was cached by view
        // name and never invalidated, so an edited/re-emitted `.ssr.html` served
        // the old HTML until restart. The mtime-keyed cache must pick up the new
        // artifact once its mtime advances.
        let _lock = crate::test_lock::lock();
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", tmp.path()); }

        let view_dir = tmp.path().join(".ship/views/ssr-cache-bust");
        let dist_dir = view_dir.join("dist");
        std::fs::create_dir_all(&dist_dir).unwrap();
        std::fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();
        let ssr_path = dist_dir.join("index.ssr.html");
        std::fs::write(&ssr_path, "<p>v1-pre-rendered</p>").unwrap();
        evict_ssr_cache("ssr-cache-bust");

        let first = ssr_html_for_view("ssr-cache-bust");
        assert!(first.contains("v1-pre-rendered"), "first read sees v1; got: {first}");

        // Rewrite the artifact and advance its mtime — a restart-free edit.
        std::fs::write(&ssr_path, "<p>v2-pre-rendered</p>").unwrap();
        let now = std::fs::metadata(&ssr_path).and_then(|m| m.modified()).unwrap();
        let bumped = now + std::time::Duration::from_secs(5);
        filetime::set_file_mtime(&ssr_path, filetime::FileTime::from_system_time(bumped)).unwrap();

        let second = ssr_html_for_view("ssr-cache-bust");
        assert!(
            second.contains("v2-pre-rendered"),
            "changed artifact must invalidate the cache (was serving stale forever); got: {second}"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }
    }

    #[test]
    fn prod_ssr_empty_artifact_is_not_cached() {
        // The empty-pinning bug: an absent/empty `.ssr.html` was cached as the
        // empty string, pinning the view to an empty #sui-root even after its
        // artifact landed. Empty/absent must be a MISS — once the real artifact
        // appears, the next request must serve it.
        let _lock = crate::test_lock::lock();
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", tmp.path()); }

        let view_dir = tmp.path().join(".ship/views/ssr-late-artifact");
        let dist_dir = view_dir.join("dist");
        std::fs::create_dir_all(&dist_dir).unwrap();
        std::fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();
        evict_ssr_cache("ssr-late-artifact");

        // No artifact yet → MISS, empty result, nothing cached.
        assert!(ssr_html_for_view("ssr-late-artifact").is_empty());

        // The artifact lands afterwards — must be served, not shadowed by a
        // cached empty.
        std::fs::write(dist_dir.join("index.ssr.html"), "<p>landed</p>").unwrap();
        let after = ssr_html_for_view("ssr-late-artifact");
        assert!(
            after.contains("landed"),
            "empty must never be cached; the landed artifact must be served; got: {after}"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }
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
