//! Project layout, identity, discovery, registry, global state.
//!
//! L0 surface only. `init_project` and `init_project_with_registry` live in
//! ship-platform / runtime (they depend on config schemas and registry).

pub const PRIMARY_CONFIG_FILE: &str = "ship.jsonc";

// `.ship/` discovery + `SHIP_DIR_NAME` now live in the compiler-free
// `ship-paths` leaf so leaf crates can resolve project dirs without the
// substrate frontier. Re-exported here so `ship_substrate::project::*` and
// internal `crate::project::*` / `super::SHIP_DIR_NAME` consumers keep
// resolving unchanged.
pub use ship_paths::project::SHIP_DIR_NAME;

mod global;
mod identity;
mod paths;
mod space;

pub use ship_paths::project::{
    BOUNDARY_FILE, get_project_dir, resolve_project_ship_dir, ship_dir_from_path,
};
pub use global::get_global_dir;
pub use identity::{
    ensure_space_id, get_project_name, project_slug_from_ship_dir, sanitize_file_name,
};
pub use paths::{
    adrs_dir, agents_ns, features_dir, mcp_config_path, modes_dir, notes_dir,
    permissions_config_path, project_ns, project_skills_dir, releases_dir, rules_dir, skills_dir,
    teams_dir, upcoming_releases_dir, user_skills_dir, vision_doc_path,
    vision_template_path,
};
pub use space::resolve_active_space;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    // `.ship/` discovery tests moved with the code to `ship-paths`
    // (project::discovery). The tests below cover the global/identity
    // submodules that remain in this crate.
    use super::global::{is_likely_rust_test_binary, parse_test_global_dir_pid};
    use super::identity::is_transient_project_root;

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
        assert_eq!(parse_test_global_dir_pid("ship-test-global-123"), Some(123));
        assert_eq!(
            parse_test_global_dir_pid("ship-test-global-123-ThreadId9"),
            Some(123)
        );
        assert_eq!(parse_test_global_dir_pid("ship-test-global-abc"), None);
        assert_eq!(parse_test_global_dir_pid("other-prefix-123"), None);
        Ok(())
    }

    #[test]
    fn project_slug_uses_name_and_id_tokens() -> Result<()> {
        let tmp = tempdir()?;
        let ship = crate::ShipHome::project_ship_dir(tmp.path());
        fs::create_dir_all(&ship)?;
        fs::write(
            ship.join(PRIMARY_CONFIG_FILE),
            r#"{"id": "AbC123xy", "name": "Platform API"}"#,
        )?;

        let slug = project_slug_from_ship_dir(&ship);
        assert!(slug.ends_with("platform-api-abc123xy"));
        Ok(())
    }

    #[test]
    fn transient_path_detection_catches_temp_and_e2e_layouts() {
        assert!(is_transient_project_root(Path::new("/tmp/demo")));
        assert!(is_transient_project_root(Path::new("/private/tmp/demo")));
        assert!(is_transient_project_root(Path::new(
            "/repo/target/tmp/ship-e2e"
        )));
        assert!(is_transient_project_root(Path::new(
            "C:/Users/me/AppData/Local/Temp/ship-e2e"
        )));
        assert!(is_transient_project_root(Path::new(
            "/work/examples/e2e/sandbox"
        )));
        assert!(!is_transient_project_root(Path::new(
            "/Users/me/dev/real-project"
        )));
    }

    #[test]
    fn ensure_space_id_populates_missing_id() -> Result<()> {
        let tmp = tempdir()?;
        let ship = crate::ShipHome::project_ship_dir(tmp.path());
        fs::create_dir_all(&ship)?;
        fs::write(
            ship.join(PRIMARY_CONFIG_FILE),
            r#"{"module": {"name": "legacy-project", "version": "1.0.0"}}"#,
        )?;

        let id = ensure_space_id(&ship)?;
        assert!(!id.trim().is_empty());

        let raw = fs::read_to_string(ship.join(PRIMARY_CONFIG_FILE))?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)?;
        let persisted_id = parsed
            .get("id")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        assert_eq!(persisted_id, id);
        Ok(())
    }


}
