use std::fs;

use axum::extract::Path;
use axum::extract::OriginalUri;
use axum::http::StatusCode;

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

    let resp = serve_view_payload(Path("ssr-payload-precompiled".to_string())).await;
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
    // Intentionally no dist/index.ssr.html.

    let resp = serve_view_payload(Path("ssr-payload-fallback".to_string())).await;
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

    let resp = serve_view_host(make_uri("/view-host/ssr-host-test-splice")).await;
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

/// When no `.ssr.html` exists, `/view-host/<name>` returns a valid HTML
/// shell with an empty `<div id="sui-root">` (no SSR, client mounts it).
#[tokio::test(flavor = "multi_thread")]
async fn view_host_without_ssr_returns_valid_shell() {
    let tmp = tempfile::tempdir().unwrap();
    let _g = lock_with_dir(tmp.path());

    let view_dir = tmp.path().join(".ship/views/ssr-host-test-empty");
    fs::create_dir_all(&view_dir).unwrap();
    fs::write(view_dir.join("index.sui"), "<p>hi</p>").unwrap();
    // Intentionally no dist/index.ssr.html.

    let resp = serve_view_host(make_uri("/view-host/ssr-host-test-empty")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_str(resp).await;
    assert!(
        body.contains("<!doctype html>"),
        "shell must be valid HTML; got: {body}"
    );
    assert!(
        body.contains("<div id=\"sui-root\"></div>"),
        "empty shell must have empty sui-root; got: {body}"
    );
}
