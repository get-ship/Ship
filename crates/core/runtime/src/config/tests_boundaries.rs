use super::*;
use std::fs;
use tempfile::tempdir;

// ── Boundary 1: types.rs is pure data ─────────────────────────────────────────

#[test]
fn git_config_default_has_committed_files() {
    let git = GitConfig::default();
    assert!(
        git.commit.contains(&"ship.jsonc".to_string()),
        "GitConfig::default() must commit ship.jsonc"
    );
    assert!(
        git.ignore.is_empty(),
        "GitConfig::default() must have an empty ignore list"
    );
}

#[test]
fn project_config_roundtrips_json() {
    let config = ProjectConfig {
        name: Some("test-project".to_string()),
        description: Some("boundary test".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let restored: ProjectConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.name.as_deref(), Some("test-project"));
    assert_eq!(restored.description.as_deref(), Some("boundary test"));
    assert_eq!(restored.statuses.len(), config.statuses.len());
    assert_eq!(restored.namespaces, config.namespaces);
}

#[test]
fn agent_profile_has_required_fields() {
    let profile = AgentProfile {
        id: "test".to_string(),
        name: "Test".to_string(),
        ..Default::default()
    };
    assert_eq!(profile.id, "test");
    assert_eq!(profile.name, "Test");
    assert!(profile.active_tools.is_empty());
    assert!(profile.skills.is_empty());
    assert!(profile.rules.is_empty());
}

// ── Boundary 2: io.rs reads/writes atomically ──────────────────────────────────

#[test]
fn get_config_returns_default_for_missing_file() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let result = get_config(Some(tmp.path().to_path_buf()));
    assert!(result.is_ok(), "get_config on empty dir must not error");
    let config = result.unwrap();
    assert!(config.modes.is_empty());
    assert!(config.mcp_servers.is_empty());
    Ok(())
}

#[test]
fn save_then_get_config_roundtrips() -> anyhow::Result<()> {
    let tmp_a = tempdir()?;
    let config_a = ProjectConfig {
        name: Some("project-a".to_string()),
        description: Some("roundtrip test".to_string()),
        providers: vec!["boundary-provider".to_string()],
        ..Default::default()
    };
    save_config(&config_a, Some(tmp_a.path().to_path_buf()))?;
    let loaded = get_config(Some(tmp_a.path().to_path_buf()))?;
    assert_eq!(loaded.name.as_deref(), Some("project-a"));
    assert_eq!(loaded.description.as_deref(), Some("roundtrip test"));

    // Boundary: a second independent project must not inherit project-a's providers
    // from the global kv store. Fails today because save_runtime_settings ignores
    // its ship_dir argument and writes to a shared "runtime" namespace.
    let tmp_b = tempdir()?;
    let loaded_b = get_config(Some(tmp_b.path().to_path_buf()))?;
    assert_ne!(
        loaded_b.providers,
        vec!["boundary-provider".to_string()],
        "project-b must not inherit project-a's providers from global kv"
    );
    Ok(())
}

#[test]
fn save_config_creates_ship_jsonc() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    save_config(&ProjectConfig::default(), Some(tmp.path().to_path_buf()))?;
    assert!(
        tmp.path().join("ship.jsonc").exists(),
        "save_config must create ship.jsonc"
    );
    Ok(())
}

// ── Boundary 3: git.rs is isolated ─────────────────────────────────────────────

#[test]
fn generate_gitignore_creates_file() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    generate_gitignore(tmp.path(), &GitConfig::default())?;
    let gitignore = tmp.path().join(".gitignore");
    assert!(gitignore.exists(), ".gitignore must be created");
    let content = fs::read_to_string(&gitignore)?;
    assert!(!content.is_empty(), ".gitignore must not be empty");
    assert!(content.contains(".tmp-global/"), "must include .tmp-global/");
    Ok(())
}

#[test]
fn set_git_config_persists() -> anyhow::Result<()> {
    let tmp_a = tempdir()?;
    set_git_config(
        tmp_a.path(),
        GitConfig {
            ignore: vec!["vars/".to_string()],
            commit: vec![],
        },
    )?;
    let git_a = get_git_config(tmp_a.path())?;
    assert!(
        git_a.ignore.contains(&"vars/".to_string()),
        "ignore must contain vars/ after set"
    );

    // Boundary: an independent project must not see the same git settings.
    // Fails today because get_runtime_settings ignores its ship_dir argument
    // and reads from a shared "runtime" kv namespace.
    let tmp_b = tempdir()?;
    let git_b = get_git_config(tmp_b.path())?;
    assert!(
        !git_b.ignore.contains(&"vars/".to_string()),
        "git isolation: project-b must not inherit project-a's ignore entries"
    );
    Ok(())
}

// ── Boundary 4: discover.rs finds project root ─────────────────────────────────

#[test]
fn discover_returns_none_for_non_ship_dir() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let projects = discover_projects(tmp.path().to_path_buf())?;
    assert!(projects.is_empty(), "must return empty for dir with no .ship/ children");
    Ok(())
}

#[test]
fn discover_returns_path_for_ship_dir() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    // Create .ship/ship.jsonc directly under temp_dir, then discover from temp_dir.
    // Boundary: discover_projects should find a project when .ship/ is a direct child
    // of root. Fails today because the function only scans children for a nested .ship/,
    // not root itself.
    let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
    fs::create_dir_all(&ship_dir)?;
    fs::write(ship_dir.join("ship.jsonc"), b"{}")?;
    let projects = discover_projects(tmp.path().to_path_buf())?;
    assert!(
        !projects.is_empty(),
        "discover must find the project when .ship/ is a direct child of root"
    );
    Ok(())
}

// ── Boundary 5: mod.rs exports are stable ──────────────────────────────────────

#[test]
fn public_types_are_exported() {
    // Compile-time guard: removing any of these from mod.rs pub use breaks this test.
    #[allow(dead_code)]
    fn _check(
        _: AgentProfile,
        _: AiConfig,
        _: GitConfig,
        _: HookConfig,
        _: ProjectConfig,
        _: McpConfig,
        _: McpSection,
        _: McpServerConfig,
        _: PermissionConfig,
    ) {
    }
}
