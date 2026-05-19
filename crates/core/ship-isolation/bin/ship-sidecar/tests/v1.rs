//! End-to-end test for ship.sidecar.v1 via the ConnectRPC client.
//!
//! Spawns the `ship-sidecar` binary on a free port, drives it through
//! the generated `HandshakeClient` / `ProcessesClient` / `EventsClient`,
//! asserts each shape, then shuts it down.

#![allow(clippy::field_reassign_with_default)]

use connectrpc::client::{ClientConfig, HttpClient};
use ship_isolation::sidecar::proto::*;
use ship_isolation::sidecar::proto::__buffa::view::oneof::envelope as env_view;
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

fn clients(addr: &str) -> (HandshakeClient<HttpClient>, ProcessesClient<HttpClient>, EventsClient<HttpClient>) {
    let http = HttpClient::plaintext();
    let uri: http::Uri = format!("http://{addr}").parse().unwrap();
    let config = ClientConfig::new(uri);
    (
        HandshakeClient::new(http.clone(), config.clone()),
        ProcessesClient::new(http.clone(), config.clone()),
        EventsClient::new(http, config),
    )
}

#[tokio::test]
async fn handshake_returns_v1_capabilities() {
    let (_child, addr) = start_sidecar().await;
    let (hs, _, _) = clients(&addr);

    let resp = hs.hello(HelloRequest::default()).await.expect("hello");
    let view = resp.into_view();
    assert!(view.protocol_version.starts_with("1."));
    assert!(!view.sidecar_version.is_empty());
    let caps: Vec<&str> = view.capabilities.iter().copied().collect();
    assert!(caps.contains(&"processes"), "missing processes capability");
    assert!(caps.contains(&"events.acp"), "missing events.acp capability");

}

#[tokio::test]
async fn run_unary_returns_stdout_and_exit_code() {
    let (_child, addr) = start_sidecar().await;
    let (_, procs, _) = clients(&addr);

    let mut req = ProcessRunRequest::default();
    req.bin = "/bin/sh".into();
    req.args = vec!["-c".into(), "echo hello-stdout; exit 7".into()];
    req.cwd = "/".into();

    let resp = procs.run(req).await.expect("run");
    let view = resp.into_view();
    assert_eq!(view.exit_code, 7);
    let stdout: &[u8] = view.stdout;
    assert!(
        std::str::from_utf8(stdout).unwrap().contains("hello-stdout"),
        "stdout missing expected line: {:?}",
        std::str::from_utf8(stdout)
    );
}

#[tokio::test]
async fn create_then_event_stream_carries_output_and_exit() {
    let (_child, addr) = start_sidecar().await;
    let (_, procs, events) = clients(&addr);

    // Subscribe to events BEFORE creating the process so we don't miss anything.
    let mut stream = events.stream().await.expect("events.stream");

    // Tiny delay for the subscriber to attach.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut req = ProcessCreateRequest::default();
    req.bin = "/bin/sh".into();
    req.args = vec![
        "-c".into(),
        "echo hello-out; echo hello-err 1>&2; exit 5".into(),
    ];
    req.cwd = "/".into();
    req.id_hint = "proc-test".into();
    let info = procs.create(req).await.expect("create");
    assert_eq!(info.into_view().id, "proc-test");

    let mut saw_out = false;
    let mut saw_err = false;
    let mut saw_exit: Option<i32> = None;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let msg = tokio::time::timeout(Duration::from_millis(500), stream.message()).await;
        let Ok(Ok(Some(env))) = msg else { continue };
        match env.payload.as_ref() {
            Some(env_view::Payload::ProcessOutput(o)) => {
                let data: &[u8] = o.data;
                let text = std::str::from_utf8(data).unwrap_or_default();
                if text.contains("hello-out") {
                    saw_out = true;
                }
                if text.contains("hello-err") {
                    saw_err = true;
                }
            }
            Some(env_view::Payload::ProcessExited(x)) => {
                saw_exit = Some(x.exit_code);
            }
            _ => {}
        }
        if saw_out && saw_err && saw_exit.is_some() {
            break;
        }
    }

    assert!(saw_out, "did not receive stdout event");
    assert!(saw_err, "did not receive stderr event");
    assert_eq!(saw_exit, Some(5), "wrong exit code");

}

#[tokio::test]
async fn write_input_is_delivered_to_running_process() {
    let (_child, addr) = start_sidecar().await;
    let (_, procs, events) = clients(&addr);

    let mut stream = events.stream().await.expect("events.stream");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut req = ProcessCreateRequest::default();
    req.bin = "/bin/sh".into();
    req.args = vec!["-c".into(), "read line; echo GOT $line".into()];
    req.cwd = "/".into();
    req.id_hint = "proc-stdin".into();
    let _ = procs.create(req).await.expect("create");

    let mut w = ProcessWriteInputRequest::default();
    w.id = "proc-stdin".into();
    w.data = b"hello\n".to_vec();
    let resp = procs.write_input(w).await.expect("write_input");
    assert!(resp.into_view().ok);

    let mut saw_echo = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let msg = tokio::time::timeout(Duration::from_millis(500), stream.message()).await;
        let Ok(Ok(Some(env))) = msg else { continue };
        if let Some(env_view::Payload::ProcessOutput(o)) = env.payload.as_ref() {
            let data: &[u8] = o.data;
            if std::str::from_utf8(data).unwrap_or_default().contains("GOT hello") {
                saw_echo = true;
                break;
            }
        }
    }
    assert!(saw_echo, "did not see 'GOT hello' echoed back");

}

#[tokio::test]
async fn stop_terminates_process_with_grace() {
    let (_child, addr) = start_sidecar().await;
    let (_, procs, events) = clients(&addr);

    let mut stream = events.stream().await.expect("events.stream");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut req = ProcessCreateRequest::default();
    req.bin = "/bin/sh".into();
    // Bare sleep — sh's default disposition for SIGTERM is to exit
    // (no trap installed), which avoids subshell signal-delivery
    // quirks that bite when the foreground command is a long sleep.
    req.args = vec!["-c".into(), "exec sleep 30".into()];
    req.cwd = "/".into();
    req.id_hint = "proc-stop".into();
    let _ = procs.create(req).await.expect("create");

    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut stop = ProcessStopRequest::default();
    stop.id = "proc-stop".into();
    stop.grace_ms = 2000;
    let resp = procs.stop(stop).await.expect("stop");
    assert!(resp.into_view().ok);

    let mut saw_exit = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while tokio::time::Instant::now() < deadline {
        let msg = tokio::time::timeout(Duration::from_millis(500), stream.message()).await;
        let Ok(Ok(Some(env))) = msg else { continue };
        if let Some(env_view::Payload::ProcessExited(_)) = env.payload.as_ref() {
            saw_exit = true;
            break;
        }
    }
    assert!(saw_exit, "stop did not lead to ProcessExited");

}

#[tokio::test]
async fn list_and_get_round_trip() {
    let (_child, addr) = start_sidecar().await;
    let (_, procs, _) = clients(&addr);

    let mut req = ProcessCreateRequest::default();
    req.bin = "/bin/sh".into();
    req.args = vec!["-c".into(), "sleep 10".into()];
    req.cwd = "/".into();
    req.id_hint = "proc-list".into();
    let _ = procs.create(req).await.expect("create");

    let list = procs.list(ProcessListRequest::default()).await.expect("list");
    let lv = list.into_view();
    let processes: Vec<_> = lv.processes.iter().collect();
    assert!(
        processes.iter().any(|p| p.id == "proc-list"),
        "list did not include proc-list"
    );

    let mut get_req = ProcessGetRequest::default();
    get_req.id = "proc-list".into();
    let got = procs.get(get_req).await.expect("get");
    assert_eq!(got.into_view().id, "proc-list");

}

#[tokio::test]
async fn get_unknown_id_returns_not_found() {
    let (_child, addr) = start_sidecar().await;
    let (_, procs, _) = clients(&addr);

    let mut req = ProcessGetRequest::default();
    req.id = "does-not-exist".into();
    let err = procs.get(req).await.expect_err("expected error");
    assert_eq!(err.code, connectrpc::ErrorCode::NotFound);

}

#[tokio::test]
async fn get_after_exit_returns_exited_state_with_code() {
    // Regression: the host-side `wait_for_exit` driver subscribes to
    // events AFTER calling Create — if the process exits in between,
    // the broadcast channel doesn't replay. Get must be the
    // race-closing fallback: it must return PROCESS_STATE_EXITED +
    // exit_code for a process that completed before any subscription.
    let (_child, addr) = start_sidecar().await;
    let (_, procs, _) = clients(&addr);

    let mut req = ProcessCreateRequest::default();
    req.bin = "/bin/sh".into();
    req.args = vec!["-c".into(), "exit 3".into()];
    req.cwd = "/".into();
    req.id_hint = "proc-fast-exit".into();
    let _ = procs.create(req).await.expect("create");

    // Give the watcher a moment to reap.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut get = ProcessGetRequest::default();
    get.id = "proc-fast-exit".into();
    let info = procs.get(get).await.expect("get");
    let v = info.into_view();
    assert_eq!(
        v.state.as_known(),
        Some(ProcessState::PROCESS_STATE_EXITED),
        "Get must report EXITED state after the process completed"
    );
    assert_eq!(v.exit_code, 3, "Get must surface the exit code");
}

#[tokio::test]
async fn pty_create_returns_unimplemented_in_v1() {
    let (_child, addr) = start_sidecar().await;
    let (_, procs, _) = clients(&addr);

    let mut req = ProcessCreateRequest::default();
    req.bin = "/bin/sh".into();
    req.args = vec!["-c".into(), "echo nope".into()];
    req.cwd = "/".into();
    req.pty = true;
    let err = procs.create(req).await.expect_err("expected unimplemented");
    assert_eq!(err.code, connectrpc::ErrorCode::Unimplemented);

}
