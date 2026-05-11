//! Shared test daemon for the ship-set render integration suites.
//!
//! Spawns a real `shipd` process in a fresh tempdir. The endpoints under
//! `/_sui/*` are unauthenticated, so we use a bare `reqwest::Client`.

use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};

pub struct TestDaemon {
    port: u16,
    _global_dir: tempfile::TempDir,
    child: Option<Child>,
}

impl TestDaemon {
    pub async fn start() -> Self {
        let global_dir = tempfile::tempdir().expect("create temp dir");
        let port = ephemeral_port();

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

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    pub fn bare(&self) -> reqwest::Client {
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

fn ephemeral_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}
