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
        // Set cwd to the tempdir so ProjectRoot::discover() returns None and
        // shipd writes to the singleton machine/daemon/. Otherwise shipd
        // would inherit the test process's cwd (the Ship repo) and write to
        // a per-worktree daemons/<hash>/ dir.
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

        // Per home-layout ADR, daemon IPC files live in machine/daemon/.
        // Wait for the port file (written *after* HTTP bind) — token is
        // written earlier and isn't a reliable readiness signal.
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
        let token = std::fs::read_to_string(&token_path)
            .expect("read daemon token")
            .trim()
            .to_string();

        // Health-poll: the HTTP listener can finish binding a few ms after
        // the port file lands. Poll /api/health until it responds.
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

/// Every response from shipd must carry `X-Shipd-Build` (from build-info).
/// Clients use this for version-skew detection.
#[tokio::test]
async fn responses_carry_x_shipd_build_header() {
    let d = TestDaemon::start().await;
    let res = d.bare().get(d.url("/api/health")).send().await.unwrap();
    let header = res
        .headers()
        .get("x-shipd-build")
        .expect("X-Shipd-Build header must be present");
    let value = header.to_str().expect("header must be ASCII");
    assert!(!value.is_empty(), "X-Shipd-Build value must not be empty");
}

#[tokio::test]
async fn capabilities_requires_auth() {
    let d = TestDaemon::start().await;
    let res = d.bare().get(d.url("/api/v1/capabilities")).send().await.unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn capabilities_with_valid_token() {
    let d = TestDaemon::start().await;
    let res = d.authed().get(d.url("/api/v1/capabilities")).send().await.unwrap();
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
async fn list_jobs_returns_ok() {
    let d = TestDaemon::start().await;
    let res = d.authed().get(d.url("/api/v1/jobs")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn cors_preflight_returns_headers() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .request(reqwest::Method::OPTIONS, d.url("/api/v1/workspaces"))
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
        .request(reqwest::Method::OPTIONS, d.url("/api/v1/workspaces"))
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
    let res = d.bare().get(d.url("/api/v1/mesh/discover")).send().await.unwrap();
    assert_eq!(res.status(), 401);
}

/// End-to-end home-layout validation: a fresh daemon start must produce
/// the canonical `library/` + `machine/<bucket>/` tree, with all daemon
/// IPC files and DBs at their typed-accessor locations and zero legacy
/// root-level paths.
///
/// This is the regression guard for the split-brain class of bug we hit
/// when the migrator and daemon code disagreed on file locations.
#[tokio::test]
async fn fresh_daemon_writes_canonical_home_layout() {
    let d = TestDaemon::start().await;
    let root = d._global_dir.path();

    // Canonical buckets exist.
    assert!(root.join("library").is_dir(), "library/ must exist");
    assert!(root.join("machine").is_dir(), "machine/ must exist");

    // Daemon IPC files live in machine/daemon/.
    for f in ["port", "pid", "token"] {
        assert!(
            root.join("machine").join("daemon").join(f).is_file(),
            "machine/daemon/{f} must exist"
        );
    }

    // Databases live in machine/db/.
    assert!(
        root.join("machine").join("db").join("platform.db").is_file(),
        "machine/db/platform.db must exist"
    );
    assert!(
        root.join("machine").join("db").join("data.db").is_file(),
        "machine/db/data.db must exist"
    );

    // Logs live in machine/logs/.
    assert!(
        root.join("machine").join("logs").join("shipd.log").is_file(),
        "machine/logs/shipd.log must exist"
    );

    // No legacy root-level paths.
    for legacy in [
        "daemon.port",
        "daemon.pid",
        "daemon.token",
        "platform.db",
        "data.db",
        "shipd.log",
        "shipd.pid",
    ] {
        assert!(
            !root.join(legacy).exists(),
            "legacy root-level path {legacy} must not exist after fresh daemon start"
        );
    }
}

#[tokio::test]
async fn mesh_register_and_discover() {
    let d = TestDaemon::start().await;
    let client = d.authed();

    let register = client
        .post(d.url("/api/v1/mesh/register"))
        .json(&serde_json::json!({
            "agent_id": "smoke-agent",
            "capabilities": ["test"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(register.status(), 200);

    let discover = client.get(d.url("/api/v1/mesh/discover")).send().await.unwrap();
    assert_eq!(discover.status(), 200);
    let body: serde_json::Value = discover.json().await.unwrap();
    let agents = &body["data"]["agents"];
    assert!(
        agents.as_array().unwrap().iter().any(|a| a["agent_id"] == "smoke-agent"),
        "registered agent not found in discover: {body}"
    );
}
