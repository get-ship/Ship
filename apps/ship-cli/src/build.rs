//! `ship build` — production build orchestrator.
//!
//! Wraps `just build` so external users without `just` can ship.
//! Streams each stage's output to stdout; exits non-zero on first
//! stage failure.

use std::process::{Command, Stdio};

pub fn run(dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        println!("ship build (dry-run): would invoke `just build`");
        return Ok(());
    }

    // Prefer `just build` when available — it's the canonical recipe
    // that the rest of the codebase documents. Fall back to a manual
    // sequence when `just` isn't on PATH.
    // Avoid pulling in the `which` crate — query PATH directly.
    let has_just = Command::new("just")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if has_just {
        let status = Command::new("just")
            .arg("build")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        if !status.success() {
            anyhow::bail!("just build exited with status {status}");
        }
        return Ok(());
    }

    // Manual fallback. Keep the steps in sync with the `build` recipe
    // in the workspace Justfile.
    let steps: &[(&str, &[&str])] = &[
        ("cargo", &["run", "-p", "xtask", "--quiet", "--", "gen-types"]),
        ("bun", &["install", "--frozen-lockfile"]),
        ("bun", &["run", "--filter", "@ship/set", "icons"]),
        ("bun", &["run", "--filter", "@ship/set", "build"]),
        ("bun", &["run", "--filter", "@ship/ui-components", "build"]),
        ("cargo", &["build", "--release"]),
    ];

    for (cmd, args) in steps {
        println!("→ {cmd} {}", args.join(" "));
        let status = Command::new(cmd)
            .args(*args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        if !status.success() {
            anyhow::bail!("{cmd} {args:?} exited with status {status}");
        }
    }
    Ok(())
}
