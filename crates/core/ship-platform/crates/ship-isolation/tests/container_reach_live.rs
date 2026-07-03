//! Live podman test — the reachability keystone: an actor running
//! *inside a real container* can reach the host daemon's HTTP plane.
//!
//! Stands up a throwaway TCP listener on the host (the stand-in for the
//! daemon), prepares a podman container in sidecar mode with a network,
//! then drives the production `SidecarRuntime` to spawn an actor
//! that connects back to the host via the URL the production rewrite
//! produces (`Docker::container_reachable_url` → `host.docker.internal`,
//! resolved by the `--add-host=...:host-gateway` the backend now sets).
//! Exit 0 + the host observing the connection proves the path.
//!
//! This is the compat-matrix cell "Linux × T2(podman): container actor →
//! host daemon plane". Auto-skips when podman or the sidecar binary are
//! absent. ubuntu:24.04 for glibc parity (see `podman_sidecar_live`); the
//! in-container client is bash's `/dev/tcp` so no extra packages.

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ship_isolation::backends::{Docker, DockerConfig};
use ship_isolation::runtime::SidecarRuntime;
use ship_isolation::{Isolator, NetworkPolicy, SandboxPolicy};
use ship_substrate::supervisor::{ProcessKind, ProcessProfile, ProcessRuntime, TransportKind};

fn podman_available() -> bool {
    Command::new("podman")
        .args(["version", "--format", "{{.Client.Version}}"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn sidecar_binary() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../../target/debug/ship-sidecar");
    p.canonicalize().unwrap_or(p)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn container_actor_reaches_host_daemon_plane() {
    if !podman_available() {
        eprintln!("container_reach_live: skipping — podman not reachable");
        return;
    }
    let sidecar = sidecar_binary();
    if !sidecar.exists() {
        eprintln!(
            "container_reach_live: skipping — ship-sidecar not built at {}",
            sidecar.display()
        );
        return;
    }

    // Host stand-in for the daemon: bind 0.0.0.0 so the container's
    // host-gateway route reaches it, then accept exactly one connection.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("bind host listener");
    let host_port = listener.local_addr().unwrap().port();
    let accepted = Arc::new(AtomicBool::new(false));
    let accepted_w = accepted.clone();
    let accept_task = tokio::spawn(async move {
        if let Ok(Ok((mut sock, _))) =
            tokio::time::timeout(Duration::from_secs(20), listener.accept()).await
        {
            accepted_w.store(true, Ordering::SeqCst);
            use tokio::io::AsyncWriteExt;
            let _ = sock.write_all(b"ok").await;
        }
    });

    // The production rewrite is what an in-container agent would be handed
    // as $SHIP_DAEMON_URL. Derive the host the actor dials from it, so the
    // test exercises the real function rather than hardcoding the alias.
    let daemon_url = Docker::container_reachable_url(&format!("http://127.0.0.1:{host_port}"));
    assert_eq!(
        daemon_url,
        format!("http://host.docker.internal:{host_port}"),
        "rewrite must target the host-gateway alias"
    );
    let dial_host = "host.docker.internal";

    let cfg = DockerConfig {
        engine_bin: "podman".into(),
        image: "ubuntu:24.04".into(),
        name_prefix: "ship-podman-reach-live".into(),
        sidecar_bin: sidecar.clone(),
        ..DockerConfig::default()
    };
    let backend = Docker::new(cfg);
    if let Err(e) = backend.available().await {
        eprintln!("container_reach_live: skipping — engine unavailable: {e}");
        return;
    }

    // Networked policy (Open) so the container has an interface and the
    // backend injects `--add-host=host.docker.internal:host-gateway`.
    let policy = SandboxPolicy {
        label: "container-reach-live".into(),
        mounts: vec![],
        network: NetworkPolicy::Open,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = backend
        .prepare(&policy)
        .await
        .expect("prepare (podman sidecar mode)");
    let endpoint = Docker::sidecar_endpoint(&handle).expect("sidecar v1 endpoint");

    let runtime = SidecarRuntime::new(endpoint);
    let log_dir = std::env::temp_dir().join("ship-podman-reach-live");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("reach-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/usr/bin/bash".into(),
        // Open a TCP connection from inside the container back to the host
        // gateway. Exit 0 only if the connect succeeds.
        args: vec![
            "-c".into(),
            format!("exec 3<>/dev/tcp/{dial_host}/{host_port} && echo reached"),
        ],
        env: HashMap::new(),
        work_dir: PathBuf::from("/"),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let actor = runtime
        .spawn(profile)
        .await
        .expect("spawn actor inside container via v1");

    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let mut exit_code = None;
    while std::time::Instant::now() < deadline {
        if let Some(code) = actor.child.try_exit() {
            exit_code = Some(code);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    backend.shutdown(handle).await.expect("shutdown (podman rm)");
    accept_task.abort();

    assert_eq!(
        exit_code,
        Some(0),
        "in-container actor must connect to the host daemon plane (exit 0)"
    );
    assert!(
        accepted.load(Ordering::SeqCst),
        "host listener must observe the in-container connection"
    );
}
