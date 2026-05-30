/// Integration tests — agent CRUD: skills, rules, permissions.
///
/// Each test:
///   1. Spins an ephemeral project via `init_project`.
///   2. Calls the public fn.
///   3. Asserts the return value.
///   4. Asserts the event emitted by querying the per-thread auto-isolated store.
use anyhow::Result;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::events::filter::EventFilter;
use ship_types::event_types::{
    SKILL_CREATED, SKILL_UPDATED, SKILL_DELETED, RULE_CREATED, RULE_UPDATED, RULE_DELETED,
    PERMISSIONS_UPDATED,
};
use tempfile::tempdir;

use crate::agents::skill::{create_skill, get_skill, update_skill, delete_skill, list_skills};
use crate::agents::rule::{create_rule, get_rule, update_rule, delete_rule, list_rules};
use crate::agents::permissions::{get_permissions, save_permissions, Permissions};
use crate::testutil::init_project;

fn query_events(event_type: &str) -> Vec<ship_types::envelope::EventEnvelope> {
    let store = SqliteEventStore::new().expect("open auto-isolated test store");
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .expect("query events")
}

// ── Skill CRUD ────────────────────────────────────────────────────────────────

#[test]
fn agents_skill_crud_create_emits_skill_created() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let skill = create_skill(&ship_dir, "integ-create", "Integration", "body text")?;
    assert_eq!(skill.id, "integ-create");
    assert_eq!(skill.name, "Integration");

    let events = query_events(SKILL_CREATED);
    assert!(
        events.iter().any(|e| e.entity_id == "integ-create"),
        "expected skill.created event with entity_id=integ-create; got {events:?}"
    );
    Ok(())
}

#[test]
fn agents_skill_crud_update_emits_skill_updated() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "integ-update", "Old Name", "old body")?;
    let updated = update_skill(&ship_dir, "integ-update", Some("New Name"), Some("new body"))?;
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.content, "new body");

    let events = query_events(SKILL_UPDATED);
    assert!(
        events.iter().any(|e| e.entity_id == "integ-update"),
        "expected skill.updated event; got {events:?}"
    );
    Ok(())
}

#[test]
fn agents_skill_crud_delete_emits_skill_deleted() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "integ-delete", "Del", "body")?;
    delete_skill(&ship_dir, "integ-delete")?;
    assert!(
        get_skill(&ship_dir, "integ-delete").is_err(),
        "skill should be absent after delete"
    );

    let events = query_events(SKILL_DELETED);
    assert!(
        events.iter().any(|e| e.entity_id == "integ-delete"),
        "expected skill.deleted event; got {events:?}"
    );
    Ok(())
}

#[test]
fn agents_skill_crud_list_returns_all() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "integ-a", "A", "body a")?;
    create_skill(&ship_dir, "integ-b", "B", "body b")?;

    let skills = list_skills(&ship_dir)?;
    let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"integ-a"), "expected integ-a in list: {ids:?}");
    assert!(ids.contains(&"integ-b"), "expected integ-b in list: {ids:?}");
    Ok(())
}

#[test]
fn agents_skill_crud_create_rejects_duplicate() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "integ-dup", "Dup", "body")?;
    let err = create_skill(&ship_dir, "integ-dup", "Dup2", "body2")
        .expect_err("duplicate skill must be rejected");
    assert!(
        err.to_string().contains("already exists"),
        "unexpected error: {err}"
    );
    Ok(())
}

// ── Rule CRUD ─────────────────────────────────────────────────────────────────

#[test]
fn agents_rule_crud_create_emits_rule_created() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let rule = create_rule(ship_dir.clone(), "integ-rule", "# Content\n")?;
    assert_eq!(rule.file_name, "integ-rule.md");

    let events = query_events(RULE_CREATED);
    assert!(
        events.iter().any(|e| e.entity_id == "integ-rule"),
        "expected rule.created event with entity_id=integ-rule; got {events:?}"
    );
    Ok(())
}

#[test]
fn agents_rule_crud_update_emits_rule_updated() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_rule(ship_dir.clone(), "integ-upd-rule", "# Initial\n")?;
    let updated = update_rule(ship_dir.clone(), "integ-upd-rule.md", "# Updated\n")?;
    assert_eq!(updated.content, "# Updated\n");

    let events = query_events(RULE_UPDATED);
    assert!(
        events.iter().any(|e| e.entity_id == "integ-upd-rule"),
        "expected rule.updated event; got {events:?}"
    );
    Ok(())
}

#[test]
fn agents_rule_crud_delete_emits_rule_deleted() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_rule(ship_dir.clone(), "integ-del-rule", "# To delete\n")?;
    delete_rule(ship_dir.clone(), "integ-del-rule")?;
    assert!(
        get_rule(ship_dir, "integ-del-rule").is_err(),
        "rule should be absent after delete"
    );

    let events = query_events(RULE_DELETED);
    assert!(
        events.iter().any(|e| e.entity_id == "integ-del-rule"),
        "expected rule.deleted event; got {events:?}"
    );
    Ok(())
}

#[test]
fn agents_rule_crud_list_empty_dir() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let rules = list_rules(ship_dir)?;
    assert!(rules.is_empty(), "expected empty rule list for fresh project");
    Ok(())
}

#[test]
fn agents_rule_crud_rejects_reserved_readme() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let err = create_rule(ship_dir, "README.md", "# Reserved\n")
        .expect_err("README.md must be rejected");
    assert!(
        err.to_string().to_ascii_lowercase().contains("reserved"),
        "unexpected error: {err}"
    );
    Ok(())
}

// ── Permissions CRUD ──────────────────────────────────────────────────────────

#[test]
fn agents_permissions_crud_save_emits_permissions_updated() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let perms = Permissions::default();
    save_permissions(ship_dir.clone(), &perms)?;

    let events = query_events(PERMISSIONS_UPDATED);
    assert!(
        !events.is_empty(),
        "expected permissions.updated event after save_permissions; got none"
    );
    Ok(())
}

#[test]
fn agents_permissions_crud_get_returns_default_when_absent() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    // No permissions file written — should return Permissions::default() not Err.
    let perms = get_permissions(ship_dir)?;
    // Default is valid; just verify it round-trips.
    let _json = serde_json::to_string(&perms).expect("default permissions must serialize");
    Ok(())
}

#[test]
fn agents_permissions_crud_save_and_get_round_trip() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let mut perms = Permissions::default();
    perms.tools.allow.push("Bash".to_string());
    save_permissions(ship_dir.clone(), &perms)?;

    let loaded = get_permissions(ship_dir)?;
    assert!(
        loaded.tools.allow.contains(&"Bash".to_string()),
        "expected Bash in allow list after round-trip; got {:?}",
        loaded.tools.allow
    );
    Ok(())
}
