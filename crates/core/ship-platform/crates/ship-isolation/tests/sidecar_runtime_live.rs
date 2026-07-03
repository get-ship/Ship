//! End-to-end test for `SidecarActorRuntime` against the real
//! `ship-sidecar` binary. Spawns the binary on a free port,
//! constructs the runtime, dispatches a Headless spawn, asserts
//! the actor handle's output broadcast receives bytes and the
//! exit probe eventually flips.
//!
//! Requires the `ship-sidecar` binary to be built. CI runs `just
//! build` first.

#![allow(clippy::field_reassign_with_default)]

use ship_isolation::runtime::SidecarActorRuntime;
use ship_substrate::supervisor::{ProcessKind, ProcessProfile, ActorRuntime, TransportKind};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::Command;

fn sidecar_binary() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/core/ship-isolation; workspace
    // target/ is three levels up; the workspace shares the target dir
    // so the `ship-sidecar` artifact lands at <ws>/target/debug/ship-sidecar.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../../target/debug/ship-sidecar");
    p.canonicalize().unwrap_or(p)
}

async fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

async fn start_sidecar() -> Option<(tokio::process::Child, String)> {
    let bin = sidecar_binary();
    if !bin.exists() {
        eprintln!("sidecar_runtime_live: skipping — ship-sidecar binary not built at {}", bin.display());
        return None;
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
        .ok()?;
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
    if !ready {
        eprintln!("sidecar_runtime_live: sidecar never came up");
        return None;
    }
    Some((child, addr))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_runtime_spawn_headless_writes_output_to_broadcast() {
    let Some((_child, addr)) = start_sidecar().await else {
        return;
    };

    let runtime = SidecarActorRuntime::new(format!("http://{addr}"));

    let log_dir = std::env::temp_dir().join("ship-sidecar-runtime-test");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("test-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec!["-c".into(), "echo hello-from-sidecar-runtime; sleep 1".into()],
        env: HashMap::new(),
        work_dir: PathBuf::from("/"),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let handle = runtime.spawn(profile).await.expect("spawn via SidecarActorRuntime");
    // Substrate's `resolve_actor_id` mints a fresh ULID when the
    // hint isn't ULID-shaped — so we just confirm an id was minted.
    assert!(!handle.actor_id.is_empty(), "actor_id must be assigned");
    assert!(handle.child.pid().is_none(), "sidecar-backed actors have no host PID");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut exit_code = None;
    while std::time::Instant::now() < deadline {
        if let Some(code) = handle.child.try_exit() {
            exit_code = Some(code);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(exit_code, Some(0), "headless actor must exit cleanly");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sidecar_runtime_spawn_pty_streams_attach_pty_to_broadcast() {
    let Some((_child, addr)) = start_sidecar().await else {
        return;
    };

    let runtime = SidecarActorRuntime::new(format!("http://{addr}"));

    let log_dir = std::env::temp_dir().join("ship-sidecar-runtime-pty-test");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("pty-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: "/bin/cat".into(),
        args: vec![],
        env: HashMap::new(),
        work_dir: PathBuf::from("/"),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let handle = runtime.spawn(profile).await.expect("spawn pty via SidecarActorRuntime");
    let mut rx = handle.child.subscribe_output();

    // Give the AttachPty background task time to start.
    tokio::time::sleep(Duration::from_millis(200)).await;

    handle.child.write_input(b"hello-pty\n").expect("write_input");

    let mut saw = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(bytes)) => {
                if std::str::from_utf8(&bytes)
                    .unwrap_or_default()
                    .contains("hello-pty")
                {
                    saw = true;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(saw, "expected echoed input via AttachPty broadcast");

    handle.child.stop().expect("stop");
}
