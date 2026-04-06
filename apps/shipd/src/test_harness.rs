//! Reusable test harness for daemon integration tests.
//!
//! `TestDaemon` spins up the full Axum router with a real kernel, mesh
//! service, and temp directory. Requests go through tower::ServiceExt
//! (no TCP listener needed).

use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::rest_api::{ApiState, SharedKernel};

/// Self-contained test daemon — temp dir, kernel, router, clean teardown.
pub struct TestDaemon {
    pub router: Router,
    pub state: ApiState,
    pub kernel: SharedKernel,
    pub temp_dir: TempDir,
}

/// Parsed JSON response.
pub struct TestResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

impl TestDaemon {
    /// Create a fully wired TestDaemon with temp dir, DB, kernel, and router.
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let ship_dir = temp_dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        runtime::db::ensure_db_at(&ship_dir).unwrap();

        let kernel = Arc::new(tokio::sync::Mutex::new(
            runtime::events::KernelRouter::new(ship_dir.clone()).unwrap(),
        ));

        // Create an isolated registry — do NOT use spawn_mesh_service which
        // uses a process-global OnceLock and returns disconnected registries
        // on second+ call.
        let mesh_registry: runtime::services::mesh::SharedMeshRegistry =
            Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

        let api_state = ApiState {
            kernel: kernel.clone(),
            mesh_registry: mesh_registry.clone(),
            agent_mailboxes: Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            pty_connections: Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            ship_dir: ship_dir.clone(),
        };

        let api_routes = Router::new()
            .route(
                "/mesh/register",
                axum::routing::post(crate::rest_api::mesh_register),
            )
            .route(
                "/mesh/send",
                axum::routing::post(crate::rest_api::mesh_send),
            )
            .route(
                "/mesh/broadcast",
                axum::routing::post(crate::rest_api::mesh_broadcast),
            )
            .route(
                "/mesh/discover",
                axum::routing::get(crate::rest_api::mesh_discover),
            )
            .route(
                "/mesh/status",
                axum::routing::post(crate::rest_api::mesh_status_update),
            )
            .route(
                "/mesh/events/{agent_id}",
                axum::routing::get(crate::rest_api::mesh_events),
            )
            .route(
                "/actor/spawn",
                axum::routing::post(crate::actor_api::actor_spawn),
            )
            .route(
                "/events/route",
                axum::routing::post(crate::actor_api::event_route),
            )
            .with_state(api_state.clone());

        let runtime_routes = Router::new()
            .route(
                "/workspaces",
                axum::routing::get(crate::runtime_api::list_workspaces),
            )
            .route(
                "/sessions",
                axum::routing::get(crate::runtime_api::list_sessions),
            )
            .route(
                "/agents",
                axum::routing::get(crate::runtime_api::list_agents),
            )
            .route(
                "/events",
                axum::routing::get(crate::runtime_api::event_stream),
            )
            .with_state(api_state.clone());

        let router = Router::new()
            .nest("/api", api_routes)
            .nest("/api/runtime", runtime_routes);

        Self {
            router,
            state: api_state,
            kernel,
            temp_dir,
        }
    }

    /// Send a GET request and parse the JSON body.
    pub async fn get(&self, path: &str) -> TestResponse {
        let req = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .unwrap();

        let resp = self
            .router
            .clone()
            .oneshot(req)
            .await
            .unwrap();

        let status = resp.status().as_u16();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
        TestResponse { status, body }
    }

    /// Send a POST request with a JSON body and parse the response.
    pub async fn post(&self, path: &str, body: &impl serde::Serialize) -> TestResponse {
        let json = serde_json::to_vec(body).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let resp = self
            .router
            .clone()
            .oneshot(req)
            .await
            .unwrap();

        let status = resp.status().as_u16();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
        TestResponse { status, body }
    }

    /// Send a GET request and return the raw response (for SSE streams).
    pub async fn get_raw(&self, path: &str) -> axum::response::Response {
        let req = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .unwrap();

        self.router.clone().oneshot(req).await.unwrap()
    }

    /// Register an agent via POST /api/mesh/register.
    pub async fn register_agent(&self, agent_id: &str, caps: &[&str]) -> TestResponse {
        self.post(
            "/api/mesh/register",
            &serde_json::json!({
                "agent_id": agent_id,
                "capabilities": caps,
            }),
        )
        .await
    }

    /// Send a mesh message via POST /api/mesh/send.
    pub async fn mesh_send(
        &self,
        from: &str,
        to: &str,
        body: serde_json::Value,
    ) -> TestResponse {
        self.post(
            "/api/mesh/send",
            &serde_json::json!({ "from": from, "to": to, "body": body }),
        )
        .await
    }

    /// Broadcast via POST /api/mesh/broadcast.
    pub async fn mesh_broadcast(
        &self,
        from: &str,
        body: serde_json::Value,
        cap_filter: Option<&str>,
    ) -> TestResponse {
        let mut payload = serde_json::json!({ "from": from, "body": body });
        if let Some(cap) = cap_filter {
            payload["capability_filter"] = serde_json::json!(cap);
        }
        self.post("/api/mesh/broadcast", &payload).await
    }
}
