//! Live Lima test — drives the macOS substrate Isolator end to end.
//!
//! DRAFT for hardware verification: the launch + reap mechanics were proven on
//! a Linux/QEMU POC; the host↔guest transport needs a real Mac (this box's
//! nested-virt Lima port-forwarding is broken regardless of config). Runs
//! wherever a Lima instance is available; auto-skips otherwise.
//!
//! Prereqs (on your Mac):
//!   1. The Lima VM is Linux, so build a *Linux-musl* sidecar on the host:
//!        rustup target add x86_64-unknown-linux-musl   # Intel Mac
//!        # (aarch64-unknown-linux-musl on Apple Silicon)
//!        cargo build -p ship-sidecar --target x86_64-unknown-linux-musl
//!   2. Start the substrate instance (declares the portForwards the transport
//!      needs — Lima's auto-forward is unreliable):
//!        limactl start --name ship-default contrib/lima/ship-substrate.yaml
//!   3. cargo test -p ship-isolation --test lima_sidecar_live
//!
//! Lima mounts your home dir into the VM, so the host-built sidecar is visible
//! at its host path (used as sidecar_path). Override the instance via
//! SHIP_LIMA_TEST_INSTANCE.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use ship_isolation::backends::{Lima, LimaConfig};
use ship_isolation::runtime::SidecarRuntime;
use ship_isolation::Isolator;
use isolation_port::SandboxPolicy;
use ship_substrate::supervisor::{ProcessKind, ProcessProfile, ProcessRuntime, TransportKind};

/// First existing Linux-musl sidecar across both arches / profiles. Lima
/// mirrors the host home into the VM, so the host path doubles as the
/// in-guest sidecar_path.
fn musl_sidecar() -> Option<PathBuf> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../target");
    for arch in ["aarch64-unknown-linux-musl", "x86_64-unknown-linux-musl"] {
        for prof in ["release", "debug"] {
            let p = base.join(arch).join(prof).join("ship-sidecar");
            if p.exists() {
                return Some(p.canonicalize().unwrap_or(p));
            }
        }
    }
    None
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actor_runs_inside_lima_substrate() {
    let Some(sidecar) = musl_sidecar() else {
        eprintln!(
            "lima_sidecar_live: skipping — no Linux-musl sidecar built \
             (cargo build -p ship-sidecar --target <arch>-unknown-linux-musl)"
        );
        return;
    };
    let instance = std::env::var("SHIP_LIMA_TEST_INSTANCE").unwrap_or_else(|_| "ship-default".into());
    let backend = Lima::new(LimaConfig {
        limactl_bin: PathBuf::from("limactl"),
        instance,
        sidecar_path: sidecar.display().to_string(),
    });
    if let Err(e) = backend.available().await {
        eprintln!("lima_sidecar_live: skipping — Lima instance unavailable: {e}");
        return;
    }

    let policy = SandboxPolicy { label: "lima-live".into(), ..SandboxPolicy::default() };
    let handle = backend.prepare(&policy).await.expect("prepare");
    let endpoint = Lima::sidecar_endpoint(&handle).expect("sidecar endpoint");
    let runtime = SidecarRuntime::new(endpoint);

    let log_dir = std::env::temp_dir().join("ship-lima-live");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("lima-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec!["-c".into(), "true".into()],
        env: HashMap::new(),
        work_dir: PathBuf::from("/"),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let actor = runtime.spawn(profile).await.expect("spawn");
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
    assert!(
        exit_code.is_some(),
        "actor never exited — sidecar unreachable in the Lima VM (portForwards set?)"
    );
}
