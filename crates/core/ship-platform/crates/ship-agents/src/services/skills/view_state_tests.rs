//! Tests for `views::skills_state`. Kept in a sibling file so the module
//! itself stays under the 300-line cap.
//!
//! These drive `scopes_from` + `*_with_scopes` variants directly so
//! parallel execution doesn't race on the process-global `SHIP_DEV_PROJECT_DIR` env
//! var. Only the `list_skills(ship_dir)` public surface exercises the
//! env-var path, and we cover that with a best-effort subset check.

use crate::services::skills::view_state::{self, *};
use std::path::Path;
use tempfile::tempdir;

// Pull in pub(crate) helpers explicitly — glob (`*`) only imports public.
use view_state::scan_skill_dir;

fn write_skill(dir: &Path, name: &str, fm: &str, body: &str) {
    let d = dir.join(name);
    std::fs::create_dir_all(&d).unwrap();
    let content = format!("---\n{fm}---\n{body}");
    std::fs::write(d.join("SKILL.md"), content).unwrap();
}

#[test]
fn empty_returns_empty() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship).unwrap();
    let scopes = scopes_from(&ship, None);
    assert!(list_skills_with_scopes(&scopes).is_empty());
}

#[test]
fn parses_frontmatter_into_summary() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    std::fs::create_dir_all(space_root.join("skills")).unwrap();
    write_skill(
        &space_root.join("skills"),
        "tdd",
        "name: tdd\ndescription: Test first.\ntags: [testing, rust]\n",
        "# Body\n",
    );
    let scopes = scopes_from(&ship, None);
    let out = list_skills_with_scopes(&scopes);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "tdd");
    assert_eq!(out[0].source, SOURCE_GLOBAL);
    assert!(out[0].tags.contains(&"testing".to_string()));
}
#[test]
fn load_skill_reads_body_and_vars() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    std::fs::create_dir_all(space_root.join("skills").join("x").join("assets")).unwrap();
    write_skill(
        &space_root.join("skills"),
        "x",
        "name: x\ndescription: d\n",
        "# hi\nbody",
    );
    std::fs::write(
        space_root.join("skills").join("x").join("assets").join("vars.json"),
        r#"{"foo": {"type":"string"}, "bar": {"type":"bool"}}"#,
    )
    .unwrap();
    let scopes = scopes_from(&ship, None);
    let d = load_skill_with_scopes(&scopes, "x").unwrap();
    assert!(d.body.contains("# hi"));
    assert_eq!(d.variables, vec!["bar".to_string(), "foo".to_string()]);
}

#[test]
fn load_skill_returns_none_for_missing() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship).unwrap();
    let scopes = scopes_from(&ship, None);
    assert!(load_skill_with_scopes(&scopes, "nope").is_none());
}
#[test]
fn merges_project_and_global_scopes() {
    // Global space root hosts two skills ("g-only" and "shared").
    let global_tmp = tempdir().unwrap();
    let global_ship = global_tmp.path().join(".ship");
    let global_root = global_ship.parent().unwrap_or(&global_ship);
    std::fs::create_dir_all(global_root.join("skills")).unwrap();
    write_skill(
        &global_root.join("skills"),
        "g-only",
        "name: g-only\ndescription: global\n",
        "body",
    );
    write_skill(
        &global_root.join("skills"),
        "shared",
        "name: shared\ndescription: global version\n",
        "global body",
    );

    // Project space root hosts "p-only" and "shared" — project wins on
    // Marker cutover: the project scope anchor IS the stack root.
    let project_tmp = tempdir().unwrap();
    let project_root = project_tmp.path();
    let project_space = project_root;
    std::fs::create_dir_all(project_space.join("skills")).unwrap();
    std::fs::write(
        project_root.join("ship.config.ts"),
        "export default {};\n",
    )
    .unwrap();
    write_skill(
        &project_space.join("skills"),
        "p-only",
        "name: p-only\ndescription: project\n",
        "body",
    );
    write_skill(
        &project_space.join("skills"),
        "shared",
        "name: shared\ndescription: project version\n",
        "project body",
    );

    let scopes = scopes_from(&global_ship, Some(project_root));
    let out = list_skills_with_scopes(&scopes);
    let detail = load_skill_with_scopes(&scopes, "shared").unwrap();

    let ids: Vec<&str> = out.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"g-only"), "global skill present: {ids:?}");
    assert!(ids.contains(&"p-only"), "project skill present: {ids:?}");
    assert!(ids.contains(&"shared"), "shared skill present: {ids:?}");
    assert_eq!(
        out.iter().filter(|s| s.id == "shared").count(),
        1,
        "dedup keeps one copy of shared"
    );

    let g_only = out.iter().find(|s| s.id == "g-only").unwrap();
    assert_eq!(g_only.source, SOURCE_GLOBAL);
    let p_only = out.iter().find(|s| s.id == "p-only").unwrap();
    assert_eq!(p_only.source, SOURCE_PROJECT);
    let shared = out.iter().find(|s| s.id == "shared").unwrap();
    assert_eq!(shared.source, SOURCE_PROJECT, "project wins on id collision");

    assert_eq!(detail.description, "project version");
    assert_eq!(detail.source, SOURCE_PROJECT);
}

#[test]
fn scope_skipped_when_project_matches_global() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    std::fs::create_dir_all(space_root.join("skills")).unwrap();
    write_skill(&space_root.join("skills"), "dup", "name: dup\n", "body");
    // Passing the same path as both scopes collapses to a single (global)
    // scan — the guard on `p != ship_dir` keeps us from listing duplicate
    // entries labeled with both sources.
    let scopes = scopes_from(&ship, Some(&ship));
    let out = list_skills_with_scopes(&scopes);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].source, SOURCE_GLOBAL);
}

#[test]
fn doc_skills_with_subdirs_are_detected() {
    // Doc-skills are subdir-per-skill just like smart skills: the scanner
    // looks for SKILL.md in each subdirectory and doesn't care whether the
    // parent is named `skills/` or `docs/`. The only difference is the
    // `kind` inference from the path.
    let tmp = tempdir().unwrap();
    let docs = tmp.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    write_skill(
        &docs,
        "ship-runtime",
        "name: ship-runtime\ndescription: daemon guts\n",
        "body",
    );
    let out = scan_skill_dir(&docs, SOURCE_PROJECT);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "ship-runtime");
    assert!(matches!(out[0].kind, SkillKind::Docs));
    assert_eq!(out[0].source, SOURCE_PROJECT);
}

#[test]
fn scan_skill_dir_skips_entries_missing_skill_md() {
    let tmp = tempdir().unwrap();
    let skills = tmp.path().join("skills");
    std::fs::create_dir_all(skills.join("no-manifest")).unwrap();
    write_skill(&skills, "good", "name: good\n", "body");
    let out = scan_skill_dir(&skills, SOURCE_GLOBAL);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "good");
}


#[test]
fn list_skills_public_discovers_our_tmp_skill() {
    // End-to-end smoke for the `list_skills(ship_dir)` public API —
    // ensures the `resolve_scopes` path works. The project scope from
    // `get_project_dir(None)` may resolve to the enclosing repo's space
    // root during tests, which is fine: we only assert that our unique
    // skill appears (not that the list is exclusive to it).
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    std::fs::create_dir_all(space_root.join("skills")).unwrap();
    write_skill(
        &space_root.join("skills"),
        "zzz-tmp-unique-abcdef",
        "name: zzz-tmp-unique-abcdef\n",
        "body",
    );
    let out = list_skills(&ship);
    assert!(
        out.iter().any(|s| s.id == "zzz-tmp-unique-abcdef"),
        "unique tmp skill must be discovered"
    );
}

#[test]
fn summary_includes_files_and_indicators() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    let skill_dir = space_root.join("skills").join("x");
    std::fs::create_dir_all(skill_dir.join("assets")).unwrap();
    std::fs::create_dir_all(skill_dir.join("references")).unwrap();
    std::fs::create_dir_all(skill_dir.join("evals")).unwrap();
    write_skill(&space_root.join("skills"), "x", "name: x\n", "body");
    std::fs::write(skill_dir.join("assets").join("vars.json"), "{}").unwrap();
    std::fs::write(skill_dir.join("references").join("ref.md"), "# ref").unwrap();
    std::fs::write(skill_dir.join("evals").join("evals.json"), "[]").unwrap();
    let scopes = scopes_from(&ship, None);
    let out = list_skills_with_scopes(&scopes);
    let s = out.iter().find(|s| s.id == "x").unwrap();
    assert!(s.files.contains(&"SKILL.md".to_string()));
    assert!(s.files.contains(&"assets/vars.json".to_string()));
    assert!(s.has_vars);
    assert!(s.has_refs);
    assert!(s.has_evals);
}
#[test]
fn detail_includes_files() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    let skill_dir = space_root.join("skills").join("y");
    std::fs::create_dir_all(skill_dir.join("scripts")).unwrap();
    write_skill(&space_root.join("skills"), "y", "name: y\n", "body");
    std::fs::write(skill_dir.join("scripts").join("run.sh"), "#!/bin/bash").unwrap();
    let scopes = scopes_from(&ship, None);
    let d = load_skill_with_scopes(&scopes, "y").unwrap();
    assert!(d.files.contains(&"SKILL.md".to_string()));
    assert!(d.files.contains(&"scripts/run.sh".to_string()));
}
#[test]
fn load_skill_file_reads_content() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    std::fs::create_dir_all(space_root.join("skills")).unwrap();
    write_skill(&space_root.join("skills"), "z", "name: z\n", "hello");
    let content = load_skill_file(&ship, "z", "SKILL.md");
    assert!(content.is_some());
    assert!(content.unwrap().contains("hello"));
}

#[test]
fn load_skill_file_rejects_path_traversal() {
    let tmp = tempdir().unwrap();
    let ship = tmp.path().join(".ship");
    let space_root = ship.parent().unwrap_or(&ship);
    std::fs::create_dir_all(space_root.join("skills")).unwrap();
    write_skill(&space_root.join("skills"), "w", "name: w\n", "body");
    std::fs::write(tmp.path().join("secret.txt"), "top secret").unwrap();
    let content = load_skill_file(&ship, "w", "../../secret.txt");
    assert!(content.is_none());
}
