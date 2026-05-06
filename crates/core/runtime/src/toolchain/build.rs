//! `ship build` — drive the Vite-driven asset pipeline via the bundled bun.
//!
//! Shells to `~/.ship/runtime/bun` running
//! `<repo>/scripts/build-bundles-vite.mjs`. The script handles
//! pre-compilation of `.sui` sources via sui-compiler and walks the full
//! target list (Lit, SUI core, per-island, per-view).
//!
//! Errors propagate verbatim from the script's stderr — debugging a build
//! failure means reading the same output the script would have printed
//! interactively, no Rust-side rephrasing.

use std::path::PathBuf;
use std::process::Command;

use crate::ShipHome;

use super::ensure::ensure_present;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("toolchain not ready: {0}")]
    Toolchain(#[from] super::ensure::ToolchainError),
    #[error("build-bundles-vite.mjs not found at {path}")]
    ScriptMissing { path: PathBuf },
    #[error("io error launching bun: {0}")]
    Io(#[from] std::io::Error),
    #[error("`bun build-bundles-vite.mjs` exited with status {status}")]
    NonZero { status: i32 },
}

#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    /// Workspace root containing `scripts/build-bundles-vite.mjs`. When
    /// `None`, defaults to the current working directory.
    pub repo_root: Option<PathBuf>,
    /// When set, restrict to a single bundle target (passed through to
    /// the script as `--target <name>`).
    pub target: Option<String>,
}

/// Run the full bundle pipeline. Verifies the toolchain is present
/// before launching, so a stale `~/.ship/runtime/` produces a clear
/// "run `ship runtime install`" error rather than a confusing bun
/// failure.
pub fn run(home: &ShipHome, opts: BuildOptions) -> Result<(), BuildError> {
    ensure_present(home)?;
    let repo_root = match opts.repo_root {
        Some(p) => p,
        None => std::env::current_dir()?,
    };
    let script = repo_root
        .join("scripts")
        .join("build-bundles-vite.mjs");
    if !script.exists() {
        return Err(BuildError::ScriptMissing { path: script });
    }
    let mut cmd = Command::new(home.runtime_bun());
    cmd.arg(&script).current_dir(&repo_root);
    if let Some(target) = opts.target {
        cmd.args(["--target", &target]);
    }
    // Inherit stdio — the script's progress lines and any sui-compiler
    // diagnostics go straight to the user's terminal.
    let status = cmd
        .stdin(std::process::Stdio::null())
        .status()?;
    if !status.success() {
        return Err(BuildError::NonZero {
            status: status.code().unwrap_or(-1),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn build_errors_when_toolchain_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let err = run(
            &home,
            BuildOptions {
                repo_root: Some(tmp.path().to_path_buf()),
                target: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, BuildError::Toolchain(_)));
    }

    #[cfg(unix)]
    #[test]
    fn build_errors_when_script_missing() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        // Populate just enough toolchain to pass ensure_present.
        std::fs::create_dir_all(home.runtime_node_modules()).unwrap();
        for path in [home.runtime_bun(), home.runtime_sui_compiler()] {
            std::fs::write(&path, b"#!/bin/sh\nexit 0\n").unwrap();
            let mut perm = std::fs::metadata(&path).unwrap().permissions();
            perm.set_mode(0o755);
            std::fs::set_permissions(&path, perm).unwrap();
        }
        std::fs::write(home.runtime_package_json(), b"{}").unwrap();

        let repo = tmp.path().join("fake-repo");
        std::fs::create_dir_all(&repo).unwrap();
        let err = run(
            &home,
            BuildOptions {
                repo_root: Some(repo),
                target: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, BuildError::ScriptMissing { .. }));
    }
}

