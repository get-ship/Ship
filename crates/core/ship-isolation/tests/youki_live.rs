//! Live smoke test for the `Youki` backend. Skips when any of:
//!   - `youki` binary not at `SHIP_YOUKI_BIN` (or `/tmp/youki`)
//!   - spec template not at `SHIP_YOUKI_SPEC` (or `/tmp/youki-bundle/config.json`)
//!   - rootfs not at `SHIP_YOUKI_ROOTFS` (or `/tmp/youki-bundle/rootfs`)
//!
//! Sets up its own bundle and state dirs under temp; doesn't touch the
//! probe bundle.

#![cfg(all(feature = "youki-backend", target_os = "linux"))]

use std::collections::HashMap;
use std::path::PathBuf;

use ship_isolation::backends::{Youki, youki::YoukiConfig};
use ship_isolation::{Isolator, LaunchCommand, SandboxPolicy};

fn first_existing(env: &str, defaults: &[&str], file: bool) -> Option<PathBuf> {
    if let Ok(p) = std::env::var(env) {
        let pb = PathBuf::from(p);
        let ok = if file { pb.is_file() } else { pb.is_dir() };
        return ok.then_some(pb);
    }
    for d in defaults {
        let pb = PathBuf::from(d);
        let ok = if file { pb.is_file() } else { pb.is_dir() };
        if ok {
            return Some(pb);
        }
    }
    None
}

fn skip(reason: &str) {
    eprintln!("youki_live: skipping — {reason}");
}

#[tokio::test]
async fn youki_runs_echo_in_alpine() {
    let Some(youki_bin) = first_existing("SHIP_YOUKI_BIN", &["/tmp/youki"], true) else {
        return skip("youki binary not found");
    };
    let Some(spec_template) =
        first_existing("SHIP_YOUKI_SPEC", &["/tmp/youki-bundle/config.json"], true)
    else {
        return skip("spec template not found");
    };
    let Some(rootfs) =
        first_existing("SHIP_YOUKI_ROOTFS", &["/tmp/youki-bundle/rootfs"], false)
    else {
        return skip("rootfs dir not found");
    };

    let state_root = std::env::temp_dir().join(format!("ship-youki-state-{}", std::process::id()));
    tokio::fs::create_dir_all(&state_root).await.unwrap();

    let backend = Youki::new(YoukiConfig {
        youki_bin,
        spec_template,
        rootfs,
        state_root: state_root.clone(),
    });
    backend.available().await.expect("youki should be available");

    let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp"));
    let handle = backend.prepare(&pol).await.unwrap();

    let marker = std::env::temp_dir().join(format!("ship-youki-marker-{}", std::process::id()));
    let _ = tokio::fs::remove_file(&marker).await;

    let cmd = LaunchCommand {
        bin: PathBuf::from("/bin/sh"),
        args: vec!["-c".into(), format!("echo ok > {}", marker.display())],
        env: HashMap::new(),
        cwd: PathBuf::from("/"),
    };
    let exec = backend.exec(&handle, cmd).await.expect("spawn");
    eprintln!("youki_live: spawned pid {:?}", exec.pid);

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let body = tokio::fs::read_to_string(&marker).await.ok();
    backend.shutdown(handle).await.unwrap();
    let _ = tokio::fs::remove_dir_all(&state_root).await;

    assert_eq!(
        body.as_deref().map(str::trim),
        Some("ok"),
        "expected the container's bind-mounted /tmp write to land on the host"
    );
}
