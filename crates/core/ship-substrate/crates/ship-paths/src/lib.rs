//! ship-paths — compiler-free leaf crate for Ship's path layout.
//!
//! Holds the pieces of the home / project / env surface that depend only on
//! std + the `home` crate + `ship-types`. Leaf crates (e.g. `ship-git`) can
//! resolve `.ship/` paths through here without dragging in the substrate
//! compiler/runtime frontier.
//!
//! `ship-substrate` re-exports every item in this crate, so existing
//! `ship_substrate::{ShipHome, project::*, env::*}` consumers keep compiling
//! unchanged — there is exactly one definition of each, and it lives here.

pub mod env;
pub mod project;

mod daemon_file;
mod ship_home;

pub use daemon_file::DaemonFile;
pub use ship_home::{Db, DATA_DB_DIR, ShipHome, from_xdg};
