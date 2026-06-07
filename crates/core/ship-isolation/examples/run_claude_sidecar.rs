//! Run Claude Code inside a Ship-managed container with the *full* Tier-A
//! stack: sidecar-managed long-lived container, hostname-allowlisted
//! network egress, per-sandbox ephemeral credentials copy.
//!
//! Compared to `examples/run_claude.rs`, this:
//!   - Uses `SidecarSandbox` (long-lived container, exec via HTTP)
//!     instead of `Youki::run_to_completion` (one-shot `youki run`).
//!   - Sets `NetworkPolicy::Allowlist { hosts: ["api.anthropic.com",
//!     ...] }` so the only host the agent can reach is the API.
//!   - Mounts credentials as a Ship-owned ephemeral file (option B)
//!     rather than the host's real `~/.claude/.credentials.json`.
//!
//! Usage:
//!   cargo run -p ship-isolation \
//!       --features "youki-backend sidecar-mode" \
//!       --example run_claude_sidecar -- <workspace> "<prompt>"

#![cfg(all(
    feature = "youki-backend",
    feature = "sidecar-mode",
    target_os = "linux"
))]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ship_isolation::backends::{SidecarConfig, SidecarSandbox, Youki, YoukiConfig};
use ship_isolation::{
    EphemeralFile, FsMount, MountMode, NetworkPolicy, SandboxPolicy,
};

fn env_path(name: &str, default: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap_or_else(|_| default.into()))
}

fn must_exist(path: &Path, kind: &str) -> Result<(), String> {
    let ok = match kind {
        "file" => path.is_file(),
        "dir" => path.is_dir(),
        _ => path.exists(),
    };
    if ok {
        Ok(())
    } else {
        Err(format!("{kind} not found: {}", path.display()))
    }
}

fn workspace_default_sidecar() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../../target/debug/ship-sidecar");
    p
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let workspace = PathBuf::from(args.next().ok_or("usage: <workspace> <prompt>")?);
    let prompt = args.next().unwrap_or_else(|| "Say hello in one word.".into());

    let youki_bin = env_path("SHIP_YOUKI_BIN", "/tmp/youki");
    let spec_template = env_path("SHIP_YOUKI_SPEC", "/tmp/claude-bundle/config.json");
    let rootfs = env_path("SHIP_AGENT_ROOTFS", "/tmp/ship-rootfs/rootfs");
    let creds = env_path(
        "SHIP_CLAUDE_CREDENTIALS",
        &format!(
            "{}/.claude/.credentials.json",
            std::env::var("HOME").unwrap_or_default()
        ),
    );
    let sidecar = match std::env::var("SHIP_SIDECAR_BIN") {
        Ok(p) => PathBuf::from(p),
        Err(_) => workspace_default_sidecar(),
    };

    must_exist(&youki_bin, "file")?;
    must_exist(&spec_template, "file")?;
    must_exist(&rootfs, "dir")?;
    must_exist(&workspace, "dir")?;
    must_exist(&creds, "file")?;
    must_exist(&sidecar, "file")?;

    let state_root = std::env::temp_dir().join(format!("ship-rcs-{}", std::process::id()));
    tokio::fs::create_dir_all(&state_root).await?;

    let ephem = EphemeralFile::copy_from(&creds).await?;

    let youki_cfg = YoukiConfig {
        youki_bin,
        spec_template,
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
        label: "run-claude-sidecar".into(),
        ..SandboxPolicy::default()
    };
    policy.network = NetworkPolicy::Allowlist {
        hosts: vec![
            "api.anthropic.com".into(),
            "console.anthropic.com".into(),
            "statsig.anthropic.com".into(),
        ],
    };
    policy.mounts = vec![
        FsMount {
            host_path: workspace.clone(),
            sandbox_path: PathBuf::from("/work"),
            mode: MountMode::ReadWrite,
        },
        ephem.as_mount(PathBuf::from("/root/.claude/.credentials.json")),
        FsMount {
            host_path: PathBuf::from("/etc/resolv.conf"),
            sandbox_path: PathBuf::from("/etc/resolv.conf"),
            mode: MountMode::ReadOnly,
        },
    ];

    eprintln!("ship-isolation: starting sandbox...");
    sandbox.start(policy).await?;
    eprintln!(
        "ship-isolation: sandbox up, sidecar={} proxy={:?}",
        sandbox.base_url(),
        sandbox.proxy_port()
    );

    let mut env = HashMap::new();
    env.insert("HOME".into(), "/root".into());
    env.insert(
        "PATH".into(),
        "/opt/bun/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into(),
    );
    env.insert("TERM".into(), "dumb".into());

    let (code, stdout, stderr) = sandbox
        .run_capture(
            "/opt/bun/bin/claude",
            vec!["--print".into(), prompt],
            env,
            "/work",
        )
        .await?;

    print!("{}", String::from_utf8_lossy(&stdout));
    let stderr_s = String::from_utf8_lossy(&stderr);
    if !stderr_s.is_empty() {
        eprintln!("--- stderr ---\n{stderr_s}");
    }

    if let Some(log) = sandbox.proxy_log_entries().await {
        eprintln!("--- proxy decisions ---");
        for (host, decision) in log {
            eprintln!("  {host}: {decision:?}");
        }
    }

    sandbox.shutdown().await?;
    drop(ephem);
    let _ = tokio::fs::remove_dir_all(&state_root).await;

    std::process::exit(code.unwrap_or(0));
}
