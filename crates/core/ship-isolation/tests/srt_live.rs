//! Live smoke test against a real `srt` install.
//!
//! Skipped unless all of:
//!   - `bun` on PATH
//!   - `srt` resolvable (env `SHIP_SRT_ENTRY` or default global install path)
//!   - on Linux: `bwrap` and `socat` on PATH
//!
//! Run live with `cargo test -p ship-isolation --test srt_live -- --nocapture`
//! after `bun add -g @anthropic-ai/sandbox-runtime` and (Linux only)
//! `sudo apt install bubblewrap socat`.

use std::collections::HashMap;
use std::path::PathBuf;

use ship_isolation::backends::{SandboxRuntime, SandboxRuntimeConfig};
use ship_isolation::{Isolator, LaunchCommand, MountMode, NetworkPolicy, SandboxPolicy};

fn locate_bun() -> Option<PathBuf> {
    which_simple("bun")
}

fn locate_srt() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("SHIP_SRT_ENTRY") {
        let pb = PathBuf::from(p);
        return pb.is_file().then_some(pb);
    }
    let candidates = [
        dirs_home().map(|h| h.join(".bun/install/global/node_modules/@anthropic-ai/sandbox-runtime/dist/cli.js")),
        dirs_home().map(|h| h.join(".npm-global/lib/node_modules/@anthropic-ai/sandbox-runtime/dist/cli.js")),
        Some(PathBuf::from("/usr/local/lib/node_modules/@anthropic-ai/sandbox-runtime/dist/cli.js")),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|p| p.is_file())
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn which_simple(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(name))
            .find(|p| p.is_file())
    })
}

fn skip(reason: &str) {
    eprintln!("srt_live: skipping — {reason}");
}

fn workspace_policy(dir: &std::path::Path) -> SandboxPolicy {
    SandboxPolicy::workspace_default(dir.to_path_buf())
}

#[tokio::test]
async fn srt_blocks_write_outside_allow_list() {
    let Some(bun) = locate_bun() else {
        return skip("bun not on PATH");
    };
    let Some(srt) = locate_srt() else {
        return skip("srt entry not found — set SHIP_SRT_ENTRY or `bun add -g`");
    };
    #[cfg(target_os = "linux")]
    {
        if which_simple("bwrap").is_none() || which_simple("socat").is_none() {
            return skip("bwrap or socat not installed");
        }
    }

    let work = std::env::temp_dir().join(format!("ship-srt-test-{}", std::process::id()));
    tokio::fs::create_dir_all(&work).await.unwrap();
    let outside = std::env::temp_dir().join(format!("ship-srt-outside-{}.txt", std::process::id()));
    let _ = tokio::fs::remove_file(&outside).await;

    let mut policy = workspace_policy(&work);
    policy.network = NetworkPolicy::DenyAll;
    policy.mounts = vec![ship_isolation::FsMount {
        host_path: work.clone(),
        sandbox_path: work.clone(),
        mode: MountMode::ReadWrite,
    }];

    let sb = SandboxRuntime::new(SandboxRuntimeConfig {
        bun_bin: bun,
        srt_entry: srt,
    });
    sb.available().await.expect("srt should be available");
    let handle = sb.prepare(&policy).await.unwrap();

    let cmd = LaunchCommand {
        bin: PathBuf::from("sh"),
        args: vec!["-c".into(), format!("echo escape > {}", outside.display())],
        env: HashMap::new(),
        cwd: work.clone(),
    };
    let exec = sb.exec(&handle, cmd).await.expect("spawn failed");
    let pid = exec.pid.expect("pid");
    eprintln!("srt_live: spawned pid {pid}, waiting...");

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let escaped = tokio::fs::try_exists(&outside).await.unwrap_or(false);
    sb.shutdown(handle).await.unwrap();

    assert!(
        !escaped,
        "srt should have blocked the write to {}",
        outside.display()
    );
}

#[tokio::test]
async fn srt_allows_write_inside_workspace() {
    let Some(bun) = locate_bun() else {
        return skip("bun not on PATH");
    };
    let Some(srt) = locate_srt() else {
        return skip("srt entry not found");
    };
    #[cfg(target_os = "linux")]
    {
        if which_simple("bwrap").is_none() || which_simple("socat").is_none() {
            return skip("bwrap or socat not installed");
        }
    }

    let work = std::env::temp_dir().join(format!("ship-srt-ok-{}", std::process::id()));
    tokio::fs::create_dir_all(&work).await.unwrap();
    let inside = work.join("inside.txt");
    let _ = tokio::fs::remove_file(&inside).await;

    let policy = workspace_policy(&work);

    let sb = SandboxRuntime::new(SandboxRuntimeConfig {
        bun_bin: bun,
        srt_entry: srt,
    });
    sb.available().await.unwrap();
    let handle = sb.prepare(&policy).await.unwrap();

    let cmd = LaunchCommand {
        bin: PathBuf::from("sh"),
        args: vec!["-c".into(), format!("echo ok > {}", inside.display())],
        env: HashMap::new(),
        cwd: work.clone(),
    };
    sb.exec(&handle, cmd).await.expect("spawn failed");

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let body = tokio::fs::read_to_string(&inside).await.ok();
    sb.shutdown(handle).await.unwrap();

    assert_eq!(body.as_deref().map(str::trim), Some("ok"));
}
