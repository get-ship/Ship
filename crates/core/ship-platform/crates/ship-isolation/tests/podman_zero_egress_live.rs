//! Live podman test — the zero-egress hardened tier.
//!
//! Two properties in one run, both load-bearing:
//!   1. **T-NET-EGRESS confinement:** with `NetworkPolicy::DenyAll`
//!      (`--network none`) the actor has no network interface — an attempt
//!      to open an outbound TCP connection fails.
//!   2. **The unix-socket hardening's payoff:** the actor still RUNS — i.e.
//!      the sidecar control plane is reachable *with no network at all*,
//!      because it's a bind-mounted unix socket, not a published port.
//!      Under the old published-TCP listener, `--network none` would have
//!      severed the host↔sidecar link; over the socket it doesn't.
//!
//! `exit_code = Some(non-zero)` captures both: `Some(_)` = the actor ran
//! (sidecar reachable sans network); non-zero = egress was blocked.
//! Auto-skips when podman or the sidecar binary are absent.

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deny_all_blocks_egress_yet_sidecar_reachable() {
    if !podman_available() {
        eprintln!("podman_zero_egress_live: skipping — podman not reachable");
        return;
    }
    let sidecar = sidecar_binary();
    if !sidecar.exists() {
        eprintln!("podman_zero_egress_live: skipping — ship-sidecar not built");
        return;
    }

    let cfg = DockerConfig {
        engine_bin: "podman".into(),
        image: "ubuntu:24.04".into(),
        name_prefix: "ship-podman-zeroegress".into(),
        sidecar_bin: sidecar.clone(),
        ..DockerConfig::default()
    };
    let backend = Docker::new(cfg);
    if let Err(e) = backend.available().await {
        eprintln!("podman_zero_egress_live: skipping — engine unavailable: {e}");
        return;
    }

    // DenyAll → `--network none`: no interface. The sidecar still binds its
    // unix socket (bind-mounted, not network) so prepare's readiness probe
    // — and the spawn below — must still succeed.
    let policy = SandboxPolicy {
        label: "zero-egress".into(),
        mounts: vec![],
        network: NetworkPolicy::DenyAll,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = backend
        .prepare(&policy)
        .await
        .expect("prepare must succeed over the unix socket even with --network none");
    let endpoint = Docker::sidecar_endpoint(&handle).expect("sidecar endpoint");
    let runtime = SidecarRuntime::new(endpoint);

    let log_dir = std::env::temp_dir().join("ship-podman-zeroegress");
    std::fs::create_dir_all(&log_dir).ok();
    // Attempt an outbound TCP connection — must fail with no network.
    let profile = ProcessProfile {
        actor_id_hint: Some("zeroegress-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/usr/bin/bash".into(),
        args: vec![
            "-c".into(),
            "exec 3<>/dev/tcp/1.1.1.1/53".into(),
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
        .expect("spawn must succeed — sidecar reachable over the socket sans network");
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let mut exit_code = None;
    while std::time::Instant::now() < deadline {
        if let Some(code) = actor.child.try_exit() {
            exit_code = Some(code);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    backend.shutdown(handle).await.expect("shutdown");

    match exit_code {
        Some(0) => panic!("egress LEAK — outbound TCP succeeded under --network none"),
        Some(_) => { /* ran (sidecar reachable) + egress blocked — both hold */ }
        None => panic!("actor never exited — sidecar unreachable over the socket with no network?"),
    }
}
