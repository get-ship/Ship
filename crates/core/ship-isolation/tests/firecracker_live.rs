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

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use ship_isolation::backends::{Firecracker, FirecrackerConfig};
use ship_isolation::runtime::SidecarActorRuntime;
use ship_isolation::{FsMount, Isolator, MountMode, NetworkPolicy, SandboxPolicy};
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
        app_plane_daemon: None,
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

/// Work-unit-copy workspace: a host dir is seeded into the microVM (data
/// drive), the agent reads a seeded file + writes a new one inside the
/// guest, and the new file copies back out to the host at shutdown.
/// Firecracker has no virtio-fs, so this proves the snapshot-in/copy-out
/// model (see `docs/tier3-build.md`).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn microvm_workspace_round_trips() {
    if !kvm_accessible() {
        eprintln!("firecracker_live: skipping — /dev/kvm not accessible");
        return;
    }
    let Some(cfg) = assets() else {
        eprintln!("firecracker_live: skipping — dev assets missing");
        return;
    };
    let backend = Firecracker::new(cfg);
    if backend.available().await.is_err() {
        return;
    }

    let work = std::env::temp_dir().join(format!("ship-fcws-{}", std::process::id()));
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join("seed.txt"), b"SEED-DATA").unwrap();
    let out = work.join("out.txt");
    let _ = std::fs::remove_file(&out);

    // identical-path workspace mount (host == guest path).
    let policy = SandboxPolicy {
        label: "fc-workspace".into(),
        mounts: vec![FsMount {
            host_path: work.clone(),
            sandbox_path: work.clone(),
            mode: MountMode::ReadWrite,
        }],
        network: NetworkPolicy::DenyAll,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = backend.prepare(&policy).await.expect("boot microVM w/ workspace");
    let endpoint = Firecracker::sidecar_endpoint(&handle).unwrap();
    let runtime = SidecarActorRuntime::new(endpoint);

    let log_dir = std::env::temp_dir().join("ship-fcws-live");
    std::fs::create_dir_all(&log_dir).ok();
    // Read the seeded file AND write a new one — both inside the microVM.
    let profile = ActorProfile {
        actor_id_hint: Some("fcws-actor".into()),
        kind: ActorKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec![
            "-c".into(),
            // No explicit `sync` — the guest's periodic sync + the
            // Isolator's shutdown flush-wait must carry the write back, the
            // way a real agent (which won't sync) relies on.
            format!(
                "cat {seed} && echo AGENT_WAS_HERE > {out}",
                seed = work.join("seed.txt").display(),
                out = out.display()
            ),
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

    let actor = runtime.spawn(profile).await.expect("spawn actor in microVM");
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while std::time::Instant::now() < deadline {
        if actor.child.try_exit().is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    // shutdown flushes the guest + copies the workspace back (debugfs rdump).
    backend.shutdown(handle).await.expect("shutdown + copy back");

    let copied = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&work);
    assert_eq!(
        copied.trim(),
        "AGENT_WAS_HERE",
        "a file the agent wrote inside the microVM must copy back to the host workspace"
    );
}

/// App-plane bridge: a no-network microVM agent reaches a host "daemon"
/// over vsock. The agent `wget`s `http://127.0.0.1:8088/ping` (the in-VM
/// proxy port), which the sidecar forwards over vsock → the Isolator's host
/// uds → a mock daemon; the response is written into the workspace and
/// copied back out. Proves an isolated microVM agent can still call MCP /
/// hooks / log_progress despite having no network device.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn microvm_agent_reaches_daemon_over_app_plane() {
    if !kvm_accessible() {
        eprintln!("firecracker_live: skipping — /dev/kvm not accessible");
        return;
    }
    let Some(mut cfg) = assets() else {
        eprintln!("firecracker_live: skipping — dev assets missing");
        return;
    };

    // Mock daemon: answer any request with `PONG`.
    let mock = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_addr = mock.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        loop {
            let Ok((mut s, _)) = mock.accept().await else { break };
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf).await;
                let _ = s
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nPONG",
                    )
                    .await;
            });
        }
    });
    cfg.app_plane_daemon = Some(mock_addr);

    let backend = Firecracker::new(cfg);
    if backend.available().await.is_err() {
        return;
    }

    let work = std::env::temp_dir().join(format!("ship-fcap-{}", std::process::id()));
    std::fs::create_dir_all(&work).unwrap();
    let resp = work.join("resp.txt");
    let _ = std::fs::remove_file(&resp);

    let policy = SandboxPolicy {
        label: "fc-appplane".into(),
        mounts: vec![FsMount {
            host_path: work.clone(),
            sandbox_path: work.clone(),
            mode: MountMode::ReadWrite,
        }],
        network: NetworkPolicy::DenyAll,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = backend.prepare(&policy).await.expect("boot microVM w/ app-plane");
    let endpoint = Firecracker::sidecar_endpoint(&handle).unwrap();
    let runtime = SidecarActorRuntime::new(endpoint);

    let log_dir = std::env::temp_dir().join("ship-fcap-live");
    std::fs::create_dir_all(&log_dir).ok();
    // Agent has no network, but the app-plane proxy (127.0.0.1:8088) reaches
    // the host daemon over vsock. Write the response into the workspace.
    let profile = ActorProfile {
        actor_id_hint: Some("fcap-actor".into()),
        kind: ActorKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec![
            "-c".into(),
            format!(
                "wget -q -O {out} http://127.0.0.1:8088/ping",
                out = resp.display()
            ),
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

    let actor = runtime.spawn(profile).await.expect("spawn actor in microVM");
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while std::time::Instant::now() < deadline {
        if actor.child.try_exit().is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    backend.shutdown(handle).await.expect("shutdown + copy back");

    let got = std::fs::read_to_string(&resp).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&work);
    assert_eq!(
        got.trim(),
        "PONG",
        "the in-VM agent must reach the host daemon over the vsock app-plane"
    );
}
