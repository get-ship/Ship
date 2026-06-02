//! Integration tests for `POST /api/v1/annotations`.

use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;
use ulid::Ulid;

use crate::auth::AuthContext;
use crate::routes::rest_api::ApiState;
use crate::ship_runtime::ShipRuntime;

fn router_for(state: ApiState, auth: AuthContext) -> Router {
    Router::new()
        .route(
                "/api/v1/annotations",
            axum::routing::post(super::post_annotation),
        )
        .with_state(state)
        .layer(axum::Extension(auth))
}

fn fresh_state() -> (ApiState, TempDir) {
    let dir = TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    // Register built-in collection schemas so create_item against the
    // `annotations` table doesn't fail with "no such table".
    let mut data = ship_substrate::data::DataService::open(&ship_dir).unwrap();
    ship_platform::collections::builtin::register_builtin_schemas(&mut data).unwrap();
    drop(data);
    let kernel = Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let state: ApiState = Arc::new(ShipRuntime::new_test(ship_dir, kernel));
    (state, dir)
}

fn human_auth() -> AuthContext {
    AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        // Annotations are Space content (`collection.item_created` is
        // Space-scoped); the caller must be bound to a space.
        space_id: Some("space-test".to_string()),
        session_id: None,
        scopes: vec![crate::auth::Scope::Full],
        grants: vec![],
    }
}

async fn send_json(router: &Router, body: serde_json::Value) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/annotations")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
    (status, json)
}

fn point_payload(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "kind": "point",
        "target": { "view_id": "v-1", "selector": "#hero" },
        "intent": "request_change",
        "note": "swap the gradient",
        "created_at": "2026-04-20T00:00:00Z",
        "author_kind": "human",
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn point_annotation_persists_and_returns_201() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state.clone(), human_auth());

    let id = Ulid::new().to_string();
    let (status, body) = send_json(&router, point_payload(&id)).await;
    assert_eq!(status, 201, "{body}");
    assert_eq!(body["id"].as_str().unwrap(), id);

    // Persistence: row exists in the annotations collection.
    let mut guard = state.data_service().await.unwrap();
    let data = guard.as_mut().unwrap();
    let schema = ship_platform::collections::builtin::annotations_schema();
    let item = ship_platform::collections::service::get_item(data, &schema, &id).unwrap();
    assert!(item.is_some(), "annotation row missing");
    let row = item.unwrap();
    assert_eq!(row.data["kind"], "point");
    assert_eq!(row.data["view_id"], "v-1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn point_without_selector_rejected() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());

    let mut payload = point_payload(&Ulid::new().to_string());
    payload["target"] = serde_json::json!({ "view_id": "v-1" });

    let (status, body) = send_json(&router, payload).await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(body["error"]["code"], "selector_required");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn server_assigns_id_when_missing_or_invalid() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());

    let mut payload = point_payload("not-a-ulid");
    payload["id"] = serde_json::json!("");

    let (status, body) = send_json(&router, payload).await;
    assert_eq!(status, 201, "{body}");
    let id = body["id"].as_str().unwrap();
    assert!(Ulid::from_string(id).is_ok());
}
