//! E2E smoke tests — start a real shipd process and make HTTP requests.

use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};

struct TestDaemon {
    port: u16,
    token: String,
    _global_dir: tempfile::TempDir,
    child: Option<Child>,
}

impl TestDaemon {
    async fn start() -> Self {
        let global_dir = tempfile::tempdir().expect("create temp dir");
        let port = port();

        let bin = env!("CARGO_BIN_EXE_shipd");
        let child = Command::new(bin)
            .args(["--port", &port.to_string(), "--host", "127.0.0.1"])
            .env("SHIP_GLOBAL_DIR", global_dir.path())
            .env("RUST_LOG", "warn")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn shipd");

        // Wait for daemon.token to appear (means server is ready).
        let token_path = global_dir.path().join("daemon.token");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            if token_path.exists() {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "shipd did not write daemon.token within 10s"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let token = std::fs::read_to_string(&token_path)
            .expect("read daemon.token")
            .trim()
            .to_string();

        Self { port, token, _global_dir: global_dir, child: Some(child) }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    fn authed(&self) -> reqwest::Client {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    }

    fn bare(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.start_kill();
        }
    }
}

/// Pick a random high port to avoid collisions between parallel test runs.
fn port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn capabilities_requires_auth() {
    let d = TestDaemon::start().await;
    let res = d.bare().get(d.url("/api/capabilities")).send().await.unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn capabilities_with_valid_token() {
    let d = TestDaemon::start().await;
    let res = d.authed().get(d.url("/api/capabilities")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["authenticated"], true);
    assert!(body["mode"].is_string());
    assert!(body["tier"].is_string());
    assert!(body["features"].is_array());
}

#[tokio::test]
async fn openapi_spec_is_valid_json() {
    let d = TestDaemon::start().await;
    let res = d.bare().get(d.url("/api/openapi.json")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body.get("openapi").is_some(), "missing 'openapi' key");
    assert!(body.get("paths").is_some(), "missing 'paths' key");
}

#[tokio::test]
async fn list_workspaces_returns_ok() {
    let d = TestDaemon::start().await;
    let res = d.authed().get(d.url("/api/runtime/workspaces")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn list_jobs_returns_ok() {
    let d = TestDaemon::start().await;
    let res = d.authed().get(d.url("/api/runtime/jobs")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn cors_preflight_returns_headers() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .request(reqwest::Method::OPTIONS, d.url("/api/runtime/workspaces"))
        .header("origin", "http://localhost:3000")
        .send()
        .await
        .unwrap();
    let acao = res.headers().get("access-control-allow-origin");
    assert_eq!(
        acao.map(|v| v.to_str().unwrap()),
        Some("http://localhost:3000")
    );
}

#[tokio::test]
async fn cors_rejects_bad_origin() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .request(reqwest::Method::OPTIONS, d.url("/api/runtime/workspaces"))
        .header("origin", "https://evil.com")
        .send()
        .await
        .unwrap();
    let acao = res.headers().get("access-control-allow-origin");
    assert!(
        acao.is_none(),
        "evil.com should not receive access-control-allow-origin"
    );
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let d = TestDaemon::start().await;
    let res = d.authed().get(d.url("/api/nonexistent")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn mesh_discover_requires_auth() {
    let d = TestDaemon::start().await;
    let res = d.bare().get(d.url("/api/mesh/discover")).send().await.unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn mesh_register_and_discover() {
    let d = TestDaemon::start().await;
    let client = d.authed();

    let register = client
        .post(d.url("/api/mesh/register"))
        .json(&serde_json::json!({
            "agent_id": "smoke-agent",
            "capabilities": ["test"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(register.status(), 200);

    let discover = client.get(d.url("/api/mesh/discover")).send().await.unwrap();
    assert_eq!(discover.status(), 200);
    let body: serde_json::Value = discover.json().await.unwrap();
    let agents = &body["data"]["agents"];
    assert!(
        agents.as_array().unwrap().iter().any(|a| a["agent_id"] == "smoke-agent"),
        "registered agent not found in discover: {body}"
    );
}
