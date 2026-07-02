//! Global `~/.ship` directory — the XDG split delegate.
//!
//! The single global accessor [`get_global_dir`] is a thin wrapper around
//! `ShipHome::from_env().data_dir()`. Per the home-layout ADR, the four
//! XDG roots (config / data / state / cache) each get their own accessor
//! on `ShipHome`; this function is the canonical "DB opener" entry
//! point that returns the *data* path (where the per-scope SQLite
//! stores live). Production callers that need a different root call
//! `ShipHome::from_env()` and pick the right field directly.
//!
//! The test-context isolation guard lives in `ShipHome::from_env`; the
//! separate guard this module used to carry is gone — `ShipHome` is the
//! one source of truth. See `crates/core/ship-substrate/crates/ship-paths/`
//! for the canonical implementation.

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Returns the global *data* directory (the XDG `data` root, e.g.
/// `$XDG_DATA_HOME/ship` in production, `<base>/data` in SHIP_HOME mode).
///
/// DB openers want data; the four per-scope SQLite stores
/// (`runtime.db` / `account.db` / `spaces/<id>/space.db` / `kv.db`) live
/// under this root. Callers that need a different root (config / state
/// / cache / runtime) should call `ShipHome::from_env()` and pick the
/// field directly.
///
/// The test-context isolation guard (datasafety-shiphome) is
/// enforced inside `ShipHome::from_env`; this function is a thin
/// delegate, so a test that reaches this code path can never resolve
/// the operator's real on-disk state.
pub fn get_global_dir() -> Result<PathBuf> {
    Ok(crate::ShipHome::from_env().data_dir().to_path_buf())
}

/// Detect cargo's standard test-binary layout under `target/<profile>/deps/`.
/// Mirrors the same helper in `ship-paths/src/ship_home.rs`; the two
/// modules cannot share it without a circular dep through the
/// substrate frontier. Test-only — used by the legacy isolation-guard
/// fossil tests in `project.rs::tests`.
#[cfg(test)]
pub(crate) fn is_likely_rust_test_binary(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    // Match both standard target dirs and custom CARGO_TARGET_DIR layouts
    let in_test_deps = path_str.contains("/debug/deps/")
        || path_str.contains("/release/deps/")
        || path_str.contains("\\debug\\deps\\")
        || path_str.contains("\\release\\deps\\");
    if !in_test_deps {
        return false;
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains('-'))
}

/// The on-disk prefix used by the per-(pid, thread) test-isolation
/// slot in `ShipHome::from_env`. Test-only — kept here as a stable
/// string the legacy isolation tests reference.
#[cfg(test)]
const TEST_GLOBAL_DIR_PREFIX: &str = "ship-test-ship-home-";

/// Parse the pid out of a per-(pid, thread) test-isolation dir name.
/// Test-only — used by stale-dir sweeper fossil tests.
#[cfg(test)]
pub(crate) fn parse_test_global_dir_pid(name: &str) -> Option<u32> {
    name.strip_prefix(TEST_GLOBAL_DIR_PREFIX)?
        .split('-')
        .next()?
        .parse()
        .ok()
}
