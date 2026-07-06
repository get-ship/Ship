use ship_platform::config::{
    AgentProfile, HookConfig, McpConfig, McpSection, McpServerConfig, PermissionConfig,
    SpaceManifest, discover_projects, get_config, save_config,
};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn project_config_roundtrips_json() {
    let config = SpaceManifest {
        name: Some("test-project".to_string()),
        description: Some("boundary test".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let restored: SpaceManifest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.name.as_deref(), Some("test-project"));
    assert_eq!(restored.description.as_deref(), Some("boundary test"));
}

#[test]
fn agent_profile_has_required_fields() {
    let profile = AgentProfile {
        agent: compiler::ProfileMeta {
            id: "test".to_string(),
            name: "Test".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(profile.agent.id, "test");
    assert_eq!(profile.agent.name, "Test");
    assert!(profile.agent.active_tools.is_empty());
    assert!(profile.skills.refs.is_empty());
    assert!(profile.rules.refs.is_empty());
}

#[test]
fn get_config_returns_default_for_missing_file() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let result = get_config(Some(tmp.path().to_path_buf()));
    assert!(result.is_ok(), "get_config on empty dir must not error");
    let config = result.unwrap();
    assert!(config.agents.is_empty());
    assert!(config.mcp_servers.is_empty());
    Ok(())
}

#[test]
fn save_then_get_config_roundtrips() -> anyhow::Result<()> {
    let tmp_a = tempdir()?;
    let config_a = SpaceManifest {
        name: Some("project-a".to_string()),
        description: Some("roundtrip test".to_string()),
        harnesses: vec!["boundary-harness".to_string()],
        ..Default::default()
    };
    save_config(&config_a, Some(tmp_a.path().to_path_buf()))?;
    let loaded = get_config(Some(tmp_a.path().to_path_buf()))?;
    assert_eq!(loaded.name.as_deref(), Some("project-a"));
    assert_eq!(loaded.description.as_deref(), Some("roundtrip test"));

    let tmp_b = tempdir()?;
    let loaded_b = get_config(Some(tmp_b.path().to_path_buf()))?;
    assert_ne!(
        loaded_b.harnesses,
        vec!["boundary-harness".to_string()],
        "project-b must not inherit project-a's harnesses from global kv"
    );
    Ok(())
}

#[test]
fn save_config_creates_ship_jsonc() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    save_config(&SpaceManifest::default(), Some(tmp.path().to_path_buf()))?;
    assert!(
        tmp.path().join("ship.jsonc").exists(),
        "save_config must create ship.jsonc"
    );
    Ok(())
}

#[test]
fn discover_ignores_unregistered_ship_dirs() -> anyhow::Result<()> {
    // Enumeration reads the `projects` pointer, not the filesystem: a `.ship/`
    // directory on disk that was never registered is NOT a space.
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path().join("unregistered"));
    fs::create_dir_all(&ship_dir)?;
    fs::write(ship_dir.join("ship.jsonc"), b"{}")?;

    let projects = discover_projects(tmp.path().to_path_buf())?;
    assert!(
        projects.is_empty(),
        "unregistered .ship/ dirs must not enumerate as spaces"
    );
    Ok(())
}

// `worker_threads = 1` keeps `get_global_dir()`'s per-thread test isolation on
// ONE thread so the registering kernel and `discover_projects`' pointer read
// share the same runtime.db.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn discover_returns_registered_spaces_under_root() -> anyhow::Result<()> {
    ship_platform::install_defaults();
    let global = ship_substrate::project::get_global_dir()?;
    fs::create_dir_all(&global)?;
    let router = ship_substrate::events::new_isolated_for_test(global)?;
    let kernel = std::sync::Arc::new(ship_substrate::ports::KernelLog::new(router));

    let tmp = tempdir()?;
    let root = tmp.path().join("proj");
    fs::create_dir_all(&root)?;
    let canonical = fs::canonicalize(&root)?;
    let record = ship_space::registry::add(&kernel, &canonical, Some("proj")).await?;

    let projects = discover_projects(tmp.path().to_path_buf())?;
    assert_eq!(projects.len(), 1, "the registered space must enumerate");
    assert_eq!(projects[0].name, "proj");
    assert_eq!(projects[0].path, PathBuf::from(&record.path));

    // A root that does not enclose the registered path sees nothing.
    let elsewhere = tempdir()?;
    let none = discover_projects(elsewhere.path().to_path_buf())?;
    assert!(none.is_empty(), "enumeration is scoped to the given root");
    Ok(())
}

#[test]
fn public_types_are_exported() {
    #[allow(dead_code)]
    fn _check(
        _: AgentProfile,
        _: HookConfig,
        _: SpaceManifest,
        _: McpConfig,
        _: McpSection,
        _: McpServerConfig,
        _: PermissionConfig,
    ) {
    }
}
