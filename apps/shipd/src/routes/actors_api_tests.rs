//! Identity-binding tests for the actor control surface.
//!
//! MCP worker sessions must not be able to act on actors they don't own.
//! The path-id `{id}` is bound to `auth.agent_id` for MCP callers;
//! daemon-token callers pass through.

use axum::Extension;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tempfile::TempDir;

use crate::routes::actors_api::{ActorInputReq, actor_input_handler, kill_tree_handler};
use crate::routes::rest_api::ApiState;
use crate::test_harness::TestDaemon;

fn make_state(dir: &TempDir) -> ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_cannot_input_to_other_actor() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);

    let resp = actor_input_handler(
        State(state),
        Extension(TestDaemon::mcp_auth("agent-self")),
        Path("agent-other".to_string()),
        Json(ActorInputReq { text: "hello".into() }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_cannot_kill_tree_of_other_actor() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);

    let resp = kill_tree_handler(
        State(state),
        Extension(TestDaemon::mcp_auth("agent-self")),
        Path("agent-other".to_string()),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn cli_caller_can_kill_any_actor_tree() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);

    // No matching actor → returns OK with empty stopped list (identity
    // check passed; the request just had nothing to kill).
    let resp = kill_tree_handler(
        State(state),
        Extension(TestDaemon::cli_auth()),
        Path("any-actor".to_string()),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}
