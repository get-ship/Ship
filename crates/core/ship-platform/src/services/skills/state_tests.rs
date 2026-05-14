//! SkillService state tests. Pure logic over a tempdir-backed FilesService.

use crate::services::skills::state::*;
use crate::services::files::{FilesService, Scope};
use std::path::Path;
use tempfile::tempdir;

fn svc() -> (FilesService, tempfile::TempDir, tempfile::TempDir) {
    let user = tempdir().unwrap();
    let project = tempdir().unwrap();
    let svc = FilesService::new(
        user.path().to_path_buf(),
        Some(project.path().to_path_buf()),
    );
    (svc, user, project)
}

fn seed(s: &FilesService, scope: Scope, id: &str, fm: &str, body: &str) {
    let raw = format!("---\n{fm}\n---\n{body}");
    s.write(
        scope,
        Path::new(&format!("skills/{id}/SKILL.md")),
        raw.as_bytes(),
    )
    .unwrap();
}

#[test]
fn validate_id_accepts_schema_regex_shapes() {
    assert!(validate_id("a").is_ok());
    assert!(validate_id("my-skill").is_ok());
    assert!(validate_id("a1b2").is_ok());
}

#[test]
fn validate_id_rejects_leading_or_trailing_dash() {
    assert!(matches!(validate_id("-bad"), Err(Error::BadId(_))));
    assert!(matches!(validate_id("bad-"), Err(Error::BadId(_))));
}

#[test]
fn validate_id_rejects_uppercase_and_symbols() {
    assert!(matches!(validate_id("BadId"), Err(Error::BadId(_))));
    assert!(matches!(validate_id("bad.id"), Err(Error::BadId(_))));
    assert!(matches!(validate_id(""), Err(Error::BadId(_))));
}

#[test]
fn parse_skill_md_roundtrip() {
    let raw = "---\nname: commit\ndescription: make commits\ntags: [dev, git]\n---\nBody here\n";
    let (fm, body) = parse_skill_md(raw);
    assert_eq!(fm["name"], "commit");
    assert_eq!(fm["description"], "make commits");
    assert_eq!(fm["tags"], serde_json::json!(["dev", "git"]));
    assert_eq!(body, "Body here\n");
}

#[test]
fn list_dedupes_project_over_user() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "same", "name: user", "u");
    seed(&s, Scope::Project, "same", "name: proj", "p");
    seed(&s, Scope::User, "user-only", "name: user-only", "u");

    let items = list(&s);
    let ids: Vec<&str> = items.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["same", "user-only"]);
    let same = items.iter().find(|e| e.id == "same").unwrap();
    assert_eq!(same.scope, ScopeTag::Project);
    assert_eq!(same.name, "proj");
}

#[test]
fn load_returns_frontmatter_body_and_files() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "k", "name: k", "hello");
    // Add a companion file.
    s.write(
        Scope::Project,
        Path::new("skills/k/references/README.md"),
        b"ref",
    )
    .unwrap();
    let detail = load(&s, "k", None).unwrap();
    assert_eq!(detail.id, "k");
    assert_eq!(detail.scope, ScopeTag::Project);
    assert_eq!(detail.frontmatter["name"], "k");
    assert_eq!(detail.body, "hello");
    assert!(detail.files.iter().any(|f| f == "SKILL.md"));
    assert!(detail.files.iter().any(|f| f == "references/README.md"));
}

#[test]
fn load_missing_is_not_found() {
    let (s, _u, _p) = svc();
    assert!(matches!(load(&s, "nope", None), Err(Error::NotFound(_))));
}

#[test]
fn write_frontmatter_preserves_body() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "k", "name: k", "body line\nsecond\n");
    let fm = serde_json::json!({ "name": "k", "description": "new" });
    write_frontmatter(&s, "k", None, &fm).unwrap();
    let detail = load(&s, "k", None).unwrap();
    assert_eq!(detail.frontmatter["description"], "new");
    assert_eq!(detail.body, "body line\nsecond\n");
}

#[test]
fn write_body_preserves_frontmatter() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "k", "name: k\ndescription: orig", "old");
    write_body(&s, "k", None, "totally new body").unwrap();
    let detail = load(&s, "k", None).unwrap();
    assert_eq!(detail.frontmatter["description"], "orig");
    assert_eq!(detail.body, "totally new body");
}

#[test]
fn write_file_and_delete_file_roundtrip() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "k", "name: k", "x");
    write_file(&s, "k", None, "assets/vars.json", b"{}").unwrap();
    let detail = load(&s, "k", None).unwrap();
    assert!(detail.files.iter().any(|f| f == "assets/vars.json"));
    delete_file(&s, "k", None, "assets/vars.json").unwrap();
    let detail2 = load(&s, "k", None).unwrap();
    assert!(!detail2.files.iter().any(|f| f == "assets/vars.json"));
}

#[test]
fn delete_file_rejects_skill_md() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "k", "name: k", "x");
    let r = delete_file(&s, "k", None, "SKILL.md");
    assert!(matches!(r, Err(Error::BadPath(_))));
}

#[test]
fn file_path_rejects_dotdot() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "k", "name: k", "x");
    let r = write_file(&s, "k", None, "../escape.md", b"no");
    assert!(matches!(r, Err(Error::BadPath(_))));
}

#[test]
fn create_refuses_collision() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "dup", "name: dup", "b");
    let r = create(&s, "dup", Scope::Project, &serde_json::json!({}), "");
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
}

#[test]
fn delete_trashes_dir() {
    let (s, _u, p) = svc();
    seed(&s, Scope::Project, "doomed", "name: doomed", "b");
    delete(&s, "doomed", None).unwrap();
    assert!(matches!(load(&s, "doomed", None), Err(Error::NotFound(_))));
    let entries: Vec<String> = std::fs::read_dir(p.path().join(".trash"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(entries.iter().any(|n| n.ends_with("-doomed")), "trash: {entries:?}");
}

#[test]
fn rename_copies_tree_and_trashes_source() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "old", "name: old", "body");
    s.write(
        Scope::Project,
        Path::new("skills/old/references/r.md"),
        b"r",
    )
    .unwrap();
    rename(&s, "old", "shiny", None).unwrap();
    assert!(matches!(load(&s, "old", None), Err(Error::NotFound(_))));
    let detail = load(&s, "shiny", None).unwrap();
    assert!(detail.files.iter().any(|f| f == "references/r.md"));
}

#[test]
fn duplicate_creates_copy_same_scope() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "a", "name: a", "orig");
    duplicate(&s, "a", "b", None).unwrap();
    let a = load(&s, "a", None).unwrap();
    let b = load(&s, "b", None).unwrap();
    assert_eq!(a.body, b.body);
}

#[test]
fn move_scope_transfers_tree() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "m", "name: m", "x");
    move_scope(&s, "m", Scope::Project, Scope::User).unwrap();
    assert!(matches!(
        load(&s, "m", Some(Scope::Project)),
        Err(Error::NotFound(_))
    ));
    let d = load(&s, "m", None).unwrap();
    assert_eq!(d.scope, ScopeTag::User);
}
