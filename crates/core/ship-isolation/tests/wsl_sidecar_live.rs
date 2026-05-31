//! Live WSL test — drives the Windows substrate Isolator end to end.
//!
//! Runs ONLY on Windows: `shipd` (and therefore this test process) must be on
//! the Windows host, because WSL2's localhost forwarding is host↔distro, not
//! sibling-distro↔distro. From a sibling Linux distro the supervisor can't
//! reach the target distro's port (verified during the POC — the cross-distro
//! connect fails while `Test-NetConnection` from the Windows side returns
//! True). So on non-Windows this cell auto-skips.
//!
//! Prereqs on Windows: a registered Ship substrate distro (default
//! `ship-substrate`, override via `SHIP_WSL_TEST_DISTRO`) with `ship-sidecar`
//! at `/usr/bin/ship-sidecar`, and `wsl.exe` on PATH.
//!
//! The POC (launch via `wsl.exe` + Windows→distro localhost forward) was
//! verified manually on a WSL2 box; this is the automatable form for a
//! Windows runner.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use ship_isolation::backends::{Wsl, WslConfig};
use ship_isolation::runtime::SidecarActorRuntime;
use ship_isolation::Isolator;
use ship_substrate::isolation::SandboxPolicy;
use ship_substrate::supervisor::{ActorKind, ActorProfile, ActorRuntime, TransportKind};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actor_runs_inside_wsl_substrate() {
    if !cfg!(target_os = "windows") {
        eprintln!(
            "wsl_sidecar_live: skipping — runs on Windows (the shipd host). WSL2 localhost \
             forwarding is host<->distro; a sibling Linux distro can't reach the target distro."
        );
        return;
    }
    let distro = std::env::var("SHIP_WSL_TEST_DISTRO").unwrap_or_else(|_| "ship-substrate".into());
    let backend = Wsl::new(WslConfig {
        wsl_bin: PathBuf::from("wsl.exe"),
        distro,
        sidecar_path: "/usr/bin/ship-sidecar".into(),
    });
    if let Err(e) = backend.available().await {
        eprintln!("wsl_sidecar_live: skipping — substrate distro unavailable: {e}");
        return;
    }

    let policy = SandboxPolicy {
        label: "wsl-live".into(),
        ..SandboxPolicy::default()
    };
    let handle = backend.prepare(&policy).await.expect("prepare");
    let endpoint = Wsl::sidecar_endpoint(&handle).expect("sidecar endpoint");
    let runtime = SidecarActorRuntime::new(endpoint);

    let log_dir = std::env::temp_dir().join("ship-wsl-live");
    std::fs::create_dir_all(&log_dir).ok();
    let profile = ActorProfile {
        actor_id_hint: Some("wsl-actor".into()),
        kind: ActorKind::PipelineStep,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/busybox".into(),
        args: vec!["true".into()],
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
        "actor never exited — sidecar unreachable in the WSL distro"
    );
}
