//! End-to-end test for the sidecar's exec / events / signal surface.
//!
//! Spawns the `ship-sidecar` binary on a free TCP port, drives it
//! through the HTTP API, asserts SSE events arrive in the expected
//! shape, then shuts it down gracefully.

use base64::Engine;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::Command;

fn binary_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ship-sidecar"))
}

async fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

async fn start_sidecar() -> (tokio::process::Child, String) {
    let bin = binary_path();
    assert!(bin.exists(), "build ship-sidecar first: {}", bin.display());
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
    assert!(ready, "sidecar never came up");
    (child, addr)
}

#[tokio::test]
async fn exec_then_events_then_exited() {
    let (mut child, addr) = start_sidecar().await;
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();

    // Subscribe to events BEFORE exec so we don't miss anything.
    let events_url = format!("{base}/v1/events");
    let mut events = client
        .get(&events_url)
        .send()
        .await
        .expect("events GET")
        .bytes_stream();

    // Tiny delay so the subscriber attaches to the broadcast channel.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let exec_body = serde_json::json!({
        "session_id": "ses_default",
        "bin": "/bin/sh",
        "args": ["-c", "echo hello-stdout; echo hello-stderr 1>&2; exit 7"],
        "env": {},
        "cwd": "/"
    });
    let exec_resp = client
        .post(format!("{base}/v1/exec"))
        .json(&exec_body)
        .send()
        .await
        .expect("exec POST");
    assert!(exec_resp.status().is_success());

    let mut saw_stdout = false;
    let mut saw_stderr = false;
    let mut saw_exit_code: Option<i32> = None;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let chunk = match tokio::time::timeout(Duration::from_millis(500), events.next()).await {
            Ok(Some(Ok(c))) => c,
            _ => continue,
        };
        for line in std::str::from_utf8(&chunk).unwrap_or("").lines() {
            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };
            let v: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };
            match v["type"].as_str() {
                Some("stdout") => {
                    let b = base64::engine::general_purpose::STANDARD
                        .decode(v["data_b64"].as_str().unwrap_or(""))
                        .unwrap_or_default();
                    if String::from_utf8_lossy(&b).contains("hello-stdout") {
                        saw_stdout = true;
                    }
                }
                Some("stderr") => {
                    let b = base64::engine::general_purpose::STANDARD
                        .decode(v["data_b64"].as_str().unwrap_or(""))
                        .unwrap_or_default();
                    if String::from_utf8_lossy(&b).contains("hello-stderr") {
                        saw_stderr = true;
                    }
                }
                Some("exited") => {
                    saw_exit_code = v["code"].as_i64().map(|c| c as i32);
                }
                _ => {}
            }
        }
        if saw_stdout && saw_stderr && saw_exit_code.is_some() {
            break;
        }
    }

    assert!(saw_stdout, "missed stdout event");
    assert!(saw_stderr, "missed stderr event");
    assert_eq!(saw_exit_code, Some(7), "wrong exit code");

    let _ = child.kill().await;
}

#[tokio::test]
async fn signal_terminates_running_exec() {
    let (mut child, addr) = start_sidecar().await;
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();

    // Subscribe FIRST so we don't miss the Exited event.
    let mut events = client
        .get(format!("{base}/v1/events"))
        .send()
        .await
        .unwrap()
        .bytes_stream();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let exec_body = serde_json::json!({
        "session_id": "ses_default",
        "bin": "/bin/sh",
        "args": ["-c", "trap 'echo got-term; exit 0' TERM; sleep 30"],
        "env": {},
        "cwd": "/"
    });
    let exec: serde_json::Value = client
        .post(format!("{base}/v1/exec"))
        .json(&exec_body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let exec_id = exec["exec_id"].as_str().unwrap().to_string();

    // give sh a moment to install the trap
    tokio::time::sleep(Duration::from_millis(200)).await;

    let sig_resp = client
        .post(format!("{base}/v1/signal"))
        .json(&serde_json::json!({ "exec_id": exec_id, "signal": 15 }))
        .send()
        .await
        .unwrap();
    assert!(sig_resp.status().is_success());

    let mut exited_before_timeout = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let chunk = match tokio::time::timeout(Duration::from_millis(500), events.next()).await {
            Ok(Some(Ok(c))) => c,
            _ => continue,
        };
        for line in std::str::from_utf8(&chunk).unwrap_or("").lines() {
            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };
            let v: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v["type"] == "exited" {
                exited_before_timeout = true;
            }
        }
        if exited_before_timeout {
            break;
        }
    }
    assert!(exited_before_timeout, "exec did not exit after SIGTERM");

    let _ = child.kill().await;
}

#[tokio::test]
async fn stdin_is_delivered_to_running_exec() {
    let (mut child, addr) = start_sidecar().await;
    let base = format!("http://{addr}");
    let client = reqwest::Client::new();

    // Subscribe FIRST so we don't miss tr's HELLO stdout.
    let mut events = client
        .get(format!("{base}/v1/events"))
        .send()
        .await
        .unwrap()
        .bytes_stream();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // sh -c "read line; echo GOT $line" — exits cleanly after one
    // line of stdin, no need to close stdin or send a signal.
    let exec: serde_json::Value = client
        .post(format!("{base}/v1/exec"))
        .json(&serde_json::json!({
            "session_id": "ses_default",
            "bin": "/bin/sh",
            "args": ["-c", "read line; echo GOT $line"],
            "env": HashMap::<String, String>::new(),
            "cwd": "/"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let exec_id = exec["exec_id"].as_str().unwrap().to_string();

    let payload = base64::engine::general_purpose::STANDARD.encode("hello\n");
    client
        .post(format!("{base}/v1/stdin"))
        .json(&serde_json::json!({ "exec_id": exec_id, "data_b64": payload }))
        .send()
        .await
        .unwrap();

    let mut saw_echo = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let chunk = match tokio::time::timeout(Duration::from_millis(500), events.next()).await {
            Ok(Some(Ok(c))) => c,
            _ => continue,
        };
        for line in std::str::from_utf8(&chunk).unwrap_or("").lines() {
            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };
            let v: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v["type"] == "stdout" {
                let b = base64::engine::general_purpose::STANDARD
                    .decode(v["data_b64"].as_str().unwrap_or(""))
                    .unwrap_or_default();
                if String::from_utf8_lossy(&b).contains("GOT hello") {
                    saw_echo = true;
                }
            }
        }
        if saw_echo {
            break;
        }
    }

    assert!(saw_echo, "did not see 'GOT hello' from sh after stdin 'hello'");
    let _ = child.kill().await;
}
