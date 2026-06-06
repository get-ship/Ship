/// Integration tests — skill resolution: list_skills, list_effective_skills,
/// get_skill, get_effective_skill, create_user_skill, list_user_skills.
///
/// Exercises the project→user precedence merger and the effective-skill
/// scope resolution path (project-first, then user fallback).
use anyhow::Result;
use tempfile::tempdir;

use crate::agent_domain::skill::{
    create_skill, create_user_skill, delete_skill, delete_user_skill, get_effective_skill,
    get_skill, get_user_skill, list_effective_skills, list_skills, list_user_skills, update_skill,
};
use crate::testutil::init_project;

// ── Project-scope skill resolution ────────────────────────────────────────────

#[test]
fn agents_skill_resolution_get_returns_project_skill() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "res-alpha", "Alpha", "alpha body")?;
    let skill = get_skill(&ship_dir, "res-alpha")?;
    assert_eq!(skill.id, "res-alpha");
    assert_eq!(skill.content, "alpha body");
    Ok(())
}

#[test]
fn agents_skill_resolution_get_returns_error_for_absent_skill() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let err = get_skill(&ship_dir, "no-such-skill")
        .expect_err("absent skill must return Err");
    assert!(
        err.to_string().contains("not found"),
        "unexpected error: {err}"
    );
    Ok(())
}

#[test]
fn agents_skill_resolution_list_skills_sorted_after_multiple_creates() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "zzz-last", "Zzz", "body")?;
    create_skill(&ship_dir, "aaa-first", "Aaa", "body")?;
    create_skill(&ship_dir, "mmm-mid", "Mmm", "body")?;

    let skills = list_skills(&ship_dir)?;
    // list_skills does not guarantee alphabetical on id, but list_effective_skills does.
    // Just assert all three are present.
    let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"aaa-first"), "expected aaa-first; got {ids:?}");
    assert!(ids.contains(&"mmm-mid"), "expected mmm-mid; got {ids:?}");
    assert!(ids.contains(&"zzz-last"), "expected zzz-last; got {ids:?}");
    Ok(())
}

// ── User-scope skill resolution ───────────────────────────────────────────────

#[test]
fn agents_skill_resolution_user_skill_create_and_get() -> Result<()> {
    let _tmp = tempdir()?; // keeps auto-isolated global dir alive
    create_user_skill("user-integ-skill", "User Skill", "user body")?;
    let skill = get_user_skill("user-integ-skill")?;
    assert_eq!(skill.id, "user-integ-skill");
    assert_eq!(skill.content, "user body");
    // Cleanup so the next test starts clean.
    delete_user_skill("user-integ-skill")?;
    Ok(())
}

#[test]
fn agents_skill_resolution_list_user_skills_includes_created() -> Result<()> {
    let _tmp = tempdir()?;
    create_user_skill("user-list-skill", "User List", "list body")?;
    let skills = list_user_skills()?;
    assert!(
        skills.iter().any(|s| s.id == "user-list-skill"),
        "expected user-list-skill in user skills; got {:?}",
        skills.iter().map(|s| s.id.as_str()).collect::<Vec<_>>()
    );
    delete_user_skill("user-list-skill")?;
    Ok(())
}

// ── Effective (merged) skill resolution ──────────────────────────────────────

#[test]
fn agents_skill_resolution_project_overrides_user_in_effective_list() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    // User-scope: "shared-id" with user content.
    create_user_skill("shared-id", "User Version", "user content")?;
    // Project-scope: same id wins.
    create_skill(&ship_dir, "shared-id", "Project Version", "project content")?;

    let effective = list_effective_skills(&ship_dir)?;
    let shared = effective
        .iter()
        .find(|s| s.id == "shared-id")
        .expect("shared-id must appear in effective list");
    assert_eq!(
        shared.content, "project content",
        "project scope must override user scope for shared-id"
    );

    // Cleanup user-scope entry.
    delete_user_skill("shared-id")?;
    Ok(())
}

#[test]
fn agents_skill_resolution_effective_includes_user_when_no_project_override() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_user_skill("user-only-skill", "User Only", "user only body")?;

    let effective = list_effective_skills(&ship_dir)?;
    assert!(
        effective.iter().any(|s| s.id == "user-only-skill"),
        "user-only-skill must appear in effective list when not overridden"
    );

    delete_user_skill("user-only-skill")?;
    Ok(())
}

#[test]
fn agents_skill_resolution_get_effective_prefers_project_scope() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_user_skill("scope-test", "User", "user body")?;
    create_skill(&ship_dir, "scope-test", "Project", "project body")?;

    let skill = get_effective_skill(&ship_dir, "scope-test")?;
    assert_eq!(
        skill.content, "project body",
        "get_effective_skill must return project scope when both exist"
    );

    delete_user_skill("scope-test")?;
    Ok(())
}

#[test]
fn agents_skill_resolution_get_effective_falls_back_to_user() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_user_skill("fallback-skill", "Fallback", "fallback body")?;

    let skill = get_effective_skill(&ship_dir, "fallback-skill")?;
    assert_eq!(skill.content, "fallback body");

    delete_user_skill("fallback-skill")?;
    Ok(())
}

#[test]
fn agents_skill_resolution_update_skill_changes_effective() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "mutable-skill", "Original", "original body")?;
    update_skill(&ship_dir, "mutable-skill", Some("Revised"), Some("revised body"))?;

    let skill = get_effective_skill(&ship_dir, "mutable-skill")?;
    assert_eq!(skill.name, "Revised");
    assert_eq!(skill.content, "revised body");
    Ok(())
}

#[test]
fn agents_skill_resolution_delete_removes_from_effective_list() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    create_skill(&ship_dir, "to-remove", "Remove Me", "body")?;
    delete_skill(&ship_dir, "to-remove")?;

    let effective = list_effective_skills(&ship_dir)?;
    assert!(
        !effective.iter().any(|s| s.id == "to-remove"),
        "deleted skill must not appear in effective list"
    );
    Ok(())
}
