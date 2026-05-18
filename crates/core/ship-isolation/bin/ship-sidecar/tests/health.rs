//! Boots the `ship-sidecar` binary on a free port and verifies that
//! `/v1/health` returns the protocol's `HealthResponse` shape.

use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::Command;

fn binary_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Tests run inside the bin crate; the binary lands at workspace target/.
    p.push("../../../../target/debug/ship-sidecar");
    p
}

async fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

#[tokio::test]
async fn health_returns_status_ok() {
    let bin = binary_path();
    if !bin.exists() {
        eprintln!(
            "health test: ship-sidecar binary not found at {} — build the workspace first",
            bin.display()
        );
        return;
    }
    let port = free_port().await;
    let addr = format!("127.0.0.1:{port}");

    let mut child = Command::new(&bin)
        .arg("--addr")
        .arg(&addr)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn ship-sidecar");

    let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
    let ready = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(Some(line)) = stderr.next_line().await {
            if line.contains("listening on") {
                return true;
            }
        }
        false
    })
    .await
    .unwrap_or(false);
    assert!(ready, "sidecar never printed 'listening on'");

    let url = format!("http://{addr}/v1/health");
    let body: serde_json::Value = reqwest::get(&url)
        .await
        .expect("get /v1/health")
        .json()
        .await
        .expect("json");

    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());

    let _ = child.kill().await;
}
