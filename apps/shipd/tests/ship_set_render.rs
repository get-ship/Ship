//! E2E tests for the F-1 SSR walker debug endpoint
//! (`GET /_ship-set/render/{view}`).
//!
//! Same harness shape as `tests/smoke.rs` — spawn a real shipd process,
//! hit the route, assert response shape. The endpoint is unauthenticated
//! (matching the rest of the `_ship-set` debug surface), so we use the
//! bare client.

use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};

struct TestDaemon {
    port: u16,
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
            .current_dir(global_dir.path())
            .env("SHIP_HOME", global_dir.path())
            .env("RUST_LOG", "warn")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("spawn shipd");

        let daemon_dir = global_dir.path().join("machine").join("daemon");
        let port_path = daemon_dir.join("port");
        let token_path = daemon_dir.join("token");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
        loop {
            if port_path.exists() && token_path.exists() {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "shipd did not write port + token within 15s"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let health_url = format!("http://127.0.0.1:{}/api/health", port);
        let client = reqwest::Client::new();
        loop {
            if let Ok(r) = client.get(&health_url).send().await {
                if r.status().is_success() {
                    break;
                }
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "shipd /api/health did not respond within 15s"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Self { port, _global_dir: global_dir, child: Some(child) }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
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

fn port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn agents_view_renders() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_ship-set/render/agents"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let ct = res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("text/html"), "expected text/html, got {ct}");
    let body = res.text().await.unwrap();
    assert!(body.contains("<!-- BEGIN SSR walker output -->"));
    assert!(body.contains("<!-- END SSR walker output -->"));
    assert!(body.contains("data-set-"));
    assert!(body.contains("__set_init__"));
}

#[tokio::test]
async fn unknown_view_404() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_ship-set/render/does-not-exist"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
    let body = res.text().await.unwrap();
    assert!(
        body.contains("views/does-not-exist/view.html.set"),
        "404 body must mention the path it tried — got: {body}"
    );
}

#[tokio::test]
async fn format_json() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_ship-set/render/agents?format=json"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let ct = res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        ct.starts_with("application/json"),
        "expected application/json, got {ct}"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body.get("html").and_then(|v| v.as_str()).is_some());
    let init = body.get("init").expect("init key");
    let agents = init.get("agents").expect("agents key in init");
    assert_eq!(agents, &serde_json::json!({ "kind": "loading" }));
}
