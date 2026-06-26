//! Project-dir discovery — the compiler-free slice of `ship_substrate::project`.
//!
//! Holds `.ship/` discovery (walk ancestors, follow git-worktree pointers,
//! redirect linked worktrees to the main checkout) and the `SHIP_DIR_NAME`
//! SSOT. The identity / paths / global / space submodules with heavier deps
//! stay in `ship-substrate::project` and re-export the items here.

/// On-disk name of the project ship directory.
pub const SHIP_DIR_NAME: &str = ".ship";

mod discovery;

pub use discovery::{
    BOUNDARY_FILE, get_project_dir, resolve_project_ship_dir, ship_dir_from_path,
};
