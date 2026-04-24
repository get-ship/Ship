//! Ship home layout — typed accessors for `~/.ship/`.
//!
//! Per ADR `home-layout-and-scope-convention`, this module is the only
//! permitted origin of paths under `~/.ship/`. Raw `.join(".ship")` or
//! `home_dir().join(".ship/…")` elsewhere is flagged by
//! `scripts/check-raw-ship-paths.sh`.

pub mod daemon;
pub mod scope;
pub mod ship_home;
pub mod vector;

pub use daemon::DaemonFile;
pub use scope::Scope;
pub use ship_home::{Db, ShipHome};
pub use vector::Vector;
