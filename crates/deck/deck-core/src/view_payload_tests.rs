use std::fs;

use crate::DaemonMode;
use crate::view_host::{serve_view_host, ssr_cache};
use crate::view_payload::{serve_view_payload, serve_view_payload_subpath};

const VAR: &str = "SHIP_DEV_PROJECT_DIR";

struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    prev: Option<std::ffi::OsString>,
}
impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(v) => unsafe { std::env::set_var(VAR, v) },
            None => unsafe { std::env::remove_var(VAR) },
        }
    }
}
fn lock_with_dir(p: &std::path::Path) -> EnvGuard {
    let lock = crate::test_lock::lock();
    let prev = std::env::var_os(VAR);
    unsafe { std::env::set_var(VAR, p); }
    EnvGuard { _lock: lock, prev }
}

fn body_str(resp: &crate::DeckResponse) -> String {
    String::from_utf8(resp.body.clone()).unwrap()
}

/// When a pre-compiled `.ssr.html` file exists in `dist/`, the payload
/// endpoint must return its contents in the `html` field rather than
/// re-rendering at request time.
#[test]
fn payload_uses_precompiled_ssr_html() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-payload-precompiled");
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>source</p></div>").unwrap();
    let ssr_content = "<div><p data-ssr=\"1\">precompiled</p></div>";
    fs::write(dist_dir.join("index.ssr.html"), ssr_content).unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "ssr-payload-precompiled");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(
        v["html"].as_str().unwrap(),
        ssr_content,
        "html must come from precompiled .ssr.html"
    );
}

/// When no `.ssr.html` file exists the endpoint gracefully falls back to
/// the runtime-rendered HTML (existing behaviour is preserved).
#[test]
fn payload_falls_back_to_runtime_render_without_ssr_file() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-payload-fallback");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>runtime</p></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "ssr-payload-fallback");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let html = v["html"].as_str().unwrap();
    assert!(
        html.contains("<p>"),
        "runtime-rendered html must contain the template; got: {html}"
    );
}

/// `/view-host/<name>` with a matching `.ssr.html` must splice the SSR
/// content into the shell's `<div id="sui-root">`.
#[test]
fn view_host_splices_precompiled_ssr() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-host-test-splice");
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();
    let ssr_content = "<p data-test=\"ssr-splice\">spliced</p>";
    fs::write(dist_dir.join("index.ssr.html"), ssr_content).unwrap();

    // Evict any stale cache entry from a previous run.
    {
        let mut cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
        cache.remove("ssr-host-test-splice");
    }

    let resp = serve_view_host(DaemonMode::Development, "/view-host/ssr-host-test-splice");
    assert_eq!(resp.status, 200);
    let body = body_str(&resp);
    assert!(
        body.contains(ssr_content),
        "view-host must splice SSR content into shell; got: {body}"
    );
    assert!(
        body.contains("<div id=\"sui-root\""),
        "sui-root div must be present in shell; got: {body}"
    );
}

/// When no `.ssr.html` exists, `/view-host/<name>` live-emits SSR from
/// source so the client has real markup to morph onto (no blank flash).
#[test]
fn view_host_without_ssr_live_emits_from_source() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-host-test-empty");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();

    let resp = serve_view_host(DaemonMode::Development, "/view-host/ssr-host-test-empty");
    assert_eq!(resp.status, 200);
    let body = body_str(&resp);
    assert!(
        body.contains("<!doctype html>"),
        "shell must be valid HTML; got: {body}"
    );
    assert!(
        body.contains("<div id=\"sui-root\"")
            && !body.contains("data-view=\"ssr-host-test-empty\"></div>"),
        "live-emit must populate sui-root when no .ssr.html exists; got: {body}"
    );
    assert!(
        body.contains("<p"),
        "live-emit must include source content; got: {body}"
    );
}

// ── Mode-specific tests ───────────────────────────────────────────────────────

/// In production mode, view-host uses the SSR cache — a second request
/// returns the originally-cached content even after the file changes.
#[test]
fn view_host_prod_uses_ssr_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_name = "ssr-mode-cache-test";
    let view_dir = tmp.path().join(format!(".ship/views/{view_name}"));
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();
    let ssr_file = dist_dir.join("index.ssr.html");
    fs::write(&ssr_file, "content-v1").unwrap();

    {
        let mut cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
        cache.remove(view_name);
    }

    // First request in prod — populates cache with v1.
    let resp = serve_view_host(DaemonMode::Production, &format!("/view-host/{view_name}"));
    assert!(body_str(&resp).contains("content-v1"), "first prod request must read from disk");

    // Change file on disk.
    fs::write(&ssr_file, "content-v2").unwrap();

    // Second prod request — must still return cached v1.
    let resp = serve_view_host(DaemonMode::Production, &format!("/view-host/{view_name}"));
    let body = body_str(&resp);
    assert!(body.contains("content-v1"), "prod second request must use cache (got v2 instead)");
    assert!(!body.contains("content-v2"), "prod must not bypass cache");
}

/// In development mode, view-host bypasses the SSR cache — each request
/// reads fresh content from disk.
#[test]
fn view_host_dev_bypasses_ssr_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_name = "ssr-mode-dev-bypass";
    let view_dir = tmp.path().join(format!(".ship/views/{view_name}"));
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();
    let ssr_file = dist_dir.join("index.ssr.html");
    fs::write(&ssr_file, "dev-v1").unwrap();

    {
        let mut cache = ssr_cache().lock().unwrap_or_else(|e| e.into_inner());
        cache.remove(view_name);
    }

    let resp = serve_view_host(DaemonMode::Development, &format!("/view-host/{view_name}"));
    assert!(body_str(&resp).contains("dev-v1"), "dev first request reads from disk");

    fs::write(&ssr_file, "dev-v2").unwrap();

    let resp = serve_view_host(DaemonMode::Development, &format!("/view-host/{view_name}"));
    let body = body_str(&resp);
    assert!(body.contains("dev-v2"), "dev must bypass cache and read fresh content");
}

/// Production mode: view-payload response has must-revalidate cache header and ETag.
#[test]
fn view_payload_prod_cache_header_and_etag() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/prod-cache-check");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>hello</p></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Production, "prod-cache-check");
    assert_eq!(resp.status, 200);
    assert!(resp.cache_control.contains("must-revalidate"), "prod must have must-revalidate; got: {}", resp.cache_control);
    assert!(resp.etag.is_some(), "prod must have ETag header");
}

/// Development mode: view-payload response has no-cache header and no ETag.
#[test]
fn view_payload_dev_no_cache_no_etag() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/dev-cache-check");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>hello</p></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "dev-cache-check");
    assert_eq!(resp.status, 200);
    assert_eq!(resp.cache_control, "no-cache", "dev must use no-cache; got: {}", resp.cache_control);
    assert!(resp.etag.is_none(), "dev must not have ETag header");
}

/// Production mode: compile errors in view-payload are redacted (no source positions).
#[test]
fn view_payload_prod_compile_error_redacted() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/broken-prod");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Production, "broken-prod");
    assert_eq!(resp.status, 500);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert!(
        v["error"]["redacted_error"].is_object(),
        "prod must redact compile errors; got: {v}"
    );
}

/// Development mode: compile errors include source position detail.
#[test]
fn view_payload_dev_compile_error_includes_detail() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/broken-dev");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "broken-dev");
    assert_eq!(resp.status, 500);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let body = &v["error"]["compile_error"];
    assert!(body["line"].as_u64().unwrap() >= 1, "dev must include line number; got: {v}");
    assert!(!body["message"].as_str().unwrap_or("").is_empty(), "dev must include message; got: {v}");
}

// ── Migrated tests from view_payload.rs inline block ─────────────────────────

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

#[test]
fn top_level_view_payload_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/demo");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"),
        r#"<div><p>{count()}</p></div>"#).unwrap();
    fs::write(view_dir.join("index.controller.ts"),
        "import { controller, signal } from '@ship/sui-runtime'\nexport default controller({ scope: () => ({ count: signal(0) }) })\n").unwrap();
    fs::write(view_dir.join("index.css"), "p { color: red; }").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "demo");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(v["name"], "demo");
    assert!(v["sub"].is_null());
    let html = v["html"].as_str().unwrap();
    assert!(html.contains("<p>"), "html should contain rendered template, got {html}");
    assert!(!html.contains("__set_init__"),
        "html must NOT include the script island — scope is a separate field");
    assert_eq!(v["css"], "p { color: red; }");
    assert!(v["scope"].is_object(), "scope should be a JSON object, got {}", v["scope"]);
    assert_eq!(v["moduleUrl"], "/_sui/views/demo/view.js");
    assert!(v["islands"].as_array().unwrap().is_empty());
}

#[test]
fn sub_path_payload_concatenates_css() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/demo");
    let sub_dir = view_dir.join("[name]");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(view_dir.join("index.sui"), r#"<div></div>"#).unwrap();
    fs::write(view_dir.join("index.css"), ".parent {}").unwrap();
    fs::write(sub_dir.join("index.sui"), r#"<span></span>"#).unwrap();
    fs::write(sub_dir.join("index.css"), ".child {}").unwrap();

    let resp = serve_view_payload_subpath(DaemonMode::Development, "demo", "anything");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(v["sub"], "anything");
    let css = v["css"].as_str().unwrap();
    assert!(css.contains(".parent"), "css missing parent: {css}");
    assert!(css.contains(".child"), "css missing child: {css}");
    assert!(
        css.find(".parent").unwrap() < css.find(".child").unwrap(),
        "parent must precede child for cascade order"
    );
    assert_eq!(v["moduleUrl"], "/_sui/views/demo/view.js?sub=anything");
}

#[test]
fn unknown_view_returns_404_json() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let resp = serve_view_payload(DaemonMode::Development, "nope");
    assert_eq!(resp.status, 404);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(v["error"]["view_not_found"]["name"], "nope");
}

/// The human-facing host route serves a rendered HTML 404 — not the raw JSON
/// the programmatic payload endpoint emits — and surfaces the views the daemon
/// CAN serve so the founder can recover. The JSON contract above is untouched.
#[test]
fn host_unknown_view_serves_html_404_with_available_views() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    // Two real views so the not-found page has something to list.
    for name in ["counter", "dashboard"] {
        let dir = tmp.path().join(format!(".ship/views/{name}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("index.sui"), "<p>hi</p>").unwrap();
    }

    let resp = serve_view_host(DaemonMode::Development, "/sui/home");
    assert_eq!(resp.status, 404, "missing view must 404");
    assert!(
        resp.content_type.contains("text/html"),
        "human-facing 404 must be HTML, got: {}",
        resp.content_type
    );
    let body = body_str(&resp);
    assert!(body.contains("<!doctype html>"), "must be a rendered page; got: {body}");
    assert!(body.contains("home"), "must name the requested view; got: {body}");
    assert!(body.contains("href=\"/\""), "must link back to the shell root; got: {body}");
    // Available-views list pulled from the resolver's roots.
    assert!(body.contains("/sui/counter"), "must list servable views; got: {body}");
    assert!(body.contains("/sui/dashboard"), "must list servable views; got: {body}");
    // The page must NOT leak the JSON error shape the payload endpoint uses.
    assert!(!body.contains("\"view_not_found\""), "must not dump raw JSON; got: {body}");
}

#[test]
fn invalid_name_returns_404_json() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let resp = serve_view_payload(DaemonMode::Development, "..");
    assert_eq!(resp.status, 404);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert!(v["error"]["invalid_name"].is_object(), "got: {v}");
}

#[test]
fn compile_error_returns_500_with_position() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/broken");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "broken");
    assert_eq!(resp.status, 500);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let body = &v["error"]["compile_error"];
    assert_eq!(body["name"], "broken");
    assert!(body["line"].as_u64().unwrap() >= 1);
    assert!(!body["message"].as_str().unwrap().is_empty());
}

/// Helper: write a `.deck/islands.json` fixture into the project dir so the
/// view-payload path resolves islands from the marker-derived manifest.
fn write_islands_manifest(root: &std::path::Path, json: &str) {
    let deck = root.join(".deck");
    fs::create_dir_all(&deck).unwrap();
    fs::write(deck.join("islands.json"), json).unwrap();
}

#[test]
fn island_tags_listed() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    write_islands_manifest(tmp.path(), r#"{"ship-terminal":"ship-terminal"}"#);
    let view_dir = tmp.path().join(".ship/views/term-view");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"),
        r#"<div><ship-terminal rows={24} cols={80}></ship-terminal></div>"#).unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "term-view");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let islands = v["islands"].as_array().unwrap();
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0], "ship-terminal");
}

/// The canvas fix: a view embedding `<sui-canvas>` resolves it as an island
/// when the marker-derived manifest flags it. Under the old hardcoded
/// `SUI_ISLANDS` table `sui-canvas` was missing, so it never lazy-loaded.
#[test]
fn canvas_resolves_as_island_from_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    write_islands_manifest(tmp.path(), r#"{"sui-canvas":"sui-canvas"}"#);
    let view_dir = tmp.path().join(".ship/views/canvas-view");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"),
        r#"<div><sui-canvas></sui-canvas></div>"#).unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "canvas-view");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    let islands = v["islands"].as_array().unwrap();
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0], "sui-canvas");
}

/// A used tag absent from the manifest is NOT an island (no separate bundle).
#[test]
fn unflagged_tag_is_not_an_island() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    write_islands_manifest(tmp.path(), r#"{"ship-terminal":"ship-terminal"}"#);
    let view_dir = tmp.path().join(".ship/views/plain-view");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"),
        r#"<div><sui-button></sui-button></div>"#).unwrap();

    let resp = serve_view_payload(DaemonMode::Development, "plain-view");
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert!(v["islands"].as_array().unwrap().is_empty());
}

#[test]
fn first_party_view_payload_works() {
    let _g = lock_with_dir(repo_root());
    // First-party views live under the ship host's `apps/shell/src`, supplied
    // via `SHIP_FIRST_PARTY_VIEWS` (the resolver's first-party bucket). `studio`
    // is a first-party view — it resolves there, not under the project's
    // top-level `views/`. (Was `agents`, which the studio reorg moved into
    // `apps/shell/src/studio/`.)
    let fp_prev = std::env::var_os("SHIP_FIRST_PARTY_VIEWS");
    unsafe { std::env::set_var("SHIP_FIRST_PARTY_VIEWS", repo_root().join("apps/shell/src")); }
    let resp = serve_view_payload(DaemonMode::Development, "studio");
    match fp_prev {
        Some(v) => unsafe { std::env::set_var("SHIP_FIRST_PARTY_VIEWS", v) },
        None => unsafe { std::env::remove_var("SHIP_FIRST_PARTY_VIEWS") },
    }
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_str(&body_str(&resp)).unwrap();
    assert_eq!(v["name"], "studio");
    assert_eq!(v["moduleUrl"], "/_sui/views/studio/view.js");
}
