use std::fs;

use axum::Extension;
use axum::extract::Path;
use axum::extract::OriginalUri;
use axum::http::{StatusCode, header};

use crate::DaemonMode;
use crate::view_host::{serve_view_host, ssr_cache};
use crate::view_payload::serve_view_payload;

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

fn dev() -> Option<Extension<DaemonMode>> { Some(Extension(DaemonMode::Development)) }
fn prod() -> Option<Extension<DaemonMode>> { Some(Extension(DaemonMode::Production)) }

async fn body_str(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn make_uri(path: &str) -> OriginalUri {
    OriginalUri(format!("http://localhost{path}").parse().unwrap())
}

/// When a pre-compiled `.ssr.html` file exists in `dist/`, the payload
/// endpoint must return its contents in the `html` field rather than
/// re-rendering at request time.
#[tokio::test(flavor = "multi_thread")]
async fn payload_uses_precompiled_ssr_html() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-payload-precompiled");
    let dist_dir = view_dir.join("dist");
    fs::create_dir_all(&dist_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>source</p></div>").unwrap();
    let ssr_content = "<div><p data-ssr=\"1\">precompiled</p></div>";
    fs::write(dist_dir.join("index.ssr.html"), ssr_content).unwrap();

    let resp = serve_view_payload(None, Path("ssr-payload-precompiled".to_string())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(
        v["html"].as_str().unwrap(),
        ssr_content,
        "html must come from precompiled .ssr.html"
    );
}

/// When no `.ssr.html` file exists the endpoint gracefully falls back to
/// the runtime-rendered HTML (existing behaviour is preserved).
#[tokio::test(flavor = "multi_thread")]
async fn payload_falls_back_to_runtime_render_without_ssr_file() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-payload-fallback");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>runtime</p></div>").unwrap();

    let resp = serve_view_payload(None, Path("ssr-payload-fallback".to_string())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    let html = v["html"].as_str().unwrap();
    assert!(
        html.contains("<p>"),
        "runtime-rendered html must contain the template; got: {html}"
    );
}

/// `/view-host/<name>` with a matching `.ssr.html` must splice the SSR
/// content into the shell's `<div id="sui-root">`.
#[tokio::test(flavor = "multi_thread")]
async fn view_host_splices_precompiled_ssr() {
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

    let resp = serve_view_host(None, make_uri("/view-host/ssr-host-test-splice")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_str(resp).await;
    assert!(
        body.contains(ssr_content),
        "view-host must splice SSR content into shell; got: {body}"
    );
    assert!(
        body.contains("<div id=\"sui-root\">"),
        "sui-root div must be present in shell; got: {body}"
    );
}

/// When no `.ssr.html` exists, `/view-host/<name>` live-emits SSR from
/// source so the client has real markup to morph onto (no blank flash).
#[tokio::test(flavor = "multi_thread")]
async fn view_host_without_ssr_live_emits_from_source() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-host-test-empty");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();

    let resp = serve_view_host(None, make_uri("/view-host/ssr-host-test-empty")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_str(resp).await;
    assert!(
        body.contains("<!doctype html>"),
        "shell must be valid HTML; got: {body}"
    );
    assert!(
        body.contains("<div id=\"sui-root\">") && !body.contains("<div id=\"sui-root\"></div>"),
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
#[tokio::test(flavor = "multi_thread")]
async fn view_host_prod_uses_ssr_cache() {
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
    let resp = serve_view_host(prod(), make_uri(&format!("/view-host/{view_name}"))).await;
    assert!(body_str(resp).await.contains("content-v1"), "first prod request must read from disk");

    // Change file on disk.
    fs::write(&ssr_file, "content-v2").unwrap();

    // Second prod request — must still return cached v1.
    let resp = serve_view_host(prod(), make_uri(&format!("/view-host/{view_name}"))).await;
    let body = body_str(resp).await;
    assert!(body.contains("content-v1"), "prod second request must use cache (got v2 instead)");
    assert!(!body.contains("content-v2"), "prod must not bypass cache");
}

/// In development mode, view-host bypasses the SSR cache — each request
/// reads fresh content from disk.
#[tokio::test(flavor = "multi_thread")]
async fn view_host_dev_bypasses_ssr_cache() {
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

    let resp = serve_view_host(dev(), make_uri(&format!("/view-host/{view_name}"))).await;
    assert!(body_str(resp).await.contains("dev-v1"), "dev first request reads from disk");

    fs::write(&ssr_file, "dev-v2").unwrap();

    let resp = serve_view_host(dev(), make_uri(&format!("/view-host/{view_name}"))).await;
    let body = body_str(resp).await;
    assert!(body.contains("dev-v2"), "dev must bypass cache and read fresh content");
}

/// Production mode: view-payload response has must-revalidate cache header and ETag.
#[tokio::test(flavor = "multi_thread")]
async fn view_payload_prod_cache_header_and_etag() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/prod-cache-check");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>hello</p></div>").unwrap();

    let resp = serve_view_payload(prod(), Path("prod-cache-check".to_string())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert!(cc.contains("must-revalidate"), "prod must have must-revalidate; got: {cc}");
    assert!(resp.headers().contains_key(header::ETAG), "prod must have ETag header");
}

/// Development mode: view-payload response has no-cache header and no ETag.
#[tokio::test(flavor = "multi_thread")]
async fn view_payload_dev_no_cache_no_etag() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/dev-cache-check");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><p>hello</p></div>").unwrap();

    let resp = serve_view_payload(dev(), Path("dev-cache-check".to_string())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let cc = resp.headers().get(header::CACHE_CONTROL).unwrap().to_str().unwrap();
    assert_eq!(cc, "no-cache", "dev must use no-cache; got: {cc}");
    assert!(!resp.headers().contains_key(header::ETAG), "dev must not have ETag header");
}

/// Production mode: compile errors in view-payload are redacted (no source positions).
#[tokio::test(flavor = "multi_thread")]
async fn view_payload_prod_compile_error_redacted() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/broken-prod");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(prod(), Path("broken-prod".to_string())).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(
        v["error"]["redacted_error"].is_object(),
        "prod must redact compile errors; got: {v}"
    );
}

/// Development mode: compile errors include source position detail.
#[tokio::test(flavor = "multi_thread")]
async fn view_payload_dev_compile_error_includes_detail() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/broken-dev");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(dev(), Path("broken-dev".to_string())).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    let body = &v["error"]["compile_error"];
    assert!(body["line"].as_u64().unwrap() >= 1, "dev must include line number; got: {v}");
    assert!(!body["message"].as_str().unwrap_or("").is_empty(), "dev must include message; got: {v}");
}

// ── Migrated tests from view_payload.rs inline block ─────────────────────────

use crate::view_payload::serve_view_payload_subpath;
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

#[tokio::test(flavor = "multi_thread")]
async fn top_level_view_payload_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/demo");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"),
        r#"<div><p>{count()}</p></div>"#).unwrap();
    fs::write(view_dir.join("index.controller.ts"),
        "import { controller, signal } from '@ship/sui-runtime'\nexport default controller({ scope: () => ({ count: signal(0) }) })\n").unwrap();
    fs::write(view_dir.join("index.css"), "p { color: red; }").unwrap();

    let resp = serve_view_payload(None, Path("demo".to_string())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
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

#[tokio::test(flavor = "multi_thread")]
async fn sub_path_payload_concatenates_css() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/demo");
    let sub_dir = view_dir.join("[name]");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(view_dir.join("index.sui"), r#"<div></div>"#).unwrap();
    fs::write(view_dir.join("index.css"), ".parent {}").unwrap();
    fs::write(sub_dir.join("index.sui"), r#"<span></span>"#).unwrap();
    fs::write(sub_dir.join("index.css"), ".child {}").unwrap();

    let resp = serve_view_payload_subpath(
        None, Path(("demo".to_string(), "anything".to_string()))
    ).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
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

#[tokio::test(flavor = "multi_thread")]
async fn unknown_view_returns_404_json() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let resp = serve_view_payload(None, Path("nope".to_string())).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(v["error"]["view_not_found"]["name"], "nope");
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_name_returns_404_json() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let resp = serve_view_payload(None, Path("BadName".to_string())).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(v["error"]["invalid_name"].is_object(), "got: {v}");
}

#[tokio::test(flavor = "multi_thread")]
async fn compile_error_returns_500_with_position() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/broken");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<div><span></div>").unwrap();

    let resp = serve_view_payload(None, Path("broken".to_string())).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    let body = &v["error"]["compile_error"];
    assert_eq!(body["name"], "broken");
    assert!(body["line"].as_u64().unwrap() >= 1);
    assert!(!body["message"].as_str().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn island_tags_listed() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());
    let view_dir = tmp.path().join(".ship/views/term-view");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"),
        r#"<div><sui-terminal rows={24} cols={80}></sui-terminal></div>"#).unwrap();

    let resp = serve_view_payload(None, Path("term-view".to_string())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    let islands = v["islands"].as_array().unwrap();
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0], "sui-terminal");
}

#[tokio::test(flavor = "multi_thread")]
async fn first_party_view_payload_works() {
    let _g = lock_with_dir(repo_root());
    let resp = serve_view_payload(None, Path("agents".to_string())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(v["name"], "agents");
    assert_eq!(v["moduleUrl"], "/_sui/views/agents/view.js");
}
