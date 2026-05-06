//! Populate `~/.ship/runtime/` from local sources.
//!
//! Two source paths are supported:
//!   - **Explicit override** via `InstallOptions::bun_source` /
//!     `InstallOptions::sui_compiler_source` — used by `just dev-runtime-install`
//!     and tests, which already know exactly where the binaries are.
//!   - **Auto-discovery** — looks for `bun` on `PATH` and a
//!     `target/release/sui-compiler` walking up from the working directory.
//!     The error returned on failure tells the user which knob to flip.
//!
//! After the binaries land, this module writes a pinned `package.json`,
//! invokes the just-installed `bun` to populate `node_modules/`, and runs
//! `ensure_present` to verify the result.
//!
//! Future: a CDN-fetch mode (`InstallOptions::release`) will download
//! pre-built tarballs verified by checksum. Out of scope for this commit.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ShipHome;

use super::ensure::{ToolchainError, ensure_present};

/// Pinned versions for the bundled JS toolchain. Bumping these is a
/// deliberate Ship-version event — the daemon and the toolchain ship
/// together, so consumers don't end up with mismatched plugin / runtime
/// pairs. Update in lockstep with `packages/vite-plugin-sui/package.json`
/// and `apps/spike-bun-vite/package.json`.
///
/// `@ship/vite-plugin-sui` is **not** listed here. It's an unpublished
/// monorepo package; Step 5 wires it in by copying the plugin source
/// directly into `<runtime>/node_modules/@ship/vite-plugin-sui/` after
/// `bun install` resolves the registry deps. Adding it as a registry
/// dep here would 404 against npmjs.
const PINNED_VITE: &str = "^7.3.1";

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("could not locate `bun` binary: {0}.\n  Pass --bun <path> or place bun on PATH.")]
    MissingBun(String),
    #[error("could not locate `sui-compiler` binary: {0}.\n  Pass --sui-compiler <path> or run `cargo build -p sui-compiler --release`.")]
    MissingSuiCompiler(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("`bun install` failed in {dir}\n{stderr}")]
    BunInstall { dir: PathBuf, stderr: String },
    #[error("toolchain failed verification after install: {0}")]
    Verification(#[from] ToolchainError),
}

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Explicit path to the Bun binary to copy in. When `None`, falls back
    /// to `which bun` on PATH.
    pub bun_source: Option<PathBuf>,
    /// Explicit path to the sui-compiler binary to copy in. When `None`,
    /// falls back to walking up from cwd looking for
    /// `target/release/sui-compiler`.
    pub sui_compiler_source: Option<PathBuf>,
    /// When true, wipe `~/.ship/runtime/` before reinstalling. The default
    /// is false (idempotent overwrite of individual files).
    pub clean: bool,
    /// When true, skip the `bun install` step that populates `node_modules/`.
    /// Useful in tests that supply a fake bun. The toolchain will be flagged
    /// incomplete by `ensure_present`; that's the caller's signal that
    /// `node_modules/` still needs populating.
    pub skip_bun_install: bool,
}

/// Idempotent install. Safe to call repeatedly with the same options.
pub fn install(home: &ShipHome, opts: InstallOptions) -> Result<(), InstallError> {
    if opts.clean {
        wipe(&home.runtime_dir())?;
    }
    mkdir(&home.runtime_dir())?;

    let bun_src = locate_bun(opts.bun_source.as_deref())?;
    let compiler_src = locate_sui_compiler(opts.sui_compiler_source.as_deref())?;

    copy_executable(&bun_src, &home.runtime_bun())?;
    copy_executable(&compiler_src, &home.runtime_sui_compiler())?;
    write_package_json(home)?;

    if !opts.skip_bun_install {
        run_bun_install(home)?;
    }

    if !opts.skip_bun_install {
        ensure_present(home)?;
    }
    Ok(())
}

/// Equivalent to `install` with `clean = true`. Kept as a distinct entry
/// point so `ship runtime update` can mean "throw away and refetch" without
/// callers having to construct InstallOptions by hand.
pub fn update(home: &ShipHome, mut opts: InstallOptions) -> Result<(), InstallError> {
    opts.clean = true;
    install(home, opts)
}

fn locate_bun(explicit: Option<&Path>) -> Result<PathBuf, InstallError> {
    if let Some(p) = explicit {
        if !p.exists() {
            return Err(InstallError::MissingBun(format!("{} does not exist", p.display())));
        }
        return Ok(p.to_path_buf());
    }
    which("bun").map_err(InstallError::MissingBun)
}

fn locate_sui_compiler(explicit: Option<&Path>) -> Result<PathBuf, InstallError> {
    if let Some(p) = explicit {
        if !p.exists() {
            return Err(InstallError::MissingSuiCompiler(format!(
                "{} does not exist",
                p.display()
            )));
        }
        return Ok(p.to_path_buf());
    }
    walk_for_target_binary("sui-compiler")
        .map_err(InstallError::MissingSuiCompiler)
}

/// Walk up from cwd looking for a Cargo workspace root with
/// `target/release/<name>` or `target/debug/<name>`. Mirrors the JS-side
/// resolver in `@ship/vite-plugin-sui` so dev workflows behave identically
/// across both surfaces.
fn walk_for_target_binary(name: &str) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    let mut cur = cwd.as_path();
    let mut topmost: Option<PathBuf> = None;
    for _ in 0..16 {
        if cur.join("Cargo.toml").exists() {
            topmost = Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    let root = topmost.ok_or_else(|| "no Cargo workspace found above cwd".to_string())?;
    let release = root.join("target").join("release").join(name);
    if release.exists() {
        return Ok(release);
    }
    let debug = root.join("target").join("debug").join(name);
    if debug.exists() {
        return Ok(debug);
    }
    Err(format!(
        "neither {} nor {} exists",
        release.display(),
        debug.display()
    ))
}

fn which(name: &str) -> Result<PathBuf, String> {
    let path_var = std::env::var_os("PATH").ok_or_else(|| "PATH unset".to_string())?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!("{name} not found on PATH"))
}

fn mkdir(path: &Path) -> Result<(), InstallError> {
    std::fs::create_dir_all(path).map_err(|source| InstallError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn wipe(path: &Path) -> Result<(), InstallError> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(path).map_err(|source| InstallError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_executable(src: &Path, dst: &Path) -> Result<(), InstallError> {
    if let Some(parent) = dst.parent() {
        mkdir(parent)?;
    }
    std::fs::copy(src, dst).map_err(|source| InstallError::Io {
        path: dst.to_path_buf(),
        source,
    })?;
    set_executable(dst)?;
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), InstallError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perm = std::fs::metadata(path)
        .map_err(|source| InstallError::Io { path: path.to_path_buf(), source })?
        .permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(path, perm).map_err(|source| InstallError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), InstallError> {
    Ok(())
}

fn write_package_json(home: &ShipHome) -> Result<(), InstallError> {
    let body = format!(
        r#"{{
  "name": "ship-runtime-toolchain",
  "private": true,
  "type": "module",
  "version": "0.1.0",
  "description": "Bundled JS toolchain for Ship: vite + peer deps. Managed by `ship runtime install`. @ship/vite-plugin-sui is not listed here — Step 5 copies its source into node_modules/ after bun install completes.",
  "dependencies": {{
    "vite": "{vite}"
  }}
}}
"#,
        vite = PINNED_VITE,
    );
    let path = home.runtime_package_json();
    std::fs::write(&path, body).map_err(|source| InstallError::Io {
        path,
        source,
    })?;
    Ok(())
}

fn run_bun_install(home: &ShipHome) -> Result<(), InstallError> {
    let dir = home.runtime_dir();
    let output = Command::new(home.runtime_bun())
        .args(["install", "--no-summary"])
        .current_dir(&dir)
        .output()
        .map_err(|source| InstallError::Io {
            path: home.runtime_bun(),
            source,
        })?;
    if !output.status.success() {
        return Err(InstallError::BunInstall {
            dir,
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[cfg(unix)]
    fn make_fake_executable(path: &Path, body: &str) {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).unwrap();
        }
        std::fs::write(path, body).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(path).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(path, perm).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn install_with_explicit_sources_populates_runtime_dir() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());

        let bun_src = tmp.path().join("fake-bun");
        let compiler_src = tmp.path().join("fake-compiler");
        make_fake_executable(&bun_src, "#!/bin/sh\necho fake-bun \"$@\"\n");
        make_fake_executable(&compiler_src, "#!/bin/sh\necho fake-compiler\n");

        install(
            &home,
            InstallOptions {
                bun_source: Some(bun_src),
                sui_compiler_source: Some(compiler_src),
                skip_bun_install: true,
                ..InstallOptions::default()
            },
        )
        .unwrap();

        assert!(home.runtime_bun().is_file());
        assert!(home.runtime_sui_compiler().is_file());
        assert!(home.runtime_package_json().is_file());

        let pkg = std::fs::read_to_string(home.runtime_package_json()).unwrap();
        assert!(pkg.contains("\"vite\""));
        assert!(pkg.contains("@ship/vite-plugin-sui"));
    }

    #[cfg(unix)]
    #[test]
    fn install_is_idempotent() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());

        let bun_src = tmp.path().join("fake-bun");
        let compiler_src = tmp.path().join("fake-compiler");
        make_fake_executable(&bun_src, "#!/bin/sh\nexit 0\n");
        make_fake_executable(&compiler_src, "#!/bin/sh\nexit 0\n");

        for _ in 0..3 {
            install(
                &home,
                InstallOptions {
                    bun_source: Some(bun_src.clone()),
                    sui_compiler_source: Some(compiler_src.clone()),
                    skip_bun_install: true,
                    ..InstallOptions::default()
                },
            )
            .unwrap();
        }
        assert!(home.runtime_bun().is_file());
    }

    #[test]
    fn install_errors_when_bun_source_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let err = install(
            &home,
            InstallOptions {
                bun_source: Some(tmp.path().join("nope")),
                sui_compiler_source: Some(tmp.path().join("nope2")),
                skip_bun_install: true,
                ..InstallOptions::default()
            },
        )
        .unwrap_err();
        assert!(matches!(err, InstallError::MissingBun(_)));
    }

    #[cfg(unix)]
    #[test]
    fn update_clears_runtime_dir_before_reinstalling() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());

        // Seed a stray file under runtime/ so we can detect the wipe.
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        let stray = home.runtime_dir().join("stray.txt");
        std::fs::write(&stray, b"garbage").unwrap();

        let bun_src = tmp.path().join("fake-bun");
        let compiler_src = tmp.path().join("fake-compiler");
        make_fake_executable(&bun_src, "#!/bin/sh\nexit 0\n");
        make_fake_executable(&compiler_src, "#!/bin/sh\nexit 0\n");

        update(
            &home,
            InstallOptions {
                bun_source: Some(bun_src),
                sui_compiler_source: Some(compiler_src),
                skip_bun_install: true,
                ..InstallOptions::default()
            },
        )
        .unwrap();

        assert!(!stray.exists(), "stray file should have been wiped");
        assert!(home.runtime_bun().is_file());
    }
}
