//! Ship home layout — typed accessors for the ship home.
//!
//! Per ADR `home-layout-and-scope-convention`, this module is the only
//! permitted origin of paths under the ship home. Raw `.join(".ship")` or
//! `home_dir().join(".ship/…")` elsewhere is flagged by
//! `scripts/lint/check-raw-ship-paths.sh`.

pub mod daemon;
pub mod device_id;
pub mod ensure;
pub mod project;

pub use daemon::{
    DaemonFile, DaemonHandle, current_daemon_dir, daemon_dir_for, derive_worktree_port,
    discover_running_daemon, path_hash, worktree_port_for, worktree_root_for,
};
pub use ensure::ensure_layout;
pub use project::ProjectRoot;
// `ShipHome` / `Db` (and the `machine/db/` subdir const) live in the
// compiler-free `ship-paths` leaf; re-exported so `ship_substrate::ShipHome`,
// `ship_substrate::home::ShipHome`, and internal `crate::ShipHome` consumers
// keep resolving unchanged.
pub use device_id::read_or_generate_device_id;
pub use ship_paths::{Db, ShipHome};
