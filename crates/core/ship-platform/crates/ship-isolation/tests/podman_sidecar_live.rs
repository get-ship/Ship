//! Live podman test — the first proof an actor runs *inside a real
//! container*, driven over `ship.sidecar.v1`.
//!
//! Prepares a podman container with the host `ship-sidecar` binary
//! bind-mounted in as PID 1 (v1 listener published to a host port), then
//! drives the production `SidecarActorRuntime` against that endpoint to
//! spawn an actor and assert it exits cleanly. This is the compat-matrix
//! cell "Linux × T2(podman) × sidecar".
//!
//! Auto-skips when podman is absent or the `ship-sidecar` binary isn't
//! built. The image is `ubuntu:24.04` to match the host's glibc (the
//! bind-mounted sidecar is glibc-dynamic; a musl image like alpine would
//! fail to exec it). Static-musl build is the runtime-format-agnostic
//! follow-up; this cell proves the path with a libc-matched image.

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use ship_isolation::backends::{Docker, DockerConfig};
use ship_isolation::runtime::SidecarActorRuntime;
use ship_isolation::{Isolator, NetworkPolicy, SandboxPolicy};
use ship_substrate::supervisor::{ProcessKind, ProcessProfile, ActorRuntime, TransportKind};

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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actor_runs_inside_real_podman_container_over_v1() {
    if !podman_available() {
        eprintln!("podman_sidecar_live: skipping — podman not reachable");
        return;
    }
    let sidecar = sidecar_binary();
    if !sidecar.exists() {
        eprintln!(
            "podman_sidecar_live: skipping — ship-sidecar not built at {}",
            sidecar.display()
        );
        return;
    }

    let cfg = DockerConfig {
        engine_bin: "podman".into(),
        image: "ubuntu:24.04".into(),
        name_prefix: "ship-podman-sidecar-live".into(),
        // Sidecar mode: bind-mount this binary as PID 1, publish v1.
        sidecar_bin: sidecar.clone(),
        ..DockerConfig::default()
    };
    let backend = Docker::new(cfg);

    if let Err(e) = backend.available().await {
        eprintln!("podman_sidecar_live: skipping — engine unavailable: {e}");
        return;
    }

    // Open network so the published sidecar port (`-p`) works; egress
    // policy is mediated separately in production. No mounts needed —
    // the actor just echoes.
    let policy = SandboxPolicy {
        label: "podman-sidecar-live".into(),
        mounts: vec![],
        network: NetworkPolicy::Open,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = match backend.prepare(&policy).await {
        Ok(h) => h,
        Err(e) => panic!("prepare (podman sidecar mode) failed: {e}"),
    };
    assert_eq!(handle.backend, "docker");

    let endpoint = Docker::sidecar_endpoint(&handle)
        .expect("sidecar mode must publish a v1 endpoint");

    // Drive the PRODUCTION runtime against the in-container sidecar.
    let runtime = SidecarActorRuntime::new(endpoint);
    let log_dir = std::env::temp_dir().join("ship-podman-sidecar-live");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("podman-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec!["-c".into(), "echo hello-from-container".into()],
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
    assert!(!actor.actor_id.is_empty(), "actor_id assigned");
    assert!(
        actor.child.pid().is_none(),
        "container-backed actor has no host PID"
    );

    // The echo runs to completion inside the container.
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut exit_code = None;
    while std::time::Instant::now() < deadline {
        if let Some(code) = actor.child.try_exit() {
            exit_code = Some(code);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        exit_code,
        Some(0),
        "actor inside the podman container must exit cleanly"
    );

    backend.shutdown(handle).await.expect("shutdown (podman rm)");
}
