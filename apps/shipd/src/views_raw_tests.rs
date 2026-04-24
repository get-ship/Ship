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
async fn each_sdk_view_renders_its_root() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    for (name, marker) in [
        ("settings", "id=\"settings-"),
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
