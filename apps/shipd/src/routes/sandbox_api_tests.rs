//! Tests for the `/api/v1/sandboxes/*` endpoints.
//!
//! The dispatcher is a process-wide singleton (`OnceLock`); tests
//! that mutate registry state would race within the same test
//! process. We avoid that by giving each test a unique project
//! name + asserting only on its own sandboxes via response filtering.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use ship_isolation::tier::{IsolationProfile, IsolationStrategy, IsolationTier};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::routes::rest_api::ApiState;
use crate::routes::sandbox_api;
use crate::ship_runtime::ShipRuntime;

fn fresh_state() -> (ApiState, TempDir) {
    let dir = TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let state: ApiState = Arc::new(ShipRuntime::new_test(ship_dir, kernel));
    (state, dir)
}

fn router() -> Router {
    let (state, _dir) = fresh_state();
    Router::new()
        .route("/api/v1/sandboxes", axum::routing::get(sandbox_api::sandbox_list))
        .route(
            "/api/v1/sandboxes/{id}",
            axum::routing::get(sandbox_api::sandbox_get),
        )
        .with_state(state)
}

fn container_profile() -> IsolationProfile {
    IsolationProfile {
        tier: Some(IsolationTier::None),
        strategy: Some(IsolationStrategy::Noop),
        policy: None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sandbox_list_includes_registered_sandbox() {
    let dispatcher = ship_platform::sandbox::dispatcher();
    let project = format!("test-list-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let id = dispatcher
        .ensure_default_sandbox(&project, container_profile())
        .await
        .unwrap();

    let app = router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/sandboxes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let sandboxes = v["data"]["sandboxes"].as_array().expect("sandboxes array");
    assert!(
        sandboxes.iter().any(|s| s["id"].as_str() == Some(&id.0)),
        "list must include the registered sandbox; got {sandboxes:?}"
    );
    let mine = sandboxes
        .iter()
        .find(|s| s["id"].as_str() == Some(&id.0))
        .unwrap();
    assert_eq!(mine["project"].as_str(), Some(project.as_str()));
    assert_eq!(mine["name"].as_str(), Some("default"));
    assert_eq!(mine["state"].as_str(), Some("running"));
    assert_eq!(mine["tier"].as_str(), Some("none"));
    assert_eq!(mine["has_owner"].as_bool(), Some(false));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sandbox_get_returns_individual_sandbox() {
    let dispatcher = ship_platform::sandbox::dispatcher();
    let project = format!("test-get-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let id = dispatcher
        .ensure_default_sandbox(&project, container_profile())
        .await
        .unwrap();

    let app = router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sandboxes/{}", id.0))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["data"]["id"].as_str(), Some(id.0.as_str()));
    assert_eq!(v["data"]["project"].as_str(), Some(project.as_str()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sandbox_get_unknown_id_returns_404() {
    let app = router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/sandboxes/sbx_doesnotexist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sandbox_list_actors_reflects_register_calls() {
    let dispatcher = ship_platform::sandbox::dispatcher();
    let project = format!("test-actors-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
    let id = dispatcher
        .ensure_default_sandbox(&project, container_profile())
        .await
        .unwrap();
    dispatcher
        .register_actor(&id, format!("actor-a-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
        .await
        .unwrap();
    dispatcher
        .register_actor(&id, format!("actor-b-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
        .await
        .unwrap();

    let app = router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/sandboxes/{}", id.0))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let actors = v["data"]["actors"].as_array().expect("actors array");
    assert_eq!(actors.len(), 2);
}
