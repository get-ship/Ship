//! Tests for `POST /api/v1/create/{surface}`.
//!
//! Exercises the handler directly (per `rest_api_tests.rs`) so we don't
//! depend on full router wiring. Verifies the skill-scaffold branch
//! writes a SKILL.md with the expected frontmatter + body, and that
//! shape/name validation rejects bad input.

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use http_body_util::BodyExt;
use tempfile::TempDir;

use crate::routes::create_api::{create_surface, CreateRequest};
use crate::routes::rest_api::ApiState;

fn make_api_state(dir: &TempDir) -> ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

#[tokio::test(flavor = "multi_thread")]
async fn scaffold_skill_writes_file_and_returns_route() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = create_surface(
        State(state.clone()),
        Path("skill".into()),
        Json(CreateRequest {
            name: "my-skill".into(),
            purpose: Some("Use when shipping a new feature.".into()),
            shape: "task".into(),
        }),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["surface"], "skill");
    assert_eq!(body["data"]["route"], "/skills/my-skill");

    let path = body["data"]["path"].as_str().unwrap();
    let on_disk = std::fs::read_to_string(path).unwrap();
    assert!(on_disk.contains("name: my-skill"));
    assert!(on_disk.contains("stable-id: my-skill"));
    assert!(on_disk.contains("Use when shipping a new feature."));
    assert!(on_disk.contains("# My Skill"));
    assert!(on_disk.contains("## When to use"));

    let expected = state.ship_dir().join("skills/my-skill/SKILL.md");
    assert_eq!(std::path::Path::new(path), expected);
}

#[tokio::test(flavor = "multi_thread")]
async fn scaffold_skill_uses_shape_specific_body() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    for (shape, marker) in [
        ("task", "## When to use"),
        ("reference", "## Key concepts"),
        ("workflow", "## Steps"),
    ] {
        let resp = create_surface(
            State(state.clone()),
            Path("skill".into()),
            Json(CreateRequest {
                name: format!("skill-{shape}"),
                purpose: None,
                shape: shape.into(),
            }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK, "shape {shape} should succeed");
        let body = body_json(resp).await;
        let path = body["data"]["path"].as_str().unwrap();
        let on_disk = std::fs::read_to_string(path).unwrap();
        assert!(
            on_disk.contains(marker),
            "shape {shape} body missing marker {marker}: {on_disk}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn scaffold_skill_rejects_unknown_shape() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = create_surface(
        State(state),
        Path("skill".into()),
        Json(CreateRequest {
            name: "bad-shape".into(),
            purpose: None,
            shape: "potato".into(),
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn scaffold_skill_rejects_bad_name() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = create_surface(
        State(state),
        Path("skill".into()),
        Json(CreateRequest {
            name: "Bad Name".into(),
            purpose: None,
            shape: "task".into(),
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn scaffold_skill_conflicts_on_existing() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let req = || CreateRequest {
        name: "dup".into(),
        purpose: None,
        shape: "reference".into(),
    };

    let first = create_surface(State(state.clone()), Path("skill".into()), Json(req())).await;
    assert_eq!(first.status(), StatusCode::OK);

    let second = create_surface(State(state), Path("skill".into()), Json(req())).await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
}
