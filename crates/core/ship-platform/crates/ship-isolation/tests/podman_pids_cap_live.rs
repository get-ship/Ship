//! Live podman test — T-RES: a cgroup resource cap actually CAPS, not just
//! gets passed as a flag.
//!
//! `docker::tests::*` already asserts `--pids-limit N` is *emitted*. This
//! asserts the flag *reaches the kernel cgroup and is enforced*. Inside the
//! sandbox the probe:
//!   1. reads `pids.max` and confirms it equals the configured limit — proves
//!      the runtime translated our flag into the container's cgroup;
//!   2. tries to spawn far more processes than the limit, then reads
//!      `pids.current` and confirms it never exceeded the cap — proves the
//!      kernel refused the excess forks.
//! Both reads use shell builtins (`read … < file`), so they work even while
//! we're pinned at the pids cap and `fork()` for an external `cat` would fail.
//!
//! pids is the deterministic controller to test: exceeding it fails forks
//! immediately, with no dependence on swap, overcommit, or the OOM killer
//! (which is why memory/cpu caps aren't asserted here).
//!
//! The burst runs in a subshell so that *it* hits the fork wall (a shell
//! exits with status 2 the instant its own `fork()` is denied); the parent
//! takes a single fork while the cgroup is empty, survives, and reads the
//! result with a builtin.
//!
//! In-sandbox exit-code contract:
//!   0  — cap applied AND enforced (PASS)
//!   10 — `pids.max` present but != configured limit (cap NOT applied)
//!   11 — `pids.current` exceeded the limit (cap NOT enforced)
//!   12 — burst never approached the cap (inconclusive; host treats as SKIP)
//!   20 — no pids controller in this cgroup (host treats as SKIP)
//!
//! Auto-skips when podman or the sidecar binary are absent.

#![allow(clippy::field_reassign_with_default)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use ship_isolation::backends::{Docker, DockerConfig};
use ship_isolation::runtime::SidecarRuntime;
use ship_isolation::{Isolator, NetworkPolicy, SandboxPolicy};
use isolation_port::ResourceLimits;
use process_supervisor::{ProcessKind, ProcessProfile, ProcessRuntime, TransportKind};

/// Comfortably above the sidecar's tokio thread count (the pids controller
/// counts tasks, not just processes) yet far below the burst below.
const PIDS_LIMIT: u32 = 128;

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
async fn pids_limit_actually_caps_process_count() {
    if !podman_available() {
        eprintln!("podman_pids_cap_live: skipping — podman not reachable");
        return;
    }
    let sidecar = sidecar_binary();
    if !sidecar.exists() {
        eprintln!("podman_pids_cap_live: skipping — ship-sidecar not built");
        return;
    }

    let cfg = DockerConfig {
        engine_bin: "podman".into(),
        image: "ubuntu:24.04".into(),
        name_prefix: "ship-podman-pidscap".into(),
        sidecar_bin: sidecar.clone(),
        ..DockerConfig::default()
    };
    let backend = Docker::new(cfg);
    if let Err(e) = backend.available().await {
        eprintln!("podman_pids_cap_live: skipping — engine unavailable: {e}");
        return;
    }

    // Only the pids cap is set — no network needed, so deny it.
    let mut limits = ResourceLimits::default();
    limits.pids_max = Some(PIDS_LIMIT);
    let policy = SandboxPolicy {
        label: "pids-cap".into(),
        mounts: vec![],
        network: NetworkPolicy::DenyAll,
        env: HashMap::new(),
        limits,
    };

    let handle = backend.prepare(&policy).await.expect("prepare");
    let endpoint = Docker::sidecar_endpoint(&handle).expect("sidecar endpoint");
    let runtime = SidecarRuntime::new(endpoint);

    // Read the cgroup cap (builtin read — fork-free, survives the pids cap),
    // confirm it equals what we configured, then try to blow past it and
    // confirm the controller held the line. cgroup v2 path first, v1 fallback.
    let probe = format!(
        r#"
maxf=/sys/fs/cgroup/pids.max
curf=/sys/fs/cgroup/pids.current
if [ ! -f "$maxf" ]; then maxf=/sys/fs/cgroup/pids/pids.max; curf=/sys/fs/cgroup/pids/pids.current; fi
if [ ! -f "$maxf" ]; then exit 20; fi
read m < "$maxf"
if [ "$m" != "{lim}" ]; then exit 10; fi
( i=0; while [ $i -lt {burst} ]; do sleep 30 & i=$((i+1)); done ) 2>/dev/null
read c < "$curf"
if [ "$c" -gt "{lim}" ]; then exit 11; fi
if [ "$c" -le "{half}" ]; then exit 12; fi
exit 0
"#,
        lim = PIDS_LIMIT,
        burst = PIDS_LIMIT * 3,
        half = PIDS_LIMIT / 2,
    );

    let log_dir = std::env::temp_dir().join("ship-podman-pidscap");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("pidscap-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec!["-c".into(), probe],
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
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
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
        Some(0) => { /* pids.max == limit AND pids.current never exceeded it */ }
        Some(20) => {
            eprintln!("podman_pids_cap_live: skipping — no pids controller in this cgroup")
        }
        Some(12) => {
            eprintln!("podman_pids_cap_live: skipping — burst never approached the cap")
        }
        Some(10) => panic!(
            "pids cap NOT applied — pids.max != {PIDS_LIMIT}; --pids-limit did not reach the cgroup"
        ),
        Some(11) => {
            panic!("pids cap NOT enforced — process count exceeded {PIDS_LIMIT}")
        }
        Some(c) => panic!("probe exited {c} (unexpected)"),
        None => panic!("actor never exited within 20s"),
    }
}
