use super::*;
use axum::extract::Path;
use axum::response::IntoResponse;
use tempfile::TempDir;

async fn collect_body(response: axum::response::Response) -> serde_json::Value {
    use http_body_util::BodyExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or_else(|_| {
        serde_json::json!({"raw": String::from_utf8_lossy(&bytes).to_string()})
    })
}

async fn collect_raw(response: axum::response::Response) -> Vec<u8> {
    use http_body_util::BodyExt;
    response.into_body().collect().await.unwrap().to_bytes().to_vec()
}

fn setup_project_view(dir: &TempDir, name: &str) -> PathBuf {
    let views_dir = dir.path().join(".ship").join("views").join(name);
    std::fs::create_dir_all(&views_dir).unwrap();
    views_dir
}

// ---- is_valid_view_name ----

#[test]
fn valid_view_names() {
    assert!(is_valid_view_name("pipeline-monitor"));
    assert!(is_valid_view_name("my_view"));
    assert!(is_valid_view_name("v1"));
}

#[test]
fn invalid_view_names() {
    assert!(!is_valid_view_name(""));
    assert!(!is_valid_view_name(".."));
    assert!(!is_valid_view_name("foo/bar"));
    assert!(!is_valid_view_name("foo\\bar"));
    assert!(!is_valid_view_name("../etc/passwd"));
}

// ---- resolve_view_dir ----

#[test]
fn resolve_rejects_traversal() {
    assert!(resolve_view_dir("../etc").is_none());
    assert!(resolve_view_dir("foo/bar").is_none());
}

// ---- list_views ----

#[tokio::test]
async fn list_views_empty_returns_ok() {
    let response = list_views().await.into_response();
    let val = collect_body(response).await;
    assert_eq!(val["ok"], true);
    assert!(val["data"]["views"].is_array());
}

#[tokio::test]
async fn list_views_finds_html_view() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "dashboard");
    std::fs::write(view_dir.join("index.html"), "<h1>hi</h1>").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = list_views().await.into_response();
    unsafe { std::env::remove_var("SHIP_DIR") };

    let val = collect_body(response).await;
    let views = val["data"]["views"].as_array().unwrap();
    assert!(views.iter().any(|v| v["name"] == "dashboard" && v["type"] == "html"));
}

#[tokio::test]
async fn list_views_finds_component_view() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "monitor");
    std::fs::write(view_dir.join("index.js"), "export default class {}").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = list_views().await.into_response();
    unsafe { std::env::remove_var("SHIP_DIR") };

    let val = collect_body(response).await;
    let views = val["data"]["views"].as_array().unwrap();
    assert!(views.iter().any(|v| v["name"] == "monitor" && v["type"] == "component"));
}

// ---- serve_view ----

#[tokio::test]
async fn serve_view_rejects_traversal() {
    let response = serve_view(Path("../etc".to_string()), axum::extract::Query(std::collections::HashMap::new()), axum::http::HeaderMap::new()).await.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn serve_view_returns_html() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "test-html");
    std::fs::write(view_dir.join("index.html"), "<h1>hello</h1>").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = serve_view(Path("test-html".to_string()), axum::extract::Query(std::collections::HashMap::new()), axum::http::HeaderMap::new()).await.into_response();
    unsafe { std::env::remove_var("SHIP_DIR") };

    assert_eq!(response.status(), StatusCode::OK);
    let val = collect_body(response).await;
    assert_eq!(val["ok"], true);
    assert!(val["data"]["html"].as_str().unwrap().contains("<h1>hello</h1>"));
}

// ---- serve_view_manifest ----

#[tokio::test]
async fn manifest_from_yaml() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "pipe-mon");
    std::fs::write(
        view_dir.join("manifest.yaml"),
        "name: pipe-mon\ntype: component\nelement: ship-pipe-mon\nclaims: [pipeline]\nentry: index.js\n",
    ).unwrap();
    std::fs::write(view_dir.join("index.js"), "// js").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = serve_view_manifest(Path("pipe-mon".to_string())).await.into_response();
    unsafe { std::env::remove_var("SHIP_DIR") };

    assert_eq!(response.status(), StatusCode::OK);
    let val = collect_body(response).await;
    assert_eq!(val["ok"], true);
    assert_eq!(val["data"]["type"], "component");
    assert_eq!(val["data"]["elementName"], "ship-pipe-mon");
    assert_eq!(val["data"]["bundleUrl"], "/api/runtime/view/pipe-mon/bundle");
}

#[tokio::test]
async fn manifest_inferred_html() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "simple");
    std::fs::write(view_dir.join("index.html"), "<p>hi</p>").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = serve_view_manifest(Path("simple".to_string())).await.into_response();
    unsafe { std::env::remove_var("SHIP_DIR") };

    let val = collect_body(response).await;
    assert_eq!(val["data"]["type"], "html");
}

#[tokio::test]
async fn manifest_inferred_component() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "widget");
    std::fs::write(view_dir.join("index.js"), "class Widget {}").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = serve_view_manifest(Path("widget".to_string())).await.into_response();
    unsafe { std::env::remove_var("SHIP_DIR") };

    let val = collect_body(response).await;
    assert_eq!(val["data"]["type"], "component");
    assert_eq!(val["data"]["elementName"], "ship-widget-view");
}

#[tokio::test]
async fn manifest_not_found() {
    let response = serve_view_manifest(Path("nonexistent".to_string())).await.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ---- serve_view_bundle ----

#[tokio::test]
async fn bundle_serves_js() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "comp");
    std::fs::write(view_dir.join("index.js"), "console.log('hello');").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = serve_view_bundle(Path("comp".to_string())).await;
    unsafe { std::env::remove_var("SHIP_DIR") };

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap().to_str().unwrap(),
        "application/javascript"
    );
    assert_eq!(
        response.headers().get("cache-control").unwrap().to_str().unwrap(),
        "no-cache"
    );
    let body = collect_raw(response).await;
    assert_eq!(String::from_utf8(body).unwrap(), "console.log('hello');");
}

#[tokio::test]
async fn bundle_uses_manifest_entry() {
    let dir = TempDir::new().unwrap();
    let view_dir = setup_project_view(&dir, "custom");
    std::fs::write(
        view_dir.join("manifest.yaml"),
        "name: custom\ntype: component\nelement: ship-custom\nentry: bundle.js\n",
    ).unwrap();
    std::fs::write(view_dir.join("bundle.js"), "// custom bundle").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let response = serve_view_bundle(Path("custom".to_string())).await;
    unsafe { std::env::remove_var("SHIP_DIR") };

    assert_eq!(response.status(), StatusCode::OK);
    let body = collect_raw(response).await;
    assert_eq!(String::from_utf8(body).unwrap(), "// custom bundle");
}

#[tokio::test]
async fn bundle_not_found() {
    let response = serve_view_bundle(Path("nonexistent".to_string())).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn bundle_rejects_traversal() {
    let response = serve_view_bundle(Path("../etc".to_string())).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
