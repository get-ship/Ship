//! Tests for `views_raw::serve_view_raw` — SDK-wrapped shells and
//! legacy user-view fallback.

use super::*;
use axum::body::to_bytes;
use axum::extract::Path as AxPath;
use tempfile::TempDir;

fn make_api_state(dir: &TempDir) -> ApiState {
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        runtime::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
}

async fn body_string(resp: Response) -> String {
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn pipelines_raw_returns_wrapped_sdk_shell() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = serve_view_raw(State(state), AxPath("pipelines".into())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("text/html"), "content-type: {ct}");
    assert!(
        resp.headers().contains_key(axum::http::header::SET_COOKIE),
        "must mint session cookie"
    );

    let body = body_string(resp).await;
    assert!(body.contains("<!DOCTYPE html>"), "must be full document");
    assert!(body.contains("</html>"));
    // Absolute asset URLs so Studio's DOM-injection host loads from daemon.
    assert!(
        body.contains("http://127.0.0.1:9315/api/runtime/assets/view-tokens.css"),
        "asset URL must be absolute"
    );
    assert!(
        body.contains("http://127.0.0.1:9315/api/runtime/assets/datastar.js"),
        "asset URL must be absolute"
    );
    // SDK shell content is embedded.
    assert!(
        body.contains("id=\"pipelines-root\""),
        "SDK shell must be inlined"
    );
    // No legacy bootstrap relative @get remains.
    assert!(
        !body.contains("@get('/api/runtime/view/pipelines'"),
        "legacy relative bootstrap must be gone"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn each_sdk_view_renders_its_root() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    for (name, marker) in [
        ("pipelines", "id=\"pipelines-root\""),
        ("git", "id=\"git-root\""),
        ("specs", "id=\"specs-"),
        ("settings", "id=\"settings-"),
        ("agents", "id=\"agents-root\""),
        ("skills", "id=\"skills-root\""),
        ("chat", "id=\"chat-root\""),
        ("events", "id=\"events-root\""),
    ] {
        let resp =
            serve_view_raw(State(state.clone()), AxPath(name.into())).await;
        assert_eq!(resp.status(), StatusCode::OK, "view {name} status");
        let body = body_string(resp).await;
        assert!(
            body.contains(marker),
            "view {name} must render its root marker `{marker}`\nbody head: {}",
            &body.chars().take(300).collect::<String>()
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_name_rejected() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let resp =
        serve_view_raw(State(state), AxPath("../etc/passwd".into())).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_view_returns_404() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let resp = serve_view_raw(State(state), AxPath("nonexistent".into())).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn user_view_fallback_reads_index_html() {
    let dir = TempDir::new().unwrap();
    let ship_dir = dir.path().join(".ship");
    let view_dir = ship_dir.join("views").join("custom");
    std::fs::create_dir_all(&view_dir).unwrap();
    std::fs::write(view_dir.join("index.html"), "<p>legacy body</p>").unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();

    unsafe {
        std::env::set_var("SHIP_DIR", ship_dir.to_str().unwrap());
    }
    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        runtime::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let state: ApiState =
        std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel));

    let resp = serve_view_raw(State(state), AxPath("custom".into())).await;
    unsafe {
        std::env::remove_var("SHIP_DIR");
    }
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(body.contains("<p>legacy body</p>"));
}
