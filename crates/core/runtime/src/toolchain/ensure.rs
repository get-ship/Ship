//! `~/.ship/runtime/` — bundled JS toolchain (Bun + Vite + sui-compiler + deps).
//!
//! `ship dev` and `ship publish` shell to `bun` here to drive Vite. This module
//! owns the existence checks and the human-readable error messages users see
//! when the toolchain is missing or corrupt. Path computation stays in
//! `home::ShipHome` per the home-layout ADR; this module composes those paths
//! with filesystem checks.
//!
//! # Layout
//!
//! ```text
//! ~/.ship/runtime/
//!   bun                      # ~100 MB, single static binary
//!   sui-compiler             # Rust release build
//!   node_modules/            # vite, @ship/vite-plugin-sui, peer deps
//!   package.json             # pinned versions for the JS toolchain
//!   bun.lock                 # bun lockfile
//! ```

use std::path::PathBuf;

use crate::ShipHome;

/// Why the toolchain wasn't usable. Each variant carries enough detail to
/// produce a self-explanatory error without the caller having to invent
/// language. The `Display` impl appends the canonical install hint.
#[derive(Debug, thiserror::Error)]
pub enum ToolchainError {
    /// `~/.ship/runtime/` does not exist on disk.
    #[error("Ship toolchain not installed at {path}.\n  Run `ship runtime install` to fetch.")]
    Missing { path: PathBuf },
    /// The toolchain dir exists but a required component is absent.
    #[error("Ship toolchain at {root} is incomplete: missing {component}.\n  Run `ship runtime install` to repair.")]
    Incomplete { root: PathBuf, component: &'static str },
    /// A required binary exists but is not executable. On Unix this means the
    /// install copy clobbered the +x bit; on Windows we'll never report this.
    #[cfg(unix)]
    #[error("Ship toolchain binary {path} is not executable.\n  Run `ship runtime install` to repair.")]
    NotExecutable { path: PathBuf },
}

/// Result alias used by every checker in this module. Kept narrow because the
/// caller almost always wants `Result<(), ToolchainError>` — there is no
/// success value to bubble up.
pub type Result<T> = std::result::Result<T, ToolchainError>;

/// Verify that every required toolchain component exists on disk. Returns
/// the first missing or unusable piece, in the order:
/// 1. `runtime/` directory exists at all
/// 2. `bun` binary present (and +x on unix)
/// 3. `sui-compiler` binary present (and +x on unix)
/// 4. `node_modules/` populated
/// 5. `package.json` present (pinned-version contract)
///
/// Callers that don't need every check (e.g. `ship build` only needs the
/// JS pipeline; a hypothetical Rust-only path could skip step 3) can call
/// the individual `check_*` helpers below.
pub fn ensure_present(home: &ShipHome) -> Result<()> {
    check_runtime_dir(home)?;
    check_bun(home)?;
    check_sui_compiler(home)?;
    check_node_modules(home)?;
    check_package_json(home)?;
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

pub fn check_node_modules(home: &ShipHome) -> Result<()> {
    let dir = home.runtime_node_modules();
    if !dir.exists() {
        return Err(ToolchainError::Incomplete {
            root: home.runtime_dir(),
            component: "node_modules/",
        });
    }
    Ok(())
}

pub fn check_package_json(home: &ShipHome) -> Result<()> {
    let path = home.runtime_package_json();
    if !path.exists() {
        return Err(ToolchainError::Incomplete {
            root: home.runtime_dir(),
            component: "package.json",
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
        return Err(ToolchainError::Incomplete {
            root: home.runtime_dir(),
            component,
        });
    }
    let meta = std::fs::metadata(path).map_err(|_| ToolchainError::Incomplete {
        root: home.runtime_dir(),
        component,
    })?;
    let mode = meta.permissions().mode();
    if mode & 0o111 == 0 {
        return Err(ToolchainError::NotExecutable {
            path: path.to_path_buf(),
        });
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
        return Err(ToolchainError::Incomplete {
            root: home.runtime_dir(),
            component,
        });
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
        std::fs::create_dir_all(home.runtime_node_modules()).unwrap();
        touch_exec(&home.runtime_bun());
        touch_exec(&home.runtime_sui_compiler());
        touch(&home.runtime_package_json());
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
    fn ensure_present_errors_when_node_modules_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        std::fs::create_dir_all(home.runtime_dir()).unwrap();
        touch_exec(&home.runtime_bun());
        touch_exec(&home.runtime_sui_compiler());
        touch(&home.runtime_package_json());
        let err = ensure_present(&home).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("node_modules"), "{msg}");
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
        // Strip exec bit from bun.
        let mut perm = std::fs::metadata(home.runtime_bun()).unwrap().permissions();
        perm.set_mode(0o644);
        std::fs::set_permissions(home.runtime_bun(), perm).unwrap();
        let err = ensure_present(&home).unwrap_err();
        assert!(matches!(err, ToolchainError::NotExecutable { .. }));
    }
}
