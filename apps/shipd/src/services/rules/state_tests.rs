//! RuleService state tests.

use super::*;
use ship_platform::services::files::{FilesService, Scope};
use std::path::Path;
use tempfile::tempdir;

fn svc() -> (FilesService, tempfile::TempDir, tempfile::TempDir) {
    let user = tempdir().unwrap();
    let project = tempdir().unwrap();
    let svc = FilesService::new(user.path().to_path_buf(), Some(project.path().to_path_buf()));
    (svc, user, project)
}

fn seed(s: &FilesService, scope: Scope, id: &str, body: &str) {
    s.write(scope, Path::new(&format!("rules/{id}.md")), body.as_bytes()).unwrap();
}

#[test]
fn list_empty_when_no_rules() {
    let (s, _u, _p) = svc();
    assert!(list(&s).is_empty());
}

#[test]
fn list_dedupes_project_over_user() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "a", "# A user");
    seed(&s, Scope::Project, "a", "# A proj");
    seed(&s, Scope::User, "user-only", "# Uo");
    let items = list(&s);
    let ids: Vec<&str> = items.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "user-only"]);
    let a = items.iter().find(|e| e.id == "a").unwrap();
    assert_eq!(a.scope, ScopeTag::Project);
}

#[test]
fn list_skips_readme_and_non_md() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "real", "# real");
    s.write(Scope::Project, Path::new("rules/README.md"), b"nope").unwrap();
    s.write(Scope::Project, Path::new("rules/notes.txt"), b"nope").unwrap();
    let items = list(&s);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "real");
}

#[test]
fn load_body_default_prefers_project() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "x", "user");
    seed(&s, Scope::Project, "x", "proj");
    let (scope, body) = load_body(&s, "x", None).unwrap();
    assert_eq!(scope, Scope::Project);
    assert_eq!(body, "proj");
}

#[test]
fn load_body_forced_user_scope() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "x", "user");
    seed(&s, Scope::Project, "x", "proj");
    let (_, body) = load_body(&s, "x", Some(Scope::User)).unwrap();
    assert_eq!(body, "user");
}

#[test]
fn bad_id_rejected() {
    let (s, _u, _p) = svc();
    assert!(matches!(load_body(&s, "BadId", None), Err(Error::BadId(_))));
    assert!(matches!(load_body(&s, "", None), Err(Error::BadId(_))));
    assert!(matches!(load_body(&s, "-x", None), Err(Error::BadId(_))));
    assert!(matches!(load_body(&s, "x-", None), Err(Error::BadId(_))));
}

#[test]
fn write_creates_then_refuses_overwrite_without_flag() {
    let (s, _u, _p) = svc();
    write_body(&s, "r", Scope::Project, "# hi", false).unwrap();
    let r = write_body(&s, "r", Scope::Project, "# bye", false);
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
}

#[test]
fn write_allow_overwrite_replaces_body() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "r", "# old");
    write_body(&s, "r", Scope::Project, "# new", true).unwrap();
    let (_, body) = load_body(&s, "r", None).unwrap();
    assert_eq!(body, "# new");
}

#[test]
fn missing_returns_not_found() {
    let (s, _u, _p) = svc();
    assert!(matches!(load_body(&s, "ghost", None), Err(Error::NotFound(_))));
}

#[test]
fn delete_trashes_not_removes() {
    let (s, _u, p) = svc();
    seed(&s, Scope::Project, "doomed", "# x");
    delete_body(&s, "doomed", None).unwrap();
    assert!(load_body(&s, "doomed", None).is_err());
    let trashed: Vec<String> = std::fs::read_dir(p.path().join(".trash"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(trashed.iter().any(|n| n.ends_with("-doomed.md")), "trash: {trashed:?}");
}

#[test]
fn delete_forced_user_scope() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "r", "# u");
    seed(&s, Scope::Project, "r", "# p");
    delete_body(&s, "r", Some(Scope::User)).unwrap();
    // Project copy still resolves.
    let (scope, body) = load_body(&s, "r", None).unwrap();
    assert_eq!(scope, Scope::Project);
    assert_eq!(body, "# p");
}
