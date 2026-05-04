//! Ship home layout — typed accessors for `~/.ship/`.
//!
//! Per ADR `home-layout-and-scope-convention`, this module is the only
//! permitted origin of paths under `~/.ship/`. Raw `.join(".ship")` or
//! `home_dir().join(".ship/…")` elsewhere is flagged by
//! `scripts/lint/check-raw-ship-paths.sh`.

pub mod daemon;
pub mod ensure;
pub mod project;
pub mod scope;
pub mod ship_home;
pub mod vector;

pub use daemon::{
    DaemonFile, DaemonHandle, current_daemon_dir, daemon_dir_for, derive_worktree_port,
    discover_running_daemon, path_hash, worktree_port_for,
};
pub use ensure::ensure_layout;
pub use project::ProjectRoot;
pub use scope::Scope;
pub use ship_home::{Db, ShipHome};
pub use vector::Vector;
