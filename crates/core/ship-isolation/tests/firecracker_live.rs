//! Live Firecracker test — an actor runs inside a real microVM (own
//! kernel), driven over `ship.sidecar.v1` via vsock.
//!
//! Boots a Firecracker microVM through the `Firecracker` Isolator
//! (kernel + pre-baked rootfs with `ship-sidecar` as init on vsock), then
//! drives the production `SidecarActorRuntime` against the
//! `fcvsock:<port>:<uds>` endpoint to spawn an actor and assert it exits
//! cleanly inside the guest. The Tier-3 compat-matrix cell
//! "Linux × T3(firecracker) × sidecar".
//!
//! Auto-skips unless KVM is accessible AND the dev assets exist under
//! `target/fc/` (firecracker binary, `vmlinux`, `rootfs.ext4`) — produced
//! by the Tier-3 spike (see `docs/tier3-build.md`). Productionizing the
//! rootfs/image baking + pinning is a follow-up; this proves the Isolator
//! against the spike assets.

#![cfg(target_os = "linux")]
#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use ship_isolation::backends::{Firecracker, FirecrackerConfig};
use ship_isolation::runtime::SidecarActorRuntime;
use ship_isolation::{Isolator, NetworkPolicy, SandboxPolicy};
use ship_substrate::supervisor::{ActorKind, ActorProfile, ActorRuntime, TransportKind};

fn fc_base() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../../target/fc");
    p
}

/// Locate the dev assets; `None` (skip) if any are missing.
fn assets() -> Option<FirecrackerConfig> {
    let base = fc_base();
    let fc = base.join("release-v1.15.1-x86_64/firecracker-v1.15.1-x86_64");
    let kernel = base.join("vmlinux");
    let rootfs = base.join("rootfs.ext4");
    (fc.exists() && kernel.exists() && rootfs.exists()).then_some(FirecrackerConfig {
        firecracker_bin: fc,
        kernel,
        rootfs,
    })
}

fn kvm_accessible() -> bool {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/kvm")
        .is_ok()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actor_runs_inside_firecracker_microvm() {
    if !kvm_accessible() {
        eprintln!("firecracker_live: skipping — /dev/kvm not accessible");
        return;
    }
    let Some(cfg) = assets() else {
        eprintln!("firecracker_live: skipping — dev assets missing under target/fc/");
        return;
    };
    let backend = Firecracker::new(cfg);
    if let Err(e) = backend.available().await {
        eprintln!("firecracker_live: skipping — {e}");
        return;
    }

    let policy = SandboxPolicy {
        label: "firecracker-live".into(),
        mounts: vec![],
        network: NetworkPolicy::DenyAll,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = backend
        .prepare(&policy)
        .await
        .expect("boot Firecracker microVM + reach sidecar over vsock");
    let endpoint = Firecracker::sidecar_endpoint(&handle).expect("fcvsock endpoint");
    assert!(endpoint.starts_with("fcvsock:"), "endpoint: {endpoint}");

    let runtime = SidecarActorRuntime::new(endpoint);
    let log_dir = std::env::temp_dir().join("ship-fc-live");
    std::fs::create_dir_all(&log_dir).ok();
    // /bin/sh is busybox in the baked rootfs.
    let profile = ActorProfile {
        actor_id_hint: Some("fc-actor".into()),
        kind: ActorKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec!["-c".into(), "echo hello-from-microvm".into()],
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
        .expect("spawn actor inside the microVM over vsock");
    assert!(
        actor.child.pid().is_none(),
        "microVM-backed actor has no host PID"
    );

    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let mut exit_code = None;
    while std::time::Instant::now() < deadline {
        if let Some(code) = actor.child.try_exit() {
            exit_code = Some(code);
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    backend.shutdown(handle).await.expect("shutdown microVM");

    assert_eq!(
        exit_code,
        Some(0),
        "actor inside the Firecracker microVM must exit cleanly"
    );
}
