//! Project layout, discovery, registry, global state.
//!
//! L0 surface only. `init_project` and `init_project_with_registry` live in
//! ship-platform / runtime (they depend on config schemas and registry).
//!
//! Space IDENTITY is not derived here: the id lives in the `projects` pointer
//! (runtime.db), minted only by ship-space's `registry::add` and read via
//! [`resolve_active_space`]. `PRIMARY_CONFIG_FILE` (`ship.jsonc`) is project
//! configuration — never an identity source.

pub const PRIMARY_CONFIG_FILE: &str = "ship.jsonc";

// `.ship/` discovery + `SHIP_DIR_NAME` now live in the compiler-free
// `ship-paths` leaf so leaf crates can resolve project dirs without the
// substrate frontier. Re-exported here so `ship_substrate::project::*` and
// internal `crate::project::*` / `super::SHIP_DIR_NAME` consumers keep
// resolving unchanged.
pub use ship_paths::project::SHIP_DIR_NAME;

mod global;
mod paths;
mod space;

pub use ship_paths::project::{
    BOUNDARY_FILE, get_project_dir, resolve_project_ship_dir, ship_dir_from_path,
};
pub use global::get_global_dir;
pub use paths::{
    adrs_dir, agents_ns, features_dir, mcp_config_path, modes_dir, notes_dir,
    permissions_config_path, project_ns, project_skills_dir, releases_dir, rules_dir, skills_dir,
    teams_dir, upcoming_releases_dir, user_skills_dir, vision_doc_path,
    vision_template_path,
};
pub use space::resolve_active_space;
