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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::path::Path;

    // `.ship/` discovery tests moved with the code to `ship-paths`
    // (project::discovery). The tests below cover the global submodule
    // that remains in this crate.
    use super::global::{is_likely_rust_test_binary, parse_test_global_dir_pid};

    #[test]
    fn detects_rust_test_binaries_from_target_deps_path() {
        assert!(is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/deps/runtime-abc123"
        )));
        assert!(is_likely_rust_test_binary(Path::new(
            "C:\\repo\\target\\release\\deps\\runtime-abc123.exe"
        )));
        assert!(!is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/ship"
        )));
        assert!(!is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/deps/ship"
        )));
    }

    #[test]
    fn parses_test_global_dir_pid() -> Result<()> {
        assert_eq!(parse_test_global_dir_pid("ship-test-ship-home-123"), Some(123));
        assert_eq!(
            parse_test_global_dir_pid("ship-test-ship-home-123-ThreadId9"),
            Some(123)
        );
        assert_eq!(parse_test_global_dir_pid("ship-test-ship-home-abc"), None);
        assert_eq!(parse_test_global_dir_pid("other-prefix-123"), None);
        Ok(())
    }

}
