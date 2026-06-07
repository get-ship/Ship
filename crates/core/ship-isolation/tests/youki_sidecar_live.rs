//! Live test: drive a long-lived youki container via the in-container
//! `ship-sidecar`. Container PID 1 = sidecar; agent execs land via HTTP.
//!
//! Skipped unless all of:
//!   - youki at SHIP_YOUKI_BIN (or /tmp/youki)
//!   - spec template at SHIP_YOUKI_SPEC (or /tmp/claude-bundle/config.json)
//!   - rootfs at SHIP_AGENT_ROOTFS (or /tmp/ship-rootfs/rootfs)
//!   - sidecar at SHIP_SIDECAR_BIN
//!     (default: <workspace>/target/debug/ship-sidecar)

#![cfg(all(
    feature = "youki-backend",
    feature = "sidecar-mode",
    target_os = "linux"
))]

use std::path::PathBuf;
use std::sync::Arc;

use ship_isolation::backends::{SidecarConfig, SidecarSandbox, Youki, YoukiConfig};
use ship_isolation::{FsMount, MountMode, NetworkPolicy, SandboxPolicy};

fn env_or(name: &str, default: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap_or_else(|_| default.into()))
}

fn workspace_default_sidecar() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../../target/debug/ship-sidecar");
    p
}

fn skip(reason: &str) {
    eprintln!("youki_sidecar_live: skipping — {reason}");
}

#[tokio::test]
async fn sidecar_runs_exec_and_returns_exit_via_http() {
    let youki_bin = env_or("SHIP_YOUKI_BIN", "/tmp/youki");
    let spec = env_or("SHIP_YOUKI_SPEC", "/tmp/claude-bundle/config.json");
    let rootfs = env_or("SHIP_AGENT_ROOTFS", "/tmp/ship-rootfs/rootfs");
    let sidecar = match std::env::var("SHIP_SIDECAR_BIN") {
        Ok(p) => PathBuf::from(p),
        Err(_) => workspace_default_sidecar(),
    };

    eprintln!("youki_sidecar_live: youki_bin={}", youki_bin.display());
    eprintln!("youki_sidecar_live: spec={}", spec.display());
    eprintln!("youki_sidecar_live: rootfs={}", rootfs.display());
    eprintln!("youki_sidecar_live: sidecar={}", sidecar.display());

    for (p, what) in [
        (&youki_bin, "youki binary"),
        (&spec, "spec template"),
        (&sidecar, "ship-sidecar binary"),
    ] {
        if !p.is_file() {
            return skip(&format!("{what} not at {}", p.display()));
        }
    }
    if !rootfs.is_dir() {
        return skip(&format!("rootfs not at {}", rootfs.display()));
    }

    let state_root =
        std::env::temp_dir().join(format!("ship-yks-state-{}", std::process::id()));
    tokio::fs::create_dir_all(&state_root).await.unwrap();
    let workspace =
        std::env::temp_dir().join(format!("ship-yks-work-{}", std::process::id()));
    tokio::fs::create_dir_all(&workspace).await.unwrap();

    let youki_cfg = YoukiConfig {
        youki_bin,
        spec_template: spec,
        rootfs,
        state_root: state_root.clone(),
    };
    let side_cfg = SidecarConfig {
        host_sidecar_bin: sidecar,
        ..SidecarConfig::default()
    };

    let youki = Arc::new(Youki::new(youki_cfg.clone()));
    let mut sandbox = SidecarSandbox::new(youki, youki_cfg, side_cfg);

    let mut policy = SandboxPolicy {
        label: "sidecar-test".into(),
        ..SandboxPolicy::default()
    };
    policy.network = NetworkPolicy::Open;
    policy.mounts = vec![FsMount {
        host_path: workspace.clone(),
        sandbox_path: PathBuf::from("/work"),
        mode: MountMode::ReadWrite,
    }];

    eprintln!("youki_sidecar_live: about to call start()");
    sandbox.start(policy).await.expect("start sidecar sandbox");
    eprintln!("youki_sidecar_live: start() returned, base_url={}", sandbox.base_url());

    eprintln!("youki_sidecar_live: about to call exec()");
    let resp = sandbox
        .exec(
            "/bin/sh",
            vec!["-c".into(), "echo hello-from-sidecar > /work/marker.txt".into()],
            std::collections::HashMap::new(),
            "/work",
        )
        .await
        .expect("exec");
    eprintln!("youki_sidecar_live: exec returned, exec_id={}", resp.exec_id);
    let code = sandbox
        .wait_for_exit(&resp.exec_id)
        .await
        .expect("wait");
    eprintln!("youki_sidecar_live: wait returned, code={code:?}");
    assert_eq!(code, Some(0), "exec should exit 0");

    let body = tokio::fs::read_to_string(workspace.join("marker.txt"))
        .await
        .unwrap_or_default();
    eprintln!("youki_sidecar_live: marker body={body:?}");
    assert_eq!(body.trim(), "hello-from-sidecar");

    sandbox.shutdown().await.expect("shutdown");
    let _ = tokio::fs::remove_dir_all(&state_root).await;
    let _ = tokio::fs::remove_dir_all(&workspace).await;
    eprintln!("youki_sidecar_live: SUCCESS");
}
