//! `~/.ship/runtime/` — bundled toolchain (bun + sui-compiler + bundler.ts).
//!
//! Checks that the three required components exist. Path computation stays
//! in `home::ShipHome` per the home-layout ADR.
//!
//! # Layout
//!
//! ```text
//! ~/.ship/runtime/
//!   bun             # ~100 MB, single static binary
//!   sui-compiler    # Rust release build
//!   bundler.ts      # Bun worker script (copied in by `ship runtime install`)
//! ```

use std::path::PathBuf;

use ship_substrate::ShipHome;

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

/// Verify that every required toolchain component exists on disk. Returns
/// the first missing or unusable piece, in the order:
/// 1. `runtime/` directory
/// 2. `bun` binary (executable on Unix)
/// 3. `sui-compiler` binary (executable on Unix)
/// 4. `bundler.ts` script
pub fn ensure_present(home: &ShipHome) -> Result<()> {
    check_runtime_dir(home)?;
    check_bun(home)?;
    check_sui_compiler(home)?;
    check_bundler_ts(home)?;
    Ok(())
}

pub fn check_runtime_dir(home: &ShipHome) -> Result<()> {
    let dir = home.runtime_dir();
    if !dir.exists() {
        return Err(ToolchainError::Missing { path: dir });
    }
    Ok(())
}

pub fn check_bun(home: &ShipHome) -> Result<()> {
    let path = home.runtime_bun();
    require_executable(home, &path, "bun")
}

pub fn check_sui_compiler(home: &ShipHome) -> Result<()> {
    let path = home.runtime_sui_compiler();
    require_executable(home, &path, "sui-compiler")
}

pub fn check_bundler_ts(home: &ShipHome) -> Result<()> {
    let path = home.runtime_bundler_ts();
    if !path.exists() {
        return Err(ToolchainError::Incomplete {
            root: home.runtime_dir(),
            component: "bundler.ts",
        });
    }
    Ok(())
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

    fn populate_full_toolchain(home: &ShipHome) {
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        touch_exec(&home.runtime_bun());
        touch_exec(&home.runtime_sui_compiler());
        touch(&home.runtime_bundler_ts());
    }

    #[test]
    fn ensure_present_errors_when_runtime_dir_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let err = ensure_present(&home).unwrap_err();
        assert!(matches!(err, ToolchainError::Missing { .. }));
        let msg = format!("{err}");
        assert!(msg.contains("ship runtime install"));
    }

    #[test]
    fn ensure_present_errors_when_bun_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        let err = ensure_present(&home).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("bun"), "{msg}");
        assert!(msg.contains("ship runtime install"));
    }

    #[test]
    fn ensure_present_errors_when_bundler_ts_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        touch_exec(&home.runtime_bun());
        touch_exec(&home.runtime_sui_compiler());
        // bundler.ts NOT created
        let err = ensure_present(&home).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("bundler.ts"), "{msg}");
    }

    #[test]
    fn ensure_present_passes_with_full_toolchain() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        populate_full_toolchain(&home);
        ensure_present(&home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn ensure_present_errors_on_non_executable_bun() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        populate_full_toolchain(&home);
        let mut perm = std::fs::metadata(home.runtime_bun()).unwrap().permissions();
        perm.set_mode(0o644);
        std::fs::set_permissions(home.runtime_bun(), perm).unwrap();
        let err = ensure_present(&home).unwrap_err();
        assert!(matches!(err, ToolchainError::NotExecutable { .. }));
    }
}
