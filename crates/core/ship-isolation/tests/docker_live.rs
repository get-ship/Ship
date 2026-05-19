//! Live Docker (or Podman) integration test.
//!
//! Auto-skips when no engine is reachable. Pulls a tiny alpine image
//! to keep the test fast; if you don't want network access at test
//! time, set `SHIP_DOCKER_IMAGE` to an already-cached image.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use ship_isolation::backends::{Docker, DockerConfig};
use ship_isolation::{FsMount, Isolator, LaunchCommand, MountMode, NetworkPolicy, SandboxPolicy};

fn engine_available(bin: &str) -> bool {
    Command::new(bin)
        .args(["version", "--format", "{{.Server.Version}}"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn docker_prepare_exec_and_shutdown_against_real_engine() {
    let bin = if engine_available("docker") {
        "docker"
    } else if engine_available("podman") {
        "podman"
    } else {
        eprintln!("docker_live: skipping — no docker/podman engine reachable");
        return;
    };

    let image = std::env::var("SHIP_DOCKER_IMAGE").unwrap_or_else(|_| "alpine:3".into());

    let workspace = std::env::temp_dir().join(format!("ship-docker-live-{}", std::process::id()));
    std::fs::create_dir_all(&workspace).unwrap();

    let cfg = DockerConfig {
        engine_bin: bin.into(),
        image: image.clone(),
        name_prefix: "ship-docker-live".into(),
        ..DockerConfig::default()
    };
    let backend = Docker::new(cfg);

    if let Err(e) = backend.available().await {
        eprintln!("docker_live: skipping — engine reported unavailable: {e}");
        return;
    }

    let policy = SandboxPolicy {
        label: "live".into(),
        mounts: vec![FsMount {
            host_path: workspace.clone(),
            sandbox_path: PathBuf::from("/work"),
            mode: MountMode::ReadWrite,
        }],
        network: NetworkPolicy::DenyAll,
        env: HashMap::new(),
        limits: Default::default(),
    };

    let handle = match backend.prepare(&policy).await {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "docker_live: skipping — prepare failed (image '{image}' missing?): {e}"
            );
            let _ = std::fs::remove_dir_all(&workspace);
            return;
        }
    };
    assert_eq!(handle.backend, "docker");

    let cmd = LaunchCommand {
        bin: PathBuf::from("/bin/sh"),
        args: vec!["-c".into(), "echo hello-from-sandbox > /work/out".into()],
        env: HashMap::new(),
        cwd: PathBuf::from("/work"),
    };

    let exec = backend.exec(&handle, cmd).await.expect("exec");
    assert!(exec.pid.is_some());

    // Give docker exec a beat to finish (it's a millisecond op).
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let written = std::fs::read_to_string(workspace.join("out")).unwrap_or_default();
    assert_eq!(written.trim(), "hello-from-sandbox");

    backend.shutdown(handle).await.expect("shutdown");

    // shutdown is idempotent — calling on a new handle (different sandbox)
    // wouldn't apply, but a double shutdown of the same one is a no-op.
    let _ = std::fs::remove_dir_all(&workspace);
}
