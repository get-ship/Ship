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

use crate::auth::AuthContext;
use crate::routes::rest_api::{ApiState, SharedKernel};

/// Self-contained test daemon — temp dir, kernel, router, clean teardown.
pub struct TestDaemon {
    pub router: Router,
    pub state: ApiState,
    pub kernel: SharedKernel,
    pub temp_dir: TempDir,
}

// Note: state is Arc<ShipRuntime>, so .clone() gives another Arc handle.

/// Parsed JSON response.
pub struct TestResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

impl TestDaemon {
    /// Create a fully wired TestDaemon with temp dir, DB, kernel, and router.
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(temp_dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();

        let kernel = Arc::new(tokio::sync::Mutex::new(
            ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
        ));

        // Create an isolated registry — do NOT use spawn_mesh_service which
        // uses a process-global OnceLock and returns disconnected registries
        // on second+ call.
        let mesh_registry: ship_substrate::services::mesh::SharedMeshRegistry =
            Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

        let api_state: ApiState = Arc::new(
            crate::ship_runtime::ShipRuntime::new_test_with_registry(
                ship_dir.clone(),
                kernel.clone(),
                mesh_registry.clone(),
            ),
        );

        let api_routes = Router::new()
            .route(
                "/actor/spawn",
                axum::routing::post(crate::routes::actor_api::actor_spawn),
            )
            .route(
                "/events/route",
                axum::routing::post(crate::routes::actor_api::event_route),
            )
            .with_state(api_state.clone());

        // Per production routing (apps/shipd/src/router.rs), these live
        // directly at /api/v1/{projects,sessions,agents/active,events}.
        let runtime_routes = Router::new()
            .route(
                "/events",
                axum::routing::get(crate::routes::runtime_api::event_stream),
            )
            .with_state(api_state.clone());

        let capabilities_routes = Router::new()
            .route(
                "/capabilities",
                axum::routing::get(crate::routes::rest_api::capabilities),
            );

        let artifacts_routes = Router::new()
            .route(
                "/artifacts",
                axum::routing::post(crate::routes::artifacts_api::upload_artifact),
            )
            .route(
                "/artifacts/{id}",
                axum::routing::get(crate::routes::artifacts_api::download_artifact),
            )
            .layer(axum::extract::DefaultBodyLimit::max(
                crate::routes::artifacts_api::MAX_UPLOAD_BYTES,
            ))
            .with_state(api_state.clone());


        // Inject a default CLI AuthContext so handlers can extract it.
        // Mount under /api/v1 to match the production router (apps/shipd/
        // src/router.rs); the test paths in artifacts_api_tests and
        // collection_tests use /api/v1/* canonical URLs.
        let default_auth = Self::cli_auth();
        let router = Router::new()
            .nest("/api/v1", api_routes)
            .nest("/api/v1", capabilities_routes)
            .nest("/api/v1", artifacts_routes)
            .nest("/api/v1", runtime_routes)
            .layer(axum::Extension(default_auth));

        Self {
            router,
            state: api_state,
            kernel,
            temp_dir,
        }
    }

    /// A verified **operator-cred** AuthContext — the elevated default for
    /// tests that exercise operator-only behavior (S6, #124). Mirrors the bearer
    /// middleware's operator-session branch: `operator_cred: true` is the SOLE
    /// elevation source. Use this where a test previously relied on
    /// `CallerKind::Cli` to mean operator.
    pub fn operator_auth() -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
            // THE elevation source: a verified operator cred.
            operator_cred: true,
        }
    }

    /// A bare CLI AuthContext — the CLI transport carries `Scope::Full` for the
    /// dying REST axis but grants NO operator elevation (S6): `operator_cred`
    /// stays `false`, so the gate denies C2/C3. For operator behavior use
    /// [`operator_auth`](Self::operator_auth).
    pub fn cli_auth() -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
            operator_cred: false,
        }
    }

    /// Create an MCP AuthContext with the given agent_id.
    pub fn mcp_auth(agent_id: &str) -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Mcp,
            agent_id: Some(agent_id.to_string()),
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
            operator_cred: false,
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

    /// Alias for `post` — clearer name for JSON body requests.
    pub async fn post_json(&self, path: &str, body: &impl serde::Serialize) -> TestResponse {
        self.post(path, body).await
    }

    /// Send a PUT request with a JSON body and parse the response.
    pub async fn put_json(&self, path: &str, body: &impl serde::Serialize) -> TestResponse {
        let json = serde_json::to_vec(body).unwrap();
        let req = Request::builder()
            .method("PUT")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status().as_u16();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
        TestResponse { status, body }
    }

    /// Send a DELETE request and parse the response.
    pub async fn delete(&self, path: &str) -> TestResponse {
        let req = Request::builder()
            .method("DELETE")
            .uri(path)
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status().as_u16();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
        TestResponse { status, body }
    }

    /// Send a multipart POST. `parts` is a list of (name, filename-or-empty,
    /// content-type-or-empty, bytes) tuples. Returns the parsed JSON body.
    pub async fn post_multipart(
        &self,
        path: &str,
        parts: &[(&str, &str, &str, &[u8])],
    ) -> TestResponse {
        let boundary = "----shiptestboundary012345";
        let mut body: Vec<u8> = Vec::new();
        for (name, filename, mime, bytes) in parts {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            if filename.is_empty() {
                body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n").as_bytes(),
                );
            } else {
                body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n"
                    )
                    .as_bytes(),
                );
            }
            if !mime.is_empty() {
                body.extend_from_slice(format!("Content-Type: {mime}\r\n").as_bytes());
            }
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        let req = Request::builder()
            .method("POST")
            .uri(path)
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
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

}
