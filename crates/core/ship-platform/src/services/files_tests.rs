//! FilesService tests — path scoping, atomic writes, trash.

use crate::services::files::*;
use std::path::Path;
use tempfile::tempdir;

fn svc() -> (FilesService, tempfile::TempDir, tempfile::TempDir) {
    let user = tempdir().unwrap();
    let project = tempdir().unwrap();
    let svc = FilesService::new(user.path().to_path_buf(), Some(project.path().to_path_buf()));
    (svc, user, project)
}

#[test]
fn write_then_read_roundtrips() {
    let (s, _u, _p) = svc();
    s.write(Scope::Project, Path::new("agents/x.jsonc"), b"hello").unwrap();
    let got = s.read(Scope::Project, Path::new("agents/x.jsonc")).unwrap();
    assert_eq!(got, b"hello");
}

#[test]
fn rejects_absolute_path() {
    let (s, _u, _p) = svc();
    let r = s.resolve(Scope::Project, Path::new("/etc/passwd"));
    assert!(matches!(r, Err(Error::PathEscape(_))));
}

#[test]
fn rejects_parent_escape() {
    let (s, _u, _p) = svc();
    let r = s.resolve(Scope::Project, Path::new("../outside.txt"));
    assert!(matches!(r, Err(Error::PathEscape(_))), "got {r:?}");
}

#[test]
fn create_refuses_overwrite() {
    let (s, _u, _p) = svc();
    s.write(Scope::Project, Path::new("a.txt"), b"first").unwrap();
    let r = s.create(Scope::Project, Path::new("a.txt"), b"second");
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
    // Original content preserved.
    assert_eq!(s.read(Scope::Project, Path::new("a.txt")).unwrap(), b"first");
}

#[test]
fn trash_moves_not_deletes() {
    let (s, _u, p) = svc();
    s.write(Scope::Project, Path::new("x.jsonc"), b"doomed").unwrap();
    s.trash(Scope::Project, Path::new("x.jsonc")).unwrap();
    // Original gone.
    assert!(!s.exists(Scope::Project, Path::new("x.jsonc")));
    // Something with suffix `-x.jsonc` in .trash/.
    let trash = p.path().join(".trash");
    let entries: Vec<_> = std::fs::read_dir(&trash)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(entries.iter().any(|n| n.ends_with("-x.jsonc")), "trash: {entries:?}");
}

#[test]
fn read_jsonc_strips_comments() {
    let (s, _u, _p) = svc();
    let raw = r#"{
        // comment
        "a": 1,
        /* block */
        "b": 2,
    }"#;
    s.write(Scope::Project, Path::new("c.jsonc"), raw.as_bytes()).unwrap();
    let v: serde_json::Value = s.read_jsonc(Scope::Project, Path::new("c.jsonc")).unwrap();
    assert_eq!(v["a"], 1);
    assert_eq!(v["b"], 2);
}

#[test]
fn patch_jsonc_applies_and_writes() {
    let (s, _u, _p) = svc();
    s.write(Scope::Project, Path::new("d.jsonc"),
        br#"{ "name": "old", "tags": ["a"] }"#).unwrap();

    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "replace", "path": "/name", "value": "new" },
        { "op": "add", "path": "/tags/-", "value": "b" },
    ])).unwrap();

    s.patch_jsonc(Scope::Project, Path::new("d.jsonc"), &patch).unwrap();

    let v: serde_json::Value = s.read_jsonc(Scope::Project, Path::new("d.jsonc")).unwrap();
    assert_eq!(v["name"], "new");
    assert_eq!(v["tags"], serde_json::json!(["a", "b"]));
}

#[test]
fn project_scope_unavailable_when_not_configured() {
    let user = tempdir().unwrap();
    let s = FilesService::new(user.path().to_path_buf(), None);
    let r = s.read(Scope::Project, Path::new("x.txt"));
    assert!(matches!(r, Err(Error::ScopeUnavailable(Scope::Project))));
}

#[test]
fn write_creates_parent_dirs() {
    let (s, _u, _p) = svc();
    s.write(Scope::Project, Path::new("a/b/c/d.txt"), b"nested").unwrap();
    assert_eq!(s.read(Scope::Project, Path::new("a/b/c/d.txt")).unwrap(), b"nested");
}

#[test]
fn atomic_write_leaves_no_tmp_on_success() {
    let (s, _u, p) = svc();
    s.write(Scope::Project, Path::new("a.jsonc"), b"x").unwrap();
    let leftover = std::fs::read_dir(p.path()).unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tmp"))
        .collect::<Vec<_>>();
    assert!(leftover.is_empty(), "tmp files left behind: {leftover:?}");
}
