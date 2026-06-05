//! Populate `~/.ship/runtime/` from local sources.
//!
//! Since #163 the daemon bundles assets IN-PROCESS via Rolldown — no bun, no
//! `bundler.ts` worker. The only runtime component is the `sui-compiler` binary
//! the bundler's `load` hook spawns. Source it explicitly
//! (`InstallOptions::sui_compiler_source`, used by `just dev-runtime-install`)
//! or auto-discover the freshest `target/{release,debug}/sui-compiler`.

use std::path::{Path, PathBuf};

use crate::home::ShipHome;

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
    crate::locate::walk_for_target_binary("sui-compiler")
        .map_err(InstallError::MissingSuiCompiler)
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
