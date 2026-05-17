//! Tests for project bootstrap (`init_project`) and the substrate project
//! surface it builds on (discovery, identity, logging, event streams).

use super::init_project;
use ship_substrate::ShipHome;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::{list_events_since, read_events};
use ship_substrate::log::{log_action, log_action_by, read_log_entries};
use ship_substrate::project::{get_project_dir, sanitize_file_name, skills_dir};
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
    let project_dir = init_project(tmp.path())?;
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
    let project_dir = init_project(tmp.path())?;
    log_action_by(&project_dir, "agent", "create", "issue-abc.md")?;
    let entries = read_log_entries(&project_dir)?;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].actor, "agent");
    Ok(())
}

#[test]
fn test_read_log_entries() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let project_dir = init_project(tmp.path())?;
    log_action(&project_dir, "create", "first entry")?;
    log_action(&project_dir, "update", "second entry")?;
    let entries = read_log_entries(&project_dir)?;
    assert_eq!(entries.len(), 2);
    Ok(())
}

#[test]
fn test_init_project() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_path = init_project(tmp.path())?;
    assert!(ship_path.exists());
    assert!(ship_path.join("project/specs").is_dir());
    assert!(ship_path.join("project/features").is_dir());
    assert!(ship_path.join("project/releases").is_dir());
    assert!(ship_path.join("project/adrs").is_dir());
    assert!(ship_path.join("project/notes").is_dir());
    assert!(ship_path.join("vision.md").is_file());
    let project_skills_dir = skills_dir(&ship_path);
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
    let ship_path = init_project(tmp.path())?;
    let project_skills_dir = skills_dir(&ship_path);
    let custom_skill = project_skills_dir.join("custom/SKILL.md");
    fs::create_dir_all(custom_skill.parent().expect("custom skill dir"))?;
    fs::write(
        &custom_skill,
        "---\nname: custom\ndescription: Custom test skill.\n---\n\nmy content",
    )?;
    let ship_path2 = init_project(tmp.path())?;
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
    let ship_path = init_project(tmp.path())?;
    let template = fs::read_to_string(ship_path.join("project/features/TEMPLATE.md"))?;
    assert!(template.contains("release_id"));
    assert!(template.contains("## Why"));
    assert!(template.contains("## Delivery Todos"));
    Ok(())
}

#[test]
fn test_event_stream_since() -> anyhow::Result<()> {
    use ship_substrate::events::types::{WorkspaceActivated, event_types};
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    let tmp = tempdir()?;
    let ship_path = init_project(tmp.path())?;
    let before = chrono::Utc::now();
    let store = SqliteEventStore::new()?;
    store.append(&EventEnvelope::new(
        event_types::WORKSPACE_ACTIVATED,
        "feat-1",
        &WorkspaceActivated { agent_id: None, harnesses: vec![] },
    )?)?;
    let _ = &ship_path;
    let events = list_events_since(&before, None)?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity_id, "feat-1");
    Ok(())
}

#[test]
fn test_event_append_and_read() -> anyhow::Result<()> {
    use ship_substrate::events::types::{ProjectLog, event_types};
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    let tmp = tempdir()?;
    let ship_path = init_project(tmp.path())?;
    let store = SqliteEventStore::new()?;
    store.append(&EventEnvelope::new(
        event_types::PROJECT_LOG,
        "project",
        &ProjectLog { action: "export".to_string(), details: String::new() },
    )?)?;
    let _ = &ship_path;
    let events = read_events()?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, event_types::PROJECT_LOG);
    Ok(())
}

#[test]
fn test_get_project_dir_resolves_main_ship_from_worktree() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let main_root = tmp.path().join("main");
    let main_ship = ShipHome::project_ship_dir(&main_root);
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
    let main_ship = ShipHome::project_ship_dir(&main_root);
    let common_git = main_root.join(".git");
    let worktree_git = common_git.join("worktrees").join("feature-auth");
    let worktree_root = tmp.path().join("worktrees").join("feature-auth");
    let worktree_nested = worktree_root.join("src").join("ui");
    fs::create_dir_all(&main_ship)?;
    fs::create_dir_all(&worktree_git)?;
    fs::create_dir_all(ShipHome::project_ship_dir(&worktree_root))?;
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
