//! End-to-end test: SidecarSandbox with `NetworkPolicy::Allowlist` injects
//! HTTPS_PROXY into the container, and an in-container `curl` to a
//! denied host gets rejected by the host-side proxy.
//!
//! Skipped under the same gates as youki_sidecar_live.

#![cfg(all(
    feature = "youki-backend",
    feature = "sidecar-mode",
    target_os = "linux"
))]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use ship_isolation::backends::{SidecarConfig, SidecarSandbox, Youki, YoukiConfig};
use ship_isolation::net::ProxyDecision;
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
    eprintln!("network_live: skipping — {reason}");
}

#[tokio::test]
async fn allowlist_injects_proxy_env_and_denies_unlisted_host() {
    let youki_bin = env_or("SHIP_YOUKI_BIN", "/tmp/youki");
    let spec = env_or("SHIP_YOUKI_SPEC", "/tmp/claude-bundle/config.json");
    let rootfs = env_or("SHIP_AGENT_ROOTFS", "/tmp/ship-rootfs/rootfs");
    let sidecar = match std::env::var("SHIP_SIDECAR_BIN") {
        Ok(p) => PathBuf::from(p),
        Err(_) => workspace_default_sidecar(),
    };

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
    if !rootfs.join("usr/bin/curl").is_file() {
        return skip("rootfs missing /usr/bin/curl — rebuild rootfs script");
    }

    let state_root =
        std::env::temp_dir().join(format!("ship-net-state-{}", std::process::id()));
    tokio::fs::create_dir_all(&state_root).await.unwrap();
    let workspace =
        std::env::temp_dir().join(format!("ship-net-work-{}", std::process::id()));
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
        label: "net-test".into(),
        ..SandboxPolicy::default()
    };
    policy.network = NetworkPolicy::Allowlist {
        hosts: vec!["allowed.test".into()],
    };
    policy.mounts = vec![FsMount {
        host_path: workspace.clone(),
        sandbox_path: PathBuf::from("/work"),
        mode: MountMode::ReadWrite,
    }];

    sandbox.start(policy).await.expect("start");
    assert!(sandbox.proxy_port().is_some(), "proxy should be running");

    // 1) Inside the container, env should carry HTTPS_PROXY.
    let (code, stdout, _stderr) = sandbox
        .run_capture(
            "/bin/sh",
            vec!["-c".into(), "printenv HTTPS_PROXY".into()],
            HashMap::new(),
            "/work",
        )
        .await
        .expect("run env probe");
    assert_eq!(code, Some(0), "printenv should succeed");
    let stdout_str = String::from_utf8_lossy(&stdout).to_string();
    assert!(
        stdout_str.contains("http://"),
        "HTTPS_PROXY missing in container env: {stdout_str:?}"
    );

    // 2) A CONNECT to a denied host through the proxy must be refused.
    // curl --proxy <url> https://denied.test will see 403 from the proxy
    // and exit non-zero (exit 56 = recv error, or 22 with -f).
    let proxy_url = stdout_str.trim().to_string();
    let (code, _out, err) = sandbox
        .run_capture(
            "/bin/sh",
            vec![
                "-c".into(),
                format!(
                    "curl -sS --max-time 5 -o /dev/null -w '%{{http_code}}' \
                     --proxy {proxy_url} https://denied.test/"
                ),
            ],
            HashMap::new(),
            "/work",
        )
        .await
        .expect("run curl probe");
    eprintln!(
        "network_live: curl denied: code={code:?} stderr={:?}",
        String::from_utf8_lossy(&err)
    );
    assert!(
        code != Some(0),
        "curl should not succeed against a denied host"
    );

    // 3) Proxy log should record the deny decision.
    let log = sandbox.proxy_log_entries().await.expect("log");
    assert!(
        log.iter()
            .any(|(h, d)| h == "denied.test" && matches!(d, ProxyDecision::Deny(_))),
        "proxy log should have a deny for denied.test, got: {log:?}"
    );

    sandbox.shutdown().await.expect("shutdown");
    let _ = tokio::fs::remove_dir_all(&state_root).await;
    let _ = tokio::fs::remove_dir_all(&workspace).await;
}
