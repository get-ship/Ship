//! Live Apple Container test — DRAFT for Apple-Silicon verification.
//!
//! Constructs the AppleContainer Isolator directly (NOT via the recommender —
//! `has_impl(AppleContainer)` stays false until this is verified, so an
//! unverified backend never enters the production spawn path). Auto-skips
//! unless Apple's `container` CLI + the configured image are present.
//!
//! On an Apple-Silicon Mac:
//!   1. Build an arm64 agent-runner image with ship-sidecar baked in (adapt
//!      scripts/build-agent-image.sh — `container build` / load the image).
//!   2. SHIP_APPLE_IMAGE=<that image> cargo test -p ship-isolation \
//!        --test apple_container_live -- --nocapture
//!
//! Expect to adjust `backends/apple_container.rs::ip_from_inspect` to the real
//! `container inspect` JSON shape — that's the one unverified bit.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use ship_isolation::backends::{AppleContainer, AppleContainerConfig};
use ship_isolation::runtime::SidecarRuntime;
use ship_isolation::Isolator;
use isolation_port::SandboxPolicy;
use process_supervisor::{ProcessKind, ProcessProfile, ProcessRuntime, TransportKind};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actor_runs_inside_apple_container() {
    let image = std::env::var("SHIP_APPLE_IMAGE").unwrap_or_else(|_| "ship/agent-runner:dev".into());
    let backend = AppleContainer::new(AppleContainerConfig {
        container_bin: "container".into(),
        image,
        sidecar_path: "/usr/local/bin/ship-sidecar".into(),
    });
    if let Err(e) = backend.available().await {
        eprintln!("apple_container_live: skipping — {e}");
        return;
    }

    let policy = SandboxPolicy { label: "apple-live".into(), ..SandboxPolicy::default() };
    let handle = backend.prepare(&policy).await.expect("prepare (check ip_from_inspect if this fails)");
    let endpoint = AppleContainer::sidecar_endpoint(&handle).expect("sidecar endpoint");
    let runtime = SidecarRuntime::new(endpoint);

    let log_dir = std::env::temp_dir().join("ship-apple-live");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ProcessProfile {
        actor_id_hint: Some("apple-actor".into()),
        kind: ProcessKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/sh".into(),
        args: vec!["-c".into(), "true".into()],
        env: HashMap::new(),
        work_dir: PathBuf::from("/"),
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
    assert!(exit_code.is_some(), "actor never exited — sidecar unreachable in the microVM");
}
