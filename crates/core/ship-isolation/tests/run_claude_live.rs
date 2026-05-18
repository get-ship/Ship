//! Live end-to-end test: drive Claude Code inside a Ship-managed container
//! using the user's real OAuth credentials.
//!
//! Skipped unless all of:
//!   - `youki` at SHIP_YOUKI_BIN (or /tmp/youki)
//!   - spec template at SHIP_YOUKI_SPEC (or /tmp/claude-bundle/config.json)
//!   - rootfs at SHIP_AGENT_ROOTFS (or /tmp/ship-rootfs/rootfs)
//!     with `/opt/bun/bin/claude` inside
//!   - credentials at SHIP_CLAUDE_CREDENTIALS (or ~/.claude/.credentials.json)
//!   - env `SHIP_RUN_CLAUDE_LIVE=1` (explicit opt-in — this call hits the
//!     Anthropic API and costs money)

#![cfg(all(feature = "youki-backend", target_os = "linux"))]

use std::collections::HashMap;
use std::path::PathBuf;

use ship_isolation::backends::{Youki, YoukiConfig};
use ship_isolation::{
    FsMount, Isolator, LaunchCommand, MountMode, NetworkPolicy, SandboxPolicy,
};

fn env_or(name: &str, default: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap_or_else(|_| default.into()))
}

fn skip(reason: &str) {
    eprintln!("run_claude_live: skipping — {reason}");
}

#[tokio::test]
async fn claude_responds_inside_container_with_real_oauth() {
    if std::env::var("SHIP_RUN_CLAUDE_LIVE").as_deref() != Ok("1") {
        return skip("set SHIP_RUN_CLAUDE_LIVE=1 to enable (costs API tokens)");
    }
    let youki_bin = env_or("SHIP_YOUKI_BIN", "/tmp/youki");
    let spec = env_or("SHIP_YOUKI_SPEC", "/tmp/claude-bundle/config.json");
    let rootfs = env_or("SHIP_AGENT_ROOTFS", "/tmp/ship-rootfs/rootfs");
    let creds = env_or(
        "SHIP_CLAUDE_CREDENTIALS",
        &format!(
            "{}/.claude/.credentials.json",
            std::env::var("HOME").unwrap_or_default()
        ),
    );
    for (p, what) in [
        (&youki_bin, "youki binary"),
        (&spec, "spec template"),
        (&creds, "credentials"),
    ] {
        if !p.is_file() {
            return skip(&format!("{what} not at {}", p.display()));
        }
    }
    if !rootfs.is_dir() {
        return skip(&format!("rootfs not at {}", rootfs.display()));
    }
    if !rootfs.join("opt/bun/bin/claude").is_file() {
        return skip("rootfs missing /opt/bun/bin/claude — run scripts/build_agent_rootfs.sh");
    }

    let workspace =
        std::env::temp_dir().join(format!("ship-run-claude-test-{}", std::process::id()));
    tokio::fs::create_dir_all(&workspace).await.unwrap();
    tokio::fs::write(workspace.join("README.md"), "test workspace\n")
        .await
        .unwrap();

    let state_root =
        std::env::temp_dir().join(format!("ship-run-claude-state-{}", std::process::id()));
    tokio::fs::create_dir_all(&state_root).await.unwrap();

    let backend = Youki::new(YoukiConfig {
        youki_bin,
        spec_template: spec,
        rootfs,
        state_root: state_root.clone(),
    });
    backend.available().await.unwrap();

    let mut policy = SandboxPolicy {
        label: "live-run-claude".into(),
        ..SandboxPolicy::default()
    };
    policy.network = NetworkPolicy::Open;
    policy.mounts = vec![
        FsMount {
            host_path: workspace.clone(),
            sandbox_path: PathBuf::from("/work"),
            mode: MountMode::ReadWrite,
        },
        FsMount {
            host_path: creds.clone(),
            sandbox_path: PathBuf::from("/root/.claude/.credentials.json"),
            mode: MountMode::ReadOnly,
        },
        FsMount {
            host_path: PathBuf::from("/etc/resolv.conf"),
            sandbox_path: PathBuf::from("/etc/resolv.conf"),
            mode: MountMode::ReadOnly,
        },
    ];

    let handle = backend.prepare(&policy).await.unwrap();
    let marker = workspace.join("response.txt");

    let mut env = HashMap::new();
    env.insert("HOME".into(), "/root".into());
    env.insert(
        "PATH".into(),
        "/opt/bun/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
    );
    env.insert("TERM".into(), "dumb".into());

    let cmd = LaunchCommand {
        bin: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".into(),
            "/opt/bun/bin/claude --print 'Reply with exactly: SHIP_OK' > /work/response.txt".into(),
        ],
        env,
        cwd: PathBuf::from("/work"),
    };

    let code = backend.run_to_completion(&handle, cmd).await.unwrap();
    backend.shutdown(handle).await.unwrap();

    let _ = tokio::fs::remove_dir_all(&state_root).await;

    assert_eq!(code, Some(0), "claude should exit 0");
    let body = tokio::fs::read_to_string(&marker).await.unwrap_or_default();
    let _ = tokio::fs::remove_dir_all(&workspace).await;
    assert!(
        body.contains("SHIP_OK"),
        "expected response to contain SHIP_OK, got: {body:?}"
    );
}
