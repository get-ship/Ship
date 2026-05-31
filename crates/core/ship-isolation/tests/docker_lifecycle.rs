//! Lifecycle integration: SandboxOwner::Drop fires Docker shutdown.
//!
//! Auto-skips when docker/podman isn't reachable. Covers the contract
//! that ActorHandle ownership of the sandbox handle Just Works — the
//! supervisor needs nothing more than `handle.with_sandbox(owner)` to
//! get clean teardown on agent exit.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use ship_isolation::backends::{Docker, DockerConfig};
use ship_substrate::isolation::{Isolator, SandboxOwner, SandboxPolicy};

fn engine_available(bin: &str) -> bool {
    Command::new(bin)
        .args(["version", "--format", "{{.Server.Version}}"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn count_containers(bin: &str, name_prefix: &str) -> usize {
    let out = Command::new(bin)
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output();
    let Ok(out) = out else { return 0 };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|n| n.starts_with(name_prefix))
        .count()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropping_owner_tears_down_container() {
    let engine = if engine_available("docker") {
        "docker"
    } else if engine_available("podman") {
        "podman"
    } else {
        eprintln!("docker_lifecycle: skipping — no docker/podman");
        return;
    };

    let image =
        std::env::var("SHIP_DOCKER_IMAGE").unwrap_or_else(|_| "alpine:3".into());
    let prefix = format!("ship-life-{}-", std::process::id());

    let cfg = DockerConfig {
        engine_bin: engine.into(),
        image,
        name_prefix: prefix.clone(),
        ..DockerConfig::default()
    };

    let isolator: Arc<dyn Isolator> = Arc::new(Docker::new(cfg));
    if isolator.available().await.is_err() {
        eprintln!("docker_lifecycle: skipping — engine unavailable");
        return;
    }

    let workspace = std::env::temp_dir().join(format!("ship-life-work-{}", std::process::id()));
    std::fs::create_dir_all(&workspace).unwrap();
    let policy = SandboxPolicy {
        label: "lifecycle".into(),
        mounts: vec![ship_substrate::isolation::FsMount {
            host_path: workspace.clone(),
            sandbox_path: PathBuf::from("/work"),
            mode: ship_substrate::isolation::MountMode::ReadWrite,
        }],
        ..SandboxPolicy::default()
    };

    let handle = match isolator.prepare(&policy).await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("docker_lifecycle: skipping — prepare failed: {e}");
            let _ = std::fs::remove_dir_all(&workspace);
            return;
        }
    };

    let before = count_containers(engine, &prefix);
    assert_eq!(before, 1, "expected one container after prepare");

    // Build the owner and drop it. Drop spawns shutdown on the tokio
    // runtime; we give it a beat to complete before counting.
    {
        let _owner = Arc::new(SandboxOwner::new(isolator.clone(), handle));
        // owner drops at end of scope
    }
    // Wait for the spawned shutdown task.
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

    let after = count_containers(engine, &prefix);
    assert_eq!(
        after, 0,
        "container should be gone after SandboxOwner::Drop; engine={engine}"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

