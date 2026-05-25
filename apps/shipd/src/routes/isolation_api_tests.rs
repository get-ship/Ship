//! Integration tests for `/api/v1/isolation/*`.
//!
//! Exercises the six endpoints against a real (in-memory) ApiState +
//! daemon_settings DB:
//!
//!   GET  /api/v1/isolation/status
//!   POST /api/v1/isolation/rescan
//!   GET  /api/v1/isolation/recommendations
//!   POST /api/v1/isolation/preview
//!   POST /api/v1/isolation/verify
//!   POST /api/v1/isolation/init
//!
//! Verifies wire shape (envelope, fields, status codes) and the
//! contract that init refuses to flip enforce when verify is red.

use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::auth::AuthContext;
use crate::routes::rest_api::ApiState;
use crate::ship_runtime::ShipRuntime;

fn router_for(state: ApiState, auth: AuthContext) -> Router {
    Router::new()
        .route(
            "/api/v1/isolation/status",
            axum::routing::get(super::isolation_api::isolation_status),
        )
        .route(
            "/api/v1/isolation/rescan",
            axum::routing::post(super::isolation_api::isolation_rescan),
        )
        .route(
            "/api/v1/isolation/recommendations",
            axum::routing::get(super::isolation_api::isolation_recommendations),
        )
        .route(
            "/api/v1/isolation/preview",
            axum::routing::post(super::isolation_api::isolation_preview),
        )
        .route(
            "/api/v1/isolation/verify",
            axum::routing::post(super::isolation_api::isolation_verify),
        )
        .route(
            "/api/v1/isolation/init",
            axum::routing::post(super::isolation_api::isolation_init),
        )
        .with_state(state)
        .layer(axum::Extension(auth))
}

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

fn human_auth() -> AuthContext {
    AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        session_id: None,
        scopes: vec![crate::auth::Scope::Full],
        grants: vec![],
    }
}

async fn get(router: &Router, path: &str) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
    (status, json)
}

async fn post(
    router: &Router,
    path: &str,
    body: serde_json::Value,
) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(path)
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

// ── status / rescan ──────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn status_returns_envelope_with_capabilities_and_decision() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) = get(&router, "/api/v1/isolation/status").await;
    assert_eq!(status, 200);
    assert_eq!(body["ok"], serde_json::Value::Bool(true));
    let data = &body["data"];
    assert!(data.get("capabilities").is_some(), "capabilities missing");
    assert!(data.get("decision").is_some(), "decision missing");
    // capabilities must carry `os`
    assert!(data["capabilities"].get("os").is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rescan_returns_same_shape_as_status() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) = post(&router, "/api/v1/isolation/rescan", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let data = &body["data"];
    assert!(data.get("capabilities").is_some());
    assert!(data.get("decision").is_some());
}

// ── recommendations ──────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn recommendations_lists_all_five_tiers() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) = get(&router, "/api/v1/isolation/recommendations").await;
    assert_eq!(status, 200);
    let tiers = body["data"]["tiers"].as_array().expect("tiers array");
    assert_eq!(tiers.len(), 5, "5 tiers expected, got {}", tiers.len());
    let ids: Vec<&str> = tiers
        .iter()
        .map(|t| t["tier"].as_str().unwrap_or(""))
        .collect();
    for expected in ["none", "process", "container", "vm", "cloud"] {
        assert!(ids.contains(&expected), "missing tier: {expected}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn recommendations_marks_exactly_one_tier_recommended() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (_, body) = get(&router, "/api/v1/isolation/recommendations").await;
    let tiers = body["data"]["tiers"].as_array().unwrap();
    let recommended_count = tiers
        .iter()
        .filter(|t| t["recommended"].as_bool().unwrap_or(false))
        .count();
    assert_eq!(recommended_count, 1, "exactly one tier should be marked recommended");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn recommendations_each_strategy_has_availability_flag() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (_, body) = get(&router, "/api/v1/isolation/recommendations").await;
    for tier in body["data"]["tiers"].as_array().unwrap() {
        for strat in tier["strategies"].as_array().unwrap() {
            assert!(strat.get("available").is_some(), "strategy {strat:?} missing 'available'");
            // unavailable strategies carry a reason; available ones don't.
            let avail = strat["available"].as_bool().unwrap_or(false);
            if !avail {
                assert!(
                    strat["unavailable_reason"].is_string(),
                    "unavailable strategy {strat:?} should have a reason"
                );
            }
        }
    }
}

// ── preview ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_resolves_tier_none_to_noop() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) =
        post(&router, "/api/v1/isolation/preview", serde_json::json!({ "tier": "none" })).await;
    assert_eq!(status, 200);
    assert_eq!(body["data"]["resolved"]["strategy"], "noop");
    assert_eq!(body["data"]["resolved"]["tier"], "none");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_with_unknown_strategy_still_resolves_to_something() {
    // Unknown strategy id is ignored (parsed as None); recommender
    // falls back to OS default. Should return 200, not 400.
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) = post(
        &router,
        "/api/v1/isolation/preview",
        serde_json::json!({ "strategy": "totally-fake" }),
    )
    .await;
    assert_eq!(status, 200);
    assert!(body["data"]["resolved"]["strategy"].is_string());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_does_not_persist_settings() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state.clone(), human_auth());
    // Read setting before
    let before = ship_platform::config::get_setting("isolation.default_tier").ok().flatten();
    let _ = post(
        &router,
        "/api/v1/isolation/preview",
        serde_json::json!({ "tier": "container" }),
    )
    .await;
    let after = ship_platform::config::get_setting("isolation.default_tier").ok().flatten();
    assert_eq!(before, after, "preview should not touch persisted settings");
}

// ── verify ───────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn verify_returns_step_list() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) = post(&router, "/api/v1/isolation/verify", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let steps = body["data"]["steps"].as_array().expect("steps array");
    assert!(!steps.is_empty(), "verify should always produce step entries");
    for step in steps {
        assert!(step.get("name").is_some());
        assert!(step.get("ok").is_some());
        assert!(step.get("detail").is_some());
    }
}

// ── init ─────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn init_writes_tier_setting() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) = post(
        &router,
        "/api/v1/isolation/init",
        serde_json::json!({ "tier": "process" }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body["data"]["ok"], serde_json::Value::Bool(true));
    let v = ship_platform::config::get_setting("isolation.default_tier")
        .unwrap()
        .unwrap();
    assert_eq!(v, serde_json::Value::String("process".into()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn init_respects_verify_when_flipping_enforce() {
    // Contract: enforce only flips when verify is green. The actual
    // verify outcome depends on host capabilities — on a host where
    // verify passes, enforce flips; on a host where it fails, init
    // returns 400 and enforce stays at default.
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, body) = post(
        &router,
        "/api/v1/isolation/init",
        serde_json::json!({
            "tier": "vm",
            "strategy": "lima",   // pin LimaVm — no backend implemented
            "enforce": true,
        }),
    )
    .await;
    let verify_ok = body["data"]["verify"]["ok"].as_bool().unwrap_or(false);
    let enforce = ship_platform::config::get_setting("isolation.enforce")
        .unwrap()
        .unwrap_or(serde_json::Value::String("false".into()));
    if verify_ok {
        // Host happens to have lima + the recommender resolves cleanly.
        assert_eq!(status, 200);
        assert_eq!(enforce, serde_json::Value::String("true".into()));
    } else {
        // Verify is red — init refuses, enforce stays at default.
        assert_eq!(status, 400, "verify failed but init didn't return 400: {body:?}");
        assert_ne!(
            enforce,
            serde_json::Value::String("true".into()),
            "enforce should not flip when verify is red: body={body:?}"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn init_writes_enforce_when_explicitly_false() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (status, _body) = post(
        &router,
        "/api/v1/isolation/init",
        serde_json::json!({ "enforce": false }),
    )
    .await;
    assert_eq!(status, 200);
    let enforce = ship_platform::config::get_setting("isolation.enforce")
        .unwrap()
        .unwrap();
    assert_eq!(enforce, serde_json::Value::String("false".into()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn init_records_applied_changes() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, human_auth());
    let (_, body) = post(
        &router,
        "/api/v1/isolation/init",
        serde_json::json!({ "tier": "none", "strategy": "noop" }),
    )
    .await;
    let applied = body["data"]["applied"].as_array().expect("applied array");
    assert!(!applied.is_empty());
    let joined: String = applied
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<&str>>()
        .join(" | ");
    assert!(joined.contains("default_tier"));
    assert!(joined.contains("default_strategy"));
}
