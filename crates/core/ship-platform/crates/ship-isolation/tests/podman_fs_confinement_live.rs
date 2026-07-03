//! Live podman test — does the Tier-2 container actually ISOLATE the host
//! filesystem?
//!
//! `podman_sidecar_live` proves an actor *runs* in the container; it does
//! NOT prove the container confines. This asserts the security property:
//! an actor inside the sandbox
//!   - CAN read a file inside the bind-mounted workspace, and
//!   - CANNOT read a host file that lives OUTSIDE the workspace mount.
//!
//! This is what catches a mount-config regression that accidentally
//! exposes `$HOME` / host paths — the T-FS-READ guarantee at Tier 2.
//! Auto-skips when podman or the sidecar binary are absent. ubuntu:24.04
//! for glibc parity with the bind-mounted sidecar.

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use ship_isolation::backends::{Docker, DockerConfig};
use ship_isolation::runtime::SidecarRuntime;
use ship_isolation::{FsMount, Isolator, MountMode, NetworkPolicy, SandboxPolicy};
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
async fn container_isolates_host_filesystem() {
    if !podman_available() {
        eprintln!("podman_fs_confinement_live: skipping — podman not reachable");
        return;
    }
    let sidecar = sidecar_binary();
    if !sidecar.exists() {
        eprintln!("podman_fs_confinement_live: skipping — ship-sidecar not built");
        return;
    }

    // Workspace (bind-mounted into the container at the same path) with a
    // file the actor is allowed to read.
    let work = std::env::temp_dir().join(format!("ship-fsconf-ws-{}", std::process::id()));
    std::fs::create_dir_all(&work).unwrap();
    let inside = work.join("inside.txt");
    std::fs::write(&inside, b"INSIDE").unwrap();

    // A host secret OUTSIDE the workspace — must NOT be reachable from the
    // container (it's not mounted; the container fs is the image's).
    let outside =
        std::env::temp_dir().join(format!("ship-fsconf-secret-{}.txt", std::process::id()));
    std::fs::write(&outside, b"SECRET").unwrap();

    let cfg = DockerConfig {
        engine_bin: "podman".into(),
        image: "ubuntu:24.04".into(),
        name_prefix: "ship-podman-fsconf".into(),
        sidecar_bin: sidecar.clone(),
        ..DockerConfig::default()
    };
    let backend = Docker::new(cfg);
    if let Err(e) = backend.available().await {
        eprintln!("podman_fs_confinement_live: skipping — engine unavailable: {e}");
        let _ = std::fs::remove_file(&outside);
        return;
    }

    // Bind-mount only the workspace (identical path passthrough). DenyAll
    // network — we're testing fs, not egress.
    let policy = SandboxPolicy {
        label: "fs-confinement".into(),
        mounts: vec![FsMount {
            host_path: work.clone(),
            sandbox_path: work.clone(),
            mode: MountMode::ReadWrite,
        }],
        network: NetworkPolicy::DenyAll,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = backend.prepare(&policy).await.expect("prepare");
    let endpoint = Docker::sidecar_endpoint(&handle).expect("sidecar endpoint");
    let runtime = SidecarRuntime::new(endpoint);

    // Single assertion: the inside file is readable AND the outside secret
    // is NOT. `cat inside && ! cat outside` exits 0 iff confinement holds.
    let log_dir = std::env::temp_dir().join("ship-podman-fsconf");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("fsconf-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec![
            "-c".into(),
            format!("cat {} && ! cat {}", inside.display(), outside.display()),
        ],
        env: HashMap::new(),
        work_dir: work.clone(),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let actor = runtime.spawn(profile).await.expect("spawn actor");
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
    let _ = std::fs::remove_file(&outside);
    let _ = std::fs::remove_dir_all(&work);

    assert_eq!(
        exit_code,
        Some(0),
        "container must read the workspace file AND fail to read the host \
         secret outside the mount (exit 0 = both held)"
    );
}
