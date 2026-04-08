//! Skill variable state — vars.json parsing for use by the compiler/loader.
//!
//! CLI commands for vars have been removed. Manage skill variables
//! directly in .ship/skills/ state files.

pub mod schema;
pub mod state;

pub use schema::{load_vars_json, warn_invalid_enum_vars};
