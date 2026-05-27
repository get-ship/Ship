//! Populate `~/.ship/runtime/` from local sources.
//!
//! Since #163 the daemon bundles assets IN-PROCESS via Rolldown — no bun, no
//! `bundler.ts` worker. The only runtime component is the `sui-compiler` binary
//! the bundler's `load` hook spawns. Source it explicitly
//! (`InstallOptions::sui_compiler_source`, used by `just dev-runtime-install`)
//! or auto-discover the freshest `target/{release,debug}/sui-compiler`.

use std::path::{Path, PathBuf};

use ship_substrate::ShipHome;

use super::ensure::{ToolchainError, ensure_present};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("could not locate `sui-compiler` binary: {0}.\n  Pass --sui-compiler <path> or run `cargo build -p sui-compiler --release`.")]
    MissingSuiCompiler(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("toolchain failed verification after install: {0}")]
    Verification(#[from] ToolchainError),
}

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Explicit path to the sui-compiler binary to copy in.
    pub sui_compiler_source: Option<PathBuf>,
    /// When true, wipe `~/.ship/runtime/` before reinstalling.
    pub clean: bool,
}

/// Idempotent install. Safe to call repeatedly with the same options.
pub fn install(home: &ShipHome, opts: InstallOptions) -> Result<(), InstallError> {
    if opts.clean {
        wipe(&home.runtime_dir())?;
    }
    mkdir(&home.runtime_dir())?;

    let compiler_src = locate_sui_compiler(opts.sui_compiler_source.as_deref())?;
    copy_executable(&compiler_src, &home.runtime_sui_compiler())?;

    ensure_present(home)?;
    Ok(())
}

/// Equivalent to `install` with `clean = true`.
pub fn update(home: &ShipHome, mut opts: InstallOptions) -> Result<(), InstallError> {
    opts.clean = true;
    install(home, opts)
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
    walk_for_target_binary("sui-compiler").map_err(InstallError::MissingSuiCompiler)
}

/// Walk up from cwd to the topmost Cargo workspace, then return the freshest
/// `target/{release,debug}/<name>`. `pub(crate)` so the bundler's compiler
/// locator (`bundler::compile::locate_sui_compiler`) reuses the one path.
pub(crate) fn walk_for_target_binary(name: &str) -> Result<PathBuf, String> {
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
        "neither {}/target/release/{name} nor target/debug/{name} found",
        root.display()
    ))
}

fn wipe(dir: &Path) -> Result<(), InstallError> {
    if dir.exists() {
        std::fs::remove_dir_all(dir).map_err(|source| InstallError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn mkdir(dir: &Path) -> Result<(), InstallError> {
    std::fs::create_dir_all(dir).map_err(|source| InstallError::Io {
        path: dir.to_path_buf(),
        source,
    })
}

fn copy_executable(src: &Path, dst: &Path) -> Result<(), InstallError> {
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
    fn install_with_explicit_compiler_populates_runtime_dir() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let compiler_src = tmp.path().join("fake-compiler");
        make_fake_executable(&compiler_src, "#!/bin/sh\nexit 0\n");

        install(
            &home,
            InstallOptions { sui_compiler_source: Some(compiler_src), clean: false },
        )
        .unwrap();

        assert!(home.runtime_sui_compiler().is_file());
    }

    #[test]
    fn install_errors_when_compiler_source_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let err = install(
            &home,
            InstallOptions {
                sui_compiler_source: Some(tmp.path().join("nope")),
                clean: false,
            },
        )
        .unwrap_err();
        assert!(matches!(err, InstallError::MissingSuiCompiler(_)));
    }

    #[cfg(unix)]
    #[test]
    fn update_clears_runtime_dir_before_reinstalling() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        let stray = home.runtime_dir().join("stray.txt");
        std::fs::write(&stray, b"garbage").unwrap();

        let compiler_src = tmp.path().join("fake-compiler");
        make_fake_executable(&compiler_src, "#!/bin/sh\nexit 0\n");
        update(
            &home,
            InstallOptions { sui_compiler_source: Some(compiler_src), clean: true },
        )
        .unwrap();

        assert!(!stray.exists(), "stray file should have been wiped");
        assert!(home.runtime_sui_compiler().is_file());
    }
}
