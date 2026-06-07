//! `~/.ship/runtime/` — bundled toolchain (sui-compiler).
//!
//! Since #163 the daemon bundles assets IN-PROCESS via Rolldown (compiled into
//! the binary) — no bun, no `bundler.ts` worker. The only runtime component is
//! the sui-compiler binary the bundler's `load` hook spawns to compile `.sui`.
//! Path computation stays in `home::ShipHome` per the home-layout ADR.
//!
//! # Layout
//!
//! ```text
//! ~/.ship/runtime/
//!   sui-compiler    # Rust release build, spawned to compile .sui
//! ```

use std::path::PathBuf;

use crate::home::ShipHome;

#[derive(Debug, thiserror::Error)]
pub enum ToolchainError {
    #[error("Ship toolchain not installed at {path}.\n  Run `ship runtime install` to fetch.")]
    Missing { path: PathBuf },
    #[error("Ship toolchain at {root} is incomplete: missing {component}.\n  Run `ship runtime install` to repair.")]
    Incomplete { root: PathBuf, component: &'static str },
    #[cfg(unix)]
    #[error("Ship toolchain binary {path} is not executable.\n  Run `ship runtime install` to repair.")]
    NotExecutable { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, ToolchainError>;

/// Verify the toolchain: the `runtime/` dir and the `sui-compiler` binary.
pub fn ensure_present(home: &ShipHome) -> Result<()> {
    check_runtime_dir(home)?;
    check_sui_compiler(home)?;
    Ok(())
}

pub fn check_runtime_dir(home: &ShipHome) -> Result<()> {
    let dir = home.runtime_dir();
    if !dir.exists() {
        return Err(ToolchainError::Missing { path: dir });
    }
    Ok(())
}

pub fn check_sui_compiler(home: &ShipHome) -> Result<()> {
    let path = home.runtime_sui_compiler();
    require_executable(home, &path, "sui-compiler")
}

#[cfg(unix)]
fn require_executable(
    home: &ShipHome,
    path: &std::path::Path,
    component: &'static str,
) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if !path.exists() {
        return Err(ToolchainError::Incomplete { root: home.runtime_dir(), component });
    }
    let meta = std::fs::metadata(path).map_err(|_| ToolchainError::Incomplete {
        root: home.runtime_dir(),
        component,
    })?;
    let mode = meta.permissions().mode();
    if mode & 0o111 == 0 {
        return Err(ToolchainError::NotExecutable { path: path.to_path_buf() });
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_executable(
    home: &ShipHome,
    path: &std::path::Path,
    component: &'static str,
) -> Result<()> {
    if !path.exists() {
        return Err(ToolchainError::Incomplete { root: home.runtime_dir(), component });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn touch(path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, b"").unwrap();
    }

    #[cfg(unix)]
    fn touch_exec(path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        touch(path);
        let mut perm = std::fs::metadata(path).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(path, perm).unwrap();
    }

    #[cfg(not(unix))]
    fn touch_exec(path: &std::path::Path) {
        touch(path);
    }

    #[test]
    fn ensure_present_errors_when_runtime_dir_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let err = ensure_present(&home).unwrap_err();
        assert!(matches!(err, ToolchainError::Missing { .. }));
        assert!(format!("{err}").contains("ship runtime install"));
    }

    #[test]
    fn ensure_present_errors_when_sui_compiler_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        let err = ensure_present(&home).unwrap_err();
        assert!(format!("{err}").contains("sui-compiler"));
    }

    #[test]
    fn ensure_present_passes_with_sui_compiler() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        touch_exec(&home.runtime_sui_compiler());
        ensure_present(&home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn ensure_present_errors_on_non_executable_compiler() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        touch_exec(&home.runtime_sui_compiler());
        let mut perm = std::fs::metadata(home.runtime_sui_compiler()).unwrap().permissions();
        perm.set_mode(0o644);
        std::fs::set_permissions(home.runtime_sui_compiler(), perm).unwrap();
        let err = ensure_present(&home).unwrap_err();
        assert!(matches!(err, ToolchainError::NotExecutable { .. }));
    }
}
