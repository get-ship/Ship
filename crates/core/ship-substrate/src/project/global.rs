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

use std::path::PathBuf;

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
