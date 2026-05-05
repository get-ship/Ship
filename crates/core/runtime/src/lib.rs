// ── L0 re-exports from ship-substrate ──────────────────────────────────────
// These modules now live in ship-substrate; runtime re-exports them for
// backward compatibility. Do not add L1 code here — it would bypass the
// compile-time barrier.
pub use ship_substrate::access;
pub use ship_substrate::data;
pub use ship_substrate::db;
pub use ship_substrate::env;
pub use ship_substrate::errors;
pub use ship_substrate::events;
pub use ship_substrate::fs_util;
pub use ship_substrate::git;
pub use ship_substrate::home;
pub use ship_substrate::log;
pub use ship_substrate::persistence;
pub use ship_substrate::projections;
pub use ship_substrate::projects;
pub use ship_substrate::supervisor;
pub use ship_substrate::trash;
pub use ship_substrate::{gen_nanoid, gen_ulid};

// ── L1 re-exports from ship-primitives ───────────────────────────────────────
// These modules now live in ship-primitives; runtime re-exports them for
// backward compatibility. Do not add L1 code here — it would bypass the
// compile-time barrier.
pub use ship_primitives::agents;
pub use ship_primitives::artifacts;
pub use ship_primitives::collections;
pub use ship_primitives::config;
pub use ship_primitives::hooks;
pub use ship_primitives::registry;
pub mod services;
pub use ship_primitives::sessions;

#[cfg(feature = "unstable")]
pub use ship_primitives::sync;

// ── project: L0 surface from substrate + L1 init ────────────────────────────
// project/ submodules (discovery, global, etc.) live in ship-substrate.
// init.rs (which uses config + registry) stays in runtime.
pub mod project;

// Re-export typed home accessors at the crate root so callers can write
// `runtime::ShipHome::project_ship_dir(&p)` without going through `home::`.
pub use home::{DaemonFile, Db, Scope, ShipHome, Vector};

// Backward-compatible module aliases.
// Canonical implementation lives under `runtime::agents::*`.
pub use agents::config as agent_config;
pub use agents::providers as agent_export;
pub use agents::permissions;
pub use agents::rule;
pub use agents::skill;
pub use agents::skill_paths;
pub use agents::skill_vars;
pub use config::catalog;
pub use config::security;

pub use agent_config::{
    ProviderSettings, WorkspaceAgentSettings, resolve_provider_settings,
    resolve_provider_settings_with_agent_override,
};
pub use agent_export::{
    ModelInfo, ProviderInfo, autodetect_providers, detect_binary, detect_version, disable_provider,
    enable_provider, list_models, list_providers,
};
pub use agents::import::{
    import_from_provider, import_permissions_from_provider, import_skills_from_provider,
};
pub use config::catalog::{CatalogEntry, CatalogKind, list_catalog, list_catalog_by_kind, search_catalog};
pub use config::{
    AgentLayerConfig, AgentProfile, HookConfig, HookTrigger, McpServerConfig,
    McpServerType, PermissionConfig, ProjectConfig,
    add_mcp_server, drain_legacy_project_config_kv, drain_legacy_settings_kv,
    ensure_standard_dirs,
    get_active_agent, get_all_settings, get_config, get_effective_config,
    get_setting, list_mcp_servers,
    remove_mcp_server, save_config,
    set_setting, setting_default,
};

pub use events::{
    EventEnvelope, list_events_since, query_events_since, read_events, read_recent_events,
};
pub use projections::job::{JobRecord, JobStatus, load_jobs, load_jobs_at, project as project_jobs};
pub use log::{LogEntry, log_action, log_action_by, read_log, read_log_entries};
pub use permissions::{
    AgentLimits, CommandPermissions, FsPermissions, NetworkPermissions, NetworkPolicy, Permissions,
    ToolPermissions, get_permissions, permission_tool_ids_for_provider, save_permissions,
};
pub use rule::{Rule, create_rule, delete_rule, get_rule, list_rules, update_rule};
pub use skill::{
    Skill, SkillInstallScope, SkillSource, create_skill, create_user_skill, delete_skill,
    delete_user_skill, get_effective_skill, get_skill, get_user_skill, install_skill_from_source,
    list_effective_skills, list_skills, list_user_skills, update_skill, update_user_skill,
};
pub use skill_paths::read_skill_paths;
pub use skill_vars::{
    delete_skill_var, get_skill_vars, list_skill_vars, reset_skill_vars, set_skill_var,
};

// ─── Re-exports from db (formerly state_db) ────────────────────────────────
pub use db::types::SessionDb;
pub use errors::{KernelError, WorkspaceError};

pub use trash::{TrashEntry, list_trash, purge_old_trash, restore_file, trash_file};

pub use sessions::{
    EndSessionRequest, ProviderMatrix, ProviderMatrixRepairReport, Session, SessionStatus,
    end_session, get_active_session, list_sessions, record_session_progress, start_session,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{get_project_dir, init_project, sanitize_file_name};
    use std::fs;
    use tempfile::tempdir;

    // status / git-config / agent-layer-is-not-file-backed / generate_gitignore
    // tests removed alongside the deleted ProjectConfig.{statuses,git,ai}
    // fields and the configurable .ship/.gitignore mechanism. The new
    // top-level .gitignore template is generated unconditionally by
    // init_project; that's covered by test_init_project below.

    #[test]
    fn test_log_action() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action(&project_dir, "test", "details")?;
        let entries = read_log_entries(&project_dir)?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "test");
        assert_eq!(entries[0].details, "details");
        Ok(())
    }

    #[test]
    fn test_log_action_by() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action_by(&project_dir, "agent", "create", "issue-abc.md")?;
        let entries = read_log_entries(&project_dir)?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].actor, "agent");
        Ok(())
    }

    #[test]
    fn test_read_log_entries() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action(&project_dir, "create", "first entry")?;
        log_action(&project_dir, "update", "second entry")?;
        let entries = read_log_entries(&project_dir)?;
        assert_eq!(entries.len(), 2);
        Ok(())
    }

    #[test]
    fn test_init_project() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        assert!(ship_path.exists());
        assert!(ship_path.join("project/specs").is_dir());
        assert!(ship_path.join("project/features").is_dir());
        assert!(ship_path.join("project/releases").is_dir());
        assert!(ship_path.join("project/adrs").is_dir());
        assert!(ship_path.join("project/notes").is_dir());
        assert!(ship_path.join("vision.md").is_file());
        let project_skills_dir = crate::project::skills_dir(&ship_path);
        assert!(project_skills_dir.is_dir());
        assert!(ship_path.join("project/releases/TEMPLATE.md").is_file());
        assert!(ship_path.join("project/features/TEMPLATE.md").is_file());
        assert!(ship_path.join("project/notes/TEMPLATE.md").is_file());
        let cfg = crate::config::get_config(Some(ship_path.clone()))?;
        assert!(cfg.agents.is_empty());
        assert!(!ship_path.join("events.ndjson").is_file());
        assert!(ship_path.join("ship.jsonc").is_file());
        assert!(project_skills_dir.join("task-policy/SKILL.md").is_file());
        let skill_content = fs::read_to_string(project_skills_dir.join("task-policy/SKILL.md"))?;
        assert!(skill_content.contains("task-policy"));
        assert!(skill_content.contains("Ship Workflow Policy"));
        Ok(())
    }

    #[test]
    fn test_init_project_idempotent() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let project_skills_dir = crate::project::skills_dir(&ship_path);
        let custom_skill = project_skills_dir.join("custom/SKILL.md");
        fs::create_dir_all(custom_skill.parent().expect("custom skill dir"))?;
        fs::write(
            &custom_skill,
            "---\nname: custom\ndescription: Custom test skill.\n---\n\nmy content",
        )?;
        let ship_path2 = init_project(tmp.path().to_path_buf())?;
        assert_eq!(ship_path, ship_path2);
        assert!(custom_skill.exists());
        assert_eq!(
            fs::read_to_string(&custom_skill)?,
            "---\nname: custom\ndescription: Custom test skill.\n---\n\nmy content"
        );
        assert!(project_skills_dir.join("task-policy/SKILL.md").is_file());
        Ok(())
    }

    #[test]
    fn test_init_project_feature_template_has_rich_fields() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let template = fs::read_to_string(ship_path.join("project/features/TEMPLATE.md"))?;
        assert!(template.contains("release_id"));
        assert!(template.contains("## Why"));
        assert!(template.contains("## Delivery Todos"));
        Ok(())
    }

    #[test]
    fn test_event_stream_since() -> anyhow::Result<()> {
        use crate::persistence::event_store::{EventStore, SqliteEventStore};
        use crate::events::types::{WorkspaceActivated, event_types};
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let before = chrono::Utc::now();
        let store = SqliteEventStore::new()?;
        store.append(&EventEnvelope::new(
            event_types::WORKSPACE_ACTIVATED,
            "feat-1",
            &WorkspaceActivated { agent_id: None, providers: vec![] },
        )?)?;
        let events = list_events_since(&ship_path, &before, None)?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].entity_id, "feat-1");
        Ok(())
    }

    #[test]
    fn test_event_append_and_read() -> anyhow::Result<()> {
        use crate::persistence::event_store::{EventStore, SqliteEventStore};
        use crate::events::types::{ProjectLog, event_types};
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let store = SqliteEventStore::new()?;
        store.append(&EventEnvelope::new(
            event_types::PROJECT_LOG,
            "project",
            &ProjectLog { action: "export".to_string(), details: String::new() },
        )?)?;
        let events = read_events(&ship_path)?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, event_types::PROJECT_LOG);
        Ok(())
    }

    #[test]
    fn test_legacy_migration() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_path = tmp.path().join(".project");
        fs::create_dir_all(&project_path)?;
        let project_dir = get_project_dir(Some(tmp.path().to_path_buf()))?;
        assert!(!project_path.exists());
        assert_eq!(
            fs::canonicalize(project_dir)?,
            fs::canonicalize(crate::ShipHome::project_ship_dir(tmp.path()))?
        );
        Ok(())
    }

    #[test]
    fn test_get_project_dir_resolves_main_ship_from_worktree() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = crate::ShipHome::project_ship_dir(&main_root);
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let worktree_root = tmp.path().join("worktrees").join("feature-auth");
        let worktree_nested = worktree_root.join("src").join("ui");
        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&worktree_nested)?;
        fs::write(
            worktree_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;
        let resolved = get_project_dir(Some(worktree_nested))?;
        assert_eq!(
            fs::canonicalize(resolved)?,
            fs::canonicalize(main_ship)?,
            "worktree paths should resolve to the main checkout .ship"
        );
        Ok(())
    }

    #[test]
    fn test_get_project_dir_prefers_main_ship_when_worktree_has_local_copy() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = crate::ShipHome::project_ship_dir(&main_root);
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let worktree_root = tmp.path().join("worktrees").join("feature-auth");
        let worktree_nested = worktree_root.join("src").join("ui");
        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(crate::ShipHome::project_ship_dir(&worktree_root))?;
        fs::create_dir_all(&worktree_nested)?;
        fs::write(
            worktree_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;
        let resolved = get_project_dir(Some(worktree_nested))?;
        assert_eq!(
            fs::canonicalize(resolved)?,
            fs::canonicalize(main_ship)?,
        );
        Ok(())
    }

    #[test]
    fn test_env_override() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let override_dir = tmp.path().join("override");
        fs::create_dir_all(&override_dir)?;
        unsafe {
            std::env::set_var("SHIP_DEV_PROJECT_DIR", override_dir.to_str().unwrap());
        }
        let project_dir = get_project_dir(None)?;
        assert_eq!(project_dir, override_dir);
        unsafe {
            std::env::remove_var("SHIP_DEV_PROJECT_DIR");
        }
        Ok(())
    }

    #[test]
    fn test_sanitize_file_name() {
        assert_eq!(sanitize_file_name("My Issue Title!"), "my-issue-title");
        assert_eq!(
            sanitize_file_name("Already_Sanitized-123"),
            "already_sanitized-123"
        );
    }
}
