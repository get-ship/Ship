//! Project layout, identity, discovery, registry, global state.
//!
//! L0 surface only. `init_project` and `init_project_with_registry` live in
//! ship-platform / runtime (they depend on config schemas and registry).

pub const SHIP_DIR_NAME: &str = ".ship";
pub const PRIMARY_CONFIG_FILE: &str = "ship.jsonc";
pub const LEGACY_CONFIG_FILE: &str = "ship.toml";

mod app_state;
mod discovery;
mod global;
mod identity;
mod paths;
mod registry;

pub use app_state::{
    AppState, get_active_project_global, get_recent_projects_global, load_app_state,
    save_app_state, set_active_project_global,
};
pub use discovery::{BOUNDARY_FILE, get_project_dir, resolve_project_ship_dir, ship_dir_from_path};
pub use global::get_global_dir;
pub use identity::{
    ensure_project_id, get_project_name, project_slug_from_ship_dir, sanitize_file_name,
};
pub use paths::{
    adrs_dir, agents_ns, features_dir, legacy_agents_skills_dir, legacy_project_skills_dir,
    legacy_project_skills_dir_candidates, mcp_config_path, modes_dir, notes_dir,
    permissions_config_path, project_ns, project_skills_dir, releases_dir, rules_dir, skills_dir,
    teams_dir, upcoming_releases_dir, user_skills_dir, vision_doc_path,
    vision_template_path,
};
pub use registry::{
    ProjectEntry, ProjectRegistry, get_registry_path, list_registered_projects, load_registry,
    register_project, save_registry, unregister_project,
};

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    use super::app_state::normalize_app_state_paths;
    use super::discovery::{canonicalize_lossy, parse_main_worktree_root};
    use super::global::{is_likely_rust_test_binary, parse_test_global_dir_pid};
    use super::identity::is_transient_project_root;

    #[test]
    fn resolve_project_ship_dir_follows_git_worktree_pointer() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = crate::ShipHome::project_ship_dir(&main_root);
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let wt_root = tmp.path().join("worktrees").join("feature-auth");
        let wt_nested = wt_root.join("src").join("app");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&wt_nested)?;
        fs::write(
            wt_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let resolved = resolve_project_ship_dir(&wt_nested).expect("expected .ship resolution");
        assert_eq!(resolved, canonicalize_lossy(&main_ship));
        Ok(())
    }

    #[test]
    fn resolve_project_ship_dir_prefers_main_ship_over_worktree_copy() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = crate::ShipHome::project_ship_dir(&main_root);
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let wt_root = tmp.path().join("worktrees").join("feature-auth");
        let wt_nested = wt_root.join("src").join("app");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(crate::ShipHome::project_ship_dir(&wt_root))?;
        fs::create_dir_all(&wt_nested)?;
        fs::write(
            wt_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let resolved = resolve_project_ship_dir(&wt_nested).expect("expected .ship resolution");
        assert_eq!(resolved, canonicalize_lossy(&main_ship));
        Ok(())
    }

    #[test]
    fn resolve_project_ship_dir_follows_relative_git_worktree_pointer() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = crate::ShipHome::project_ship_dir(&main_root);
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let wt_root = tmp.path().join("worktrees").join("feature-auth");
        let wt_nested = wt_root.join("src");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&wt_nested)?;

        fs::write(
            wt_root.join(".git"),
            "gitdir: ../../main/.git/worktrees/feature-auth\n",
        )?;

        let resolved = resolve_project_ship_dir(&wt_nested).expect("expected .ship resolution");
        assert_eq!(resolved, canonicalize_lossy(&main_ship));
        Ok(())
    }

    #[test]
    fn parse_main_worktree_root_returns_none_for_main_checkout() {
        let porcelain = "worktree /home/user/ship\nHEAD abc123\nbranch refs/heads/main\n\nworktree /home/user/ship-worktrees/fix-auth\nHEAD def456\nbranch refs/heads/fix-auth\n";
        let result = parse_main_worktree_root(porcelain, Path::new("/home/user/ship"));
        assert!(result.is_none());
    }

    #[test]
    fn parse_main_worktree_root_returns_main_for_linked_worktree() {
        let porcelain = "worktree /home/user/ship\nHEAD abc123\nbranch refs/heads/main\n\nworktree /home/user/ship-worktrees/fix-auth\nHEAD def456\nbranch refs/heads/fix-auth\n";
        let result =
            parse_main_worktree_root(porcelain, Path::new("/home/user/ship-worktrees/fix-auth"));
        assert_eq!(result, Some(PathBuf::from("/home/user/ship")));
    }

    #[test]
    fn parse_main_worktree_root_handles_subdirectory_of_linked_worktree() {
        let porcelain = "worktree /home/user/ship\nHEAD abc123\nbranch refs/heads/main\n\nworktree /home/user/ship-worktrees/fix-auth\nHEAD def456\nbranch refs/heads/fix-auth\n";
        let result = parse_main_worktree_root(
            porcelain,
            Path::new("/home/user/ship-worktrees/fix-auth/src/app"),
        );
        assert_eq!(result, Some(PathBuf::from("/home/user/ship")));
    }

    #[test]
    fn parse_main_worktree_root_returns_none_for_unrecognized_dir() {
        let porcelain = "worktree /home/user/ship\nHEAD abc123\nbranch refs/heads/main\n";
        let result = parse_main_worktree_root(porcelain, Path::new("/home/other/project"));
        assert!(result.is_none());
    }

    #[test]
    fn resolve_ship_dir_uses_git_worktree_list_for_linked_worktree() -> Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let wt_root = tmp.path().join("wt");

        fs::create_dir_all(&main_root)?;

        let status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&main_root)
            .output();
        if status.is_err() || !status.unwrap().status.success() {
            return Ok(());
        }

        for (key, val) in [("user.email", "test@test.com"), ("user.name", "Test")] {
            std::process::Command::new("git")
                .args(["config", key, val])
                .current_dir(&main_root)
                .output()?;
        }

        fs::write(main_root.join("README"), "init")?;
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&main_root)
            .output()?;
        std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&main_root)
            .output()?;

        std::process::Command::new("git")
            .args(["branch", "fix-auth"])
            .current_dir(&main_root)
            .output()?;

        let wt_out = std::process::Command::new("git")
            .args([
                "worktree",
                "add",
                wt_root.to_str().unwrap(),
                "fix-auth",
            ])
            .current_dir(&main_root)
            .output()?;
        if !wt_out.status.success() {
            return Ok(());
        }

        let main_ship = crate::ShipHome::project_ship_dir(&main_root);
        fs::create_dir_all(&main_ship)?;

        let wt_ship = crate::ShipHome::project_ship_dir(&wt_root);
        fs::create_dir_all(&wt_ship)?;

        let wt_nested = wt_root.join("src").join("deep");
        fs::create_dir_all(&wt_nested)?;

        let resolved =
            resolve_project_ship_dir(&wt_nested).expect("must resolve to main .ship/");
        assert_eq!(
            resolved,
            canonicalize_lossy(&main_ship),
            "worktree must resolve to the main repo's .ship/, not the worktree copy"
        );
        Ok(())
    }

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
    fn ensure_project_id_populates_missing_id() -> Result<()> {
        let tmp = tempdir()?;
        let ship = crate::ShipHome::project_ship_dir(tmp.path());
        fs::create_dir_all(&ship)?;
        fs::write(
            ship.join(PRIMARY_CONFIG_FILE),
            r#"{"module": {"name": "legacy-project", "version": "1.0.0"}}"#,
        )?;

        let id = ensure_project_id(&ship)?;
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

    #[test]
    fn normalize_app_state_paths_canonicalizes_and_dedupes_entries() -> Result<()> {
        let tmp = tempdir()?;
        let root = tmp.path().join("workspace");
        let ship = root.join(SHIP_DIR_NAME);
        fs::create_dir_all(&ship)?;

        let mut state = AppState {
            active_project: Some(root.clone()),
            recent_projects: vec![root.clone(), ship.clone()],
        };

        assert!(normalize_app_state_paths(&mut state));
        let canonical_ship = canonicalize_lossy(&ship);
        assert_eq!(state.active_project, Some(canonical_ship.clone()));
        assert_eq!(state.recent_projects, vec![canonical_ship]);
        Ok(())
    }

    #[test]
    fn normalize_app_state_paths_reports_no_change_for_canonical_state() -> Result<()> {
        let tmp = tempdir()?;
        let ship = tmp.path().join("workspace").join(SHIP_DIR_NAME);
        fs::create_dir_all(&ship)?;
        let canonical_ship = canonicalize_lossy(&ship);

        let mut state = AppState {
            active_project: Some(canonical_ship.clone()),
            recent_projects: vec![canonical_ship],
        };

        assert!(!normalize_app_state_paths(&mut state));
        Ok(())
    }
}
