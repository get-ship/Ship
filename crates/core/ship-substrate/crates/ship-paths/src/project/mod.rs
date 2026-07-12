//! Stack-root discovery — the compiler-free leaf.
//!
//! A STACK is marked by its manifest file: the directory containing
//! `ship.config.ts` is the stack root (package.json analogy). Discovery is a
//! pure ancestor walk for that file — no git awareness, no worktree
//! redirects, no sentinel files. Worktree-aware flows are ship-code's
//! concern and pass explicit roots (dispatch provisions the worktree and
//! hands the root over; daemons take `SHIP_DEV_PROJECT_DIR`).

/// Filename of the stack manifest — the on-disk marker of a stack root.
pub const STACK_CONFIG_FILENAME: &str = "ship.config.ts";

mod discovery;

pub use discovery::{get_stack_root, resolve_stack_root, stack_root_from_path};
