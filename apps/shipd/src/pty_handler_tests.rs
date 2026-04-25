use super::{workspace_pty, MAX_PTY_CONNECTIONS};
use crate::auth::{AuthContext, Scope};
use crate::rest_api::ApiState;
use std::sync::Arc;
use tempfile::TempDir;

fn make_api_state() -> (ApiState, TempDir) {
    let tmp = TempDir::new().unwrap();
    let ship_dir = runtime::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = runtime::events::init_kernel_router(ship_dir.clone()).unwrap();
    let state = Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel));
    (state, tmp)
}

fn full_auth() -> AuthContext {
    AuthContext {
        caller_kind: runtime::events::CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn pty_returns_404_for_unknown_workspace() {
    use http_body_util::BodyExt;
    let (state, _tmp) = make_api_state();

    let req = axum::http::Request::builder()
        .uri("/api/v1/workspaces/nonexistent/pty")
        .header("host", "localhost")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(axum::body::Body::empty())
        .unwrap();

    let app = axum::Router::new()
        .route(
            "/api/v1/workspaces/{id}/pty",
            axum::routing::get(workspace_pty),
        )
        .layer(axum::Extension(full_auth()))
        .with_state(state);

    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let val: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(val["error"], "workspace not found");
}

#[tokio::test(flavor = "multi_thread")]
async fn pty_returns_404_when_no_tmux_session() {
    use http_body_util::BodyExt;
    let (state, _tmp) = make_api_state();

    runtime::workspace::create_workspace(
        state.ship_dir(),
        runtime::workspace::CreateWorkspaceRequest {
            branch: "feat/pty-no-tmux".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let ws = runtime::workspace::get_workspace(state.ship_dir(), "feat/pty-no-tmux")
        .unwrap()
        .unwrap();
    let ws_id = ws.id.clone();

    let req = axum::http::Request::builder()
        .uri(format!("/api/v1/workspaces/{ws_id}/pty"))
        .header("host", "localhost")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(axum::body::Body::empty())
        .unwrap();

    let app = axum::Router::new()
        .route(
            "/api/v1/workspaces/{id}/pty",
            axum::routing::get(workspace_pty),
        )
        .layer(axum::Extension(full_auth()))
        .with_state(state);

    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let val: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(val["error"], "no tmux session for this workspace");
}

#[tokio::test(flavor = "multi_thread")]
async fn pty_returns_429_when_over_connection_limit() {
    use http_body_util::BodyExt;
    let (state, _tmp) = make_api_state();

    runtime::workspace::create_workspace(
        state.ship_dir(),
        runtime::workspace::CreateWorkspaceRequest {
            branch: "feat/pty-limit".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let ws = runtime::workspace::get_workspace(state.ship_dir(), "feat/pty-limit")
        .unwrap()
        .unwrap();
    let ws_id = ws.id.clone();

    runtime::workspace::set_workspace_tmux_session(
        state.ship_dir(),
        "feat/pty-limit",
        Some("test-tmux-session"),
    )
    .unwrap();

    // Pre-fill the connection counter to the limit.
    {
        let mut conns = state.pty_connections().lock().await;
        conns.insert(ws_id.clone(), MAX_PTY_CONNECTIONS);
    }

    let req = axum::http::Request::builder()
        .uri(format!("/api/v1/workspaces/{ws_id}/pty"))
        .header("host", "localhost")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(axum::body::Body::empty())
        .unwrap();

    let app = axum::Router::new()
        .route(
            "/api/v1/workspaces/{id}/pty",
            axum::routing::get(workspace_pty),
        )
        .layer(axum::Extension(full_auth()))
        .with_state(state);

    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let val: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(val["error"]
        .as_str()
        .unwrap_or("")
        .contains("too many concurrent"));
}
