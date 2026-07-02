//! Ship home layout — typed accessors for `~/.ship/`.
//!
//! Per ADR `home-layout-and-scope-convention`, this module is the only
//! permitted origin of paths under `~/.ship/`. Raw `.join(".ship")` or
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
pub use ship_paths::{Db, DATA_DB_DIR, ShipHome};
pub use device_id::read_or_generate_device_id;
// Per modular-runtime-01, `Scope` and `Vector` live in `ship_types`. The
// home/{scope,vector}.rs shim files have been deleted; callers should
// import `ship_types::{Scope, Vector}` directly. This re-export stays so
// `ship_substrate::home::{Scope, Vector}` callers transition gracefully
// — remove when those drop to zero.
pub use ship_types::{Scope, Vector};
