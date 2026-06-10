//! Live Tier-1 confinement test — does the SrtWrapper sandbox actually
//! HOLD, via the production path?
//!
//! This is the test the deleted `srt_live` only *looked* like: it drives
//! the real Tier-1 spawn path the supervisor uses —
//! `backends::srt::render_settings` (the one srt renderer) →
//! `settings::wrap_launch(SrtWrapper, …)` → spawn — and asserts the
//! confinement property, not just that a process starts:
//!   - a wrapped write OUTSIDE the workspace is **blocked**;
//!   - a wrapped write INSIDE the workspace **succeeds**.
//!
//! `srt_live` (now gone) exercised the removed `SandboxRuntime::exec`
//! backend; the production tier-1 path is `wrap_launch`, which had never
//! had a confinement test. This closes that gap.
//!
//! Auto-skips unless `bun`, the `srt` CLI entry, and (Linux) `bwrap` +
//! `socat` are all present — `srt` itself refuses to run without the
//! Linux sandbox deps. Run: `cargo test -p ship-isolation --test
//! tier1_srt_confinement_live -- --nocapture`.

use std::path::PathBuf;

use ship_isolation::backends::render_settings;
use ship_isolation::settings::{wrap_launch, WrapContext};
use ship_isolation::tier::IsolationStrategy;
use ship_substrate::isolation::{FsMount, MountMode, NetworkPolicy, SandboxPolicy};

fn which(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(name))
            .find(|p| p.is_file())
    })
}

fn locate_srt() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("SHIP_SRT_ENTRY") {
        let pb = PathBuf::from(p);
        return pb.is_file().then_some(pb);
    }
    std::env::var_os("HOME").and_then(|home| {
        let p = PathBuf::from(home).join(
            ".bun/install/global/node_modules/@anthropic-ai/sandbox-runtime/dist/cli.js",
        );
        p.is_file().then_some(p)
    })
}

fn skip(reason: &str) {
    eprintln!("tier1_srt_confinement_live: skipping — {reason}");
}

/// Build + write the srt settings for a workspace via the single
/// production renderer, then return the wrapped launch command.
fn wrap(
    bun: &std::path::Path,
    srt: &std::path::Path,
    settings_path: &std::path::Path,
    workspace: &std::path::Path,
    shell_cmd: &str,
) -> ship_isolation::settings::WrappedLaunch {
    let mut policy = SandboxPolicy::workspace_default(workspace.to_path_buf());
    policy.network = NetworkPolicy::DenyAll;
    policy.mounts = vec![FsMount {
        host_path: workspace.to_path_buf(),
        sandbox_path: workspace.to_path_buf(),
        mode: MountMode::ReadWrite,
    }];
    let rendered = render_settings(&policy);
    std::fs::write(settings_path, serde_json::to_vec_pretty(&rendered).unwrap()).unwrap();

    wrap_launch(
        IsolationStrategy::SrtWrapper,
        PathBuf::from("sh"),
        vec!["-c".into(), shell_cmd.into()],
        &WrapContext {
            bun_bin: Some(bun),
            srt_entry: Some(srt),
            srt_settings_path: Some(settings_path),
            harness_native_flag: None,
        },
    )
    .expect("SrtWrapper wrap_launch must produce a command")
}

async fn run(cmd: &ship_isolation::settings::WrappedLaunch, cwd: &std::path::Path) {
    let status = tokio::process::Command::new(&cmd.bin)
        .args(&cmd.args)
        .current_dir(cwd)
        .status()
        .await
        .expect("spawn wrapped command");
    let _ = status; // exit code irrelevant — we assert on filesystem effect
}

#[tokio::test]
async fn srt_wrapper_blocks_write_outside_workspace() {
    let Some(bun) = which("bun") else {
        return skip("bun not on PATH");
    };
    let Some(srt) = locate_srt() else {
        return skip("srt entry not found (set SHIP_SRT_ENTRY or `bun add -g`)");
    };
    if which("bwrap").is_none() || which("socat").is_none() {
        return skip("bwrap / socat not installed (srt needs them on Linux)");
    }

    let work = std::env::temp_dir().join(format!("ship-t1-ws-{}", std::process::id()));
    std::fs::create_dir_all(&work).unwrap();
    let settings = work.join("srt-settings.json");
    let outside =
        std::env::temp_dir().join(format!("ship-t1-escape-{}.txt", std::process::id()));
    let inside = work.join("inside.txt");
    let _ = std::fs::remove_file(&outside);
    let _ = std::fs::remove_file(&inside);

    // 1. Escape attempt — write outside the workspace must be blocked.
    let escape = wrap(&bun, &srt, &settings, &work, &format!("echo escape > {}", outside.display()));
    run(&escape, &work).await;
    let escaped = std::fs::metadata(&outside).is_ok();

    // 2. Allowed write — inside the workspace must succeed.
    let ok = wrap(&bun, &srt, &settings, &work, &format!("echo ok > {}", inside.display()));
    run(&ok, &work).await;
    let wrote_inside = std::fs::read_to_string(&inside).ok();

    let _ = std::fs::remove_file(&outside);
    let _ = std::fs::remove_dir_all(&work);

    assert!(
        !escaped,
        "SrtWrapper must BLOCK a write outside the workspace ({}), but the file was created",
        outside.display()
    );
    assert_eq!(
        wrote_inside.as_deref().map(str::trim),
        Some("ok"),
        "SrtWrapper must ALLOW a write inside the workspace"
    );
}

/// Characterization test (pins a documented LIMITATION, not a guarantee):
/// Tier-1 SrtWrapper confines *writes* but **not reads** — a wrapped
/// process can read a host file outside the workspace and copy it in.
/// Verified empirically here. This is the threat model's Tier-1
/// **T-FS-READ ⚠️ / T-CRED ❌**: process sandboxes run as the host user and
/// `render_settings` emits empty `denyRead`, so read confinement is a
/// Tier-2 (container) property, not a Tier-1 one. Use Tier 2+ for read /
/// credential confinement.
///
/// This test exists to *pin* that boundary: if it ever flips (reads start
/// being blocked — e.g. someone adds a default-deny `denyRead`), that's a
/// deliberate posture change and the threat model + matrix must be
/// re-evaluated. It is NOT a license to leak — it documents why Tier 1 is
/// the weaker tier.
#[tokio::test]
async fn srt_wrapper_does_not_confine_reads_tier1_limitation() {
    let Some(bun) = which("bun") else {
        return skip("bun not on PATH");
    };
    let Some(srt) = locate_srt() else {
        return skip("srt entry not found");
    };
    if which("bwrap").is_none() || which("socat").is_none() {
        return skip("bwrap / socat not installed");
    }

    let work = std::env::temp_dir().join(format!("ship-t1r-ws-{}", std::process::id()));
    std::fs::create_dir_all(&work).unwrap();
    let settings = work.join("srt-settings.json");
    let secret =
        std::env::temp_dir().join(format!("ship-t1r-secret-{}.txt", std::process::id()));
    std::fs::write(&secret, b"TOPSECRET").unwrap();
    let exfil = work.join("exfil.txt");
    let _ = std::fs::remove_file(&exfil);

    // Read the outside secret, write it inside the workspace (writing
    // inside is allowed; the question is whether the READ succeeds).
    let cmd = wrap(
        &bun,
        &srt,
        &settings,
        &work,
        &format!("cat {} > {} 2>/dev/null || true", secret.display(), exfil.display()),
    );
    run(&cmd, &work).await;
    let leaked = std::fs::read_to_string(&exfil).unwrap_or_default();

    let _ = std::fs::remove_file(&secret);
    let _ = std::fs::remove_dir_all(&work);

    assert!(
        leaked.contains("TOPSECRET"),
        "EXPECTED Tier-1 limitation: SrtWrapper does not confine reads, so \
         the host secret should be readable. If this assertion fails, reads \
         are now being blocked — a deliberate posture change; update the \
         threat model (T-FS-READ/T-CRED) + compat-matrix and flip this test \
         to assert confinement."
    );
}
