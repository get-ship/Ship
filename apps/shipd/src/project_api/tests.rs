//! End-to-end tests for the project REST API endpoints.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use tempfile::TempDir;

use super::*;

fn full_auth() -> axum::Extension<crate::auth::AuthContext> {
    axum::Extension(crate::auth::AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        session_id: None,
        scopes: vec![crate::auth::Scope::Full],
    })
}

/// Unique branch name per test to avoid DB collisions across concurrent tests.
fn test_branch(name: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("test/{name}-{n}")
}

/// Set up a project_api test environment. Returns `(branch, ApiState)`
/// — branch is just a label for tests; runtime no longer tracks workspaces.
fn create_test_workspace(dir: &TempDir) -> (String, crate::rest_api::ApiState) {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let branch = test_branch("ws");
    (branch, make_api_state_with_dir(&ship_dir))
}

fn make_api_state(dir: &TempDir) -> crate::rest_api::ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    make_api_state_with_dir(&ship_dir)
}

fn make_api_state_with_dir(ship_dir: &std::path::Path) -> crate::rest_api::ApiState {
    std::fs::create_dir_all(ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(ship_dir).unwrap();
    let kernel = ship_substrate::events::init_kernel_router(ship_dir.to_path_buf()).unwrap();
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(
        ship_dir.to_path_buf(),
        kernel,
    ))
}

// ---- Session files ----

#[tokio::test(flavor = "multi_thread")]
async fn list_session_files_empty() {
    let dir = TempDir::new().unwrap();
    let (branch, state) = create_test_workspace(&dir);

    let (status, body) = list_session_files(State(state), Path(branch)).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.0.data["files"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_session_files_with_files() {
    let dir = TempDir::new().unwrap();
    let session_dir = dir.path().join(".artifacts");
    std::fs::create_dir_all(&session_dir).unwrap();
    std::fs::write(session_dir.join("notes.md"), "hello").unwrap();
    std::fs::write(session_dir.join("data.json"), "{}").unwrap();

    let (branch, state) = create_test_workspace(&dir);
    let (status, body) = list_session_files(State(state), Path(branch)).await.unwrap();
    assert_eq!(status, StatusCode::OK);

    let files = body.0.data["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    let names: Vec<&str> = files.iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"notes.md"));
    assert!(names.contains(&"data.json"));

    // Verify type detection
    let md_file = files.iter().find(|f| f["name"] == "notes.md").unwrap();
    assert_eq!(md_file["type"], "markdown");
    let json_file = files.iter().find(|f| f["name"] == "data.json").unwrap();
    assert_eq!(json_file["type"], "json");
}

#[tokio::test(flavor = "multi_thread")]
async fn read_session_file_exists() {
    let dir = TempDir::new().unwrap();
    let session_dir = dir.path().join(".artifacts");
    std::fs::create_dir_all(&session_dir).unwrap();
    std::fs::write(session_dir.join("spec.md"), "# My Spec").unwrap();

    let (branch, state) = create_test_workspace(&dir);
    let (status, body) =
        read_session_file(State(state), Path((branch, "spec.md".to_string()))).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.0.data["content"], "# My Spec");
}

#[tokio::test(flavor = "multi_thread")]
async fn read_session_file_not_found() {
    let dir = TempDir::new().unwrap();
    let (branch, state) = create_test_workspace(&dir);

    let (status, body) = read_session_file(State(state), Path((branch, "nope.txt".to_string())))
        .await
        .unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!body.0.ok);
}

#[tokio::test(flavor = "multi_thread")]
async fn write_session_file_creates() {
    let dir = TempDir::new().unwrap();
    let (branch, state) = create_test_workspace(&dir);

    let (status, _) = write_session_file(
        State(state.clone()),
        full_auth(),
        Path((branch.clone(), "new.txt".to_string())),
        axum::Json(session_files::WriteFileReq {
            content: "created".to_string(),
        }),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);

    let (status, body) =
        read_session_file(State(state), Path((branch, "new.txt".to_string()))).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.0.data["content"], "created");
}

#[tokio::test(flavor = "multi_thread")]
async fn write_session_file_creates_directory() {
    let dir = TempDir::new().unwrap();
    let (branch, state) = create_test_workspace(&dir);

    let (status, _) = write_session_file(
        State(state.clone()),
        full_auth(),
        Path((branch.clone(), "sub/dir/file.txt".to_string())),
        axum::Json(session_files::WriteFileReq {
            content: "nested".to_string(),
        }),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);

    let (status, body) = read_session_file(State(state), Path((branch, "sub/dir/file.txt".to_string())))
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.0.data["content"], "nested");
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_session_file() {
    let dir = TempDir::new().unwrap();
    let session_dir = dir.path().join(".artifacts");
    std::fs::create_dir_all(&session_dir).unwrap();
    std::fs::write(session_dir.join("doomed.txt"), "bye").unwrap();

    let (branch, state) = create_test_workspace(&dir);

    let (status, _) = super::delete_session_file(State(state.clone()), full_auth(), Path((branch.clone(), "doomed.txt".to_string())))
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);

    let (status, _) = read_session_file(State(state), Path((branch, "doomed.txt".to_string())))
        .await
        .unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---- Agents ----

#[tokio::test(flavor = "multi_thread")]
async fn list_agents_empty() {
    let dir = TempDir::new().unwrap();
    let (branch, state) = create_test_workspace(&dir);

    let (status, body) = list_agents(State(state), Path(branch)).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.0.data["agents"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn list_agents_reads_jsonc() {
    let dir = TempDir::new().unwrap();
    let agents_dir = ship_substrate::ShipHome::project_ship_dir(dir.path()).join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(
        agents_dir.join("test-agent.jsonc"),
        r#"{
    // line comment
    "name": "Test Agent",
    "description": "A test agent",
    "skills": ["skill-a"],
    "providers": ["claude"]
}"#,
    )
    .unwrap();

    let (branch, state) = create_test_workspace(&dir);
    let (status, body) = list_agents(State(state), Path(branch)).await.unwrap();
    assert_eq!(status, StatusCode::OK);

    let agents = body.0.data["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["id"], "test-agent");
    assert_eq!(agents[0]["name"], "Test Agent");
    assert_eq!(agents[0]["description"], "A test agent");
    assert_eq!(agents[0]["skills"], serde_json::json!(["skill-a"]));
    assert_eq!(agents[0]["providers"], serde_json::json!(["claude"]));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_agents_strips_comments() {
    let dir = TempDir::new().unwrap();
    let agents_dir = ship_substrate::ShipHome::project_ship_dir(dir.path()).join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    std::fs::write(
        agents_dir.join("commented.jsonc"),
        r#"{
    // single-line comment
    "name": "Commented",
    /* block
       comment */
    "description": "has comments"
}"#,
    )
    .unwrap();

    let (branch, state) = create_test_workspace(&dir);
    let (status, body) = list_agents(State(state), Path(branch)).await.unwrap();
    assert_eq!(status, StatusCode::OK);

    let agents = body.0.data["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["name"], "Commented");
    assert_eq!(agents[0]["description"], "has comments");
}

// ---- Event emit ----

#[tokio::test(flavor = "multi_thread")]
async fn emit_event_routes_to_kernel() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let (status, body) = emit_event(
        axum::extract::State(state),
        None,
        axum::Json(EmitEventReq {
            event_type: "test.event".to_string(),
            entity_id: "entity-1".to_string(),
            workspace_id: None,
            payload: serde_json::json!({"key": "value"}),
            actor_id: None,
            elevated: None,
            validate_domain: None,
            actor: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);
    assert!(body.0.ok);
}

// ---- Error cases ----

/// Path traversal via `..` must be blocked by canonicalization in safe_resolve.
#[tokio::test(flavor = "multi_thread")]
async fn session_file_path_traversal_blocked() {
    let dir = TempDir::new().unwrap();
    let session_dir = dir.path().join(".artifacts");
    std::fs::create_dir_all(&session_dir).unwrap();
    std::fs::write(session_dir.join("legit.txt"), "ok").unwrap();
    std::fs::write(dir.path().join("secret.txt"), "do not read").unwrap();

    let (branch, state) = create_test_workspace(&dir);

    // Read traversal must be rejected.
    let result = read_session_file(
        State(state.clone()),
        Path((branch.clone(), "../secret.txt".to_string())),
    )
    .await;
    assert!(result.is_err(), "read traversal must be rejected");

    // Write traversal must be rejected.
    let result = write_session_file(
        State(state),
        full_auth(),
        Path((branch, "../traversed.txt".to_string())),
        axum::Json(session_files::WriteFileReq {
            content: "escaped".to_string(),
        }),
    )
    .await;
    assert!(result.is_err(), "write traversal must be rejected");
    assert!(
        !dir.path().join("traversed.txt").exists(),
        "write traversal must not create file outside session dir"
    );
}

