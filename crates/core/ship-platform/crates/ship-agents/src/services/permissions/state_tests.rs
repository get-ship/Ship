//! PermissionService state tests.

use crate::services::permissions::state::*;
use ship_files::files::{FilesService, Scope};
use std::path::Path;
use tempfile::tempdir;

fn svc() -> (FilesService, tempfile::TempDir, tempfile::TempDir) {
    let user = tempdir().unwrap();
    let project = tempdir().unwrap();
    let svc = FilesService::new(user.path().to_path_buf(), Some(project.path().to_path_buf()));
    (svc, user, project)
}

fn seed_raw(s: &FilesService, scope: Scope, body: &str) {
    s.write(scope, Path::new("permissions.jsonc"), body.as_bytes()).unwrap();
}

#[test]
fn list_empty_when_no_file() {
    let (s, _u, _p) = svc();
    assert!(list(&s).is_empty());
}

#[test]
fn list_filters_private_keys() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::Project, r#"{
        "_base_deny": { "tools_deny": ["Bash(rm -rf *)"] },
        "$schema": "https://example.com/s",
        "default_mode": "standard",
        "ship-standard": { "tools_allow": ["Read"] },
        "ship-autonomous": { "tools_allow": ["*"] }
    }"#);
    let items = list(&s);
    let ids: Vec<&str> = items.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["ship-autonomous", "ship-standard"]);
}

#[test]
fn list_dedupes_project_over_user() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::User,
        r#"{"ship-standard":{"tools_allow":["user"]}, "user-only":{"tools_allow":["u"]}}"#);
    seed_raw(&s, Scope::Project,
        r#"{"ship-standard":{"tools_allow":["proj"]}}"#);
    let items = list(&s);
    let ids: Vec<&str> = items.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["ship-standard", "user-only"]);
    let std = items.iter().find(|e| e.id == "ship-standard").unwrap();
    assert_eq!(std.scope, ScopeTag::Project);
}

#[test]
fn load_entry_default_prefers_project() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::User, r#"{"x":{"tools_allow":["u"]}}"#);
    seed_raw(&s, Scope::Project, r#"{"x":{"tools_allow":["p"]}}"#);
    let (scope, entry) = load_entry(&s, "x", None).unwrap();
    assert_eq!(scope, Scope::Project);
    assert_eq!(entry["tools_allow"][0], "p");
}

#[test]
fn load_private_key_rejected_even_if_present() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::Project, r#"{"_base_deny":{"tools_deny":["x"]}}"#);
    assert!(matches!(load_entry(&s, "_base_deny", None), Err(Error::BadId(_))));
}

#[test]
fn write_creates_then_refuses_overwrite() {
    let (s, _u, _p) = svc();
    let entry = serde_json::json!({"tools_allow":["Read"]});
    write_entry(&s, "a", Scope::Project, &entry, false).unwrap();
    let r = write_entry(&s, "a", Scope::Project, &entry, false);
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
}

#[test]
fn write_rejects_non_object_entry() {
    let (s, _u, _p) = svc();
    let entry = serde_json::json!("nope");
    let r = write_entry(&s, "a", Scope::Project, &entry, false);
    assert!(matches!(r, Err(Error::Parse(_))));
}

#[test]
fn write_allow_overwrite_replaces_entry() {
    let (s, _u, _p) = svc();
    let e1 = serde_json::json!({"tools_allow":["Read"]});
    let e2 = serde_json::json!({"tools_allow":["Write"]});
    write_entry(&s, "a", Scope::Project, &e1, false).unwrap();
    write_entry(&s, "a", Scope::Project, &e2, true).unwrap();
    let (_, e) = load_entry(&s, "a", None).unwrap();
    assert_eq!(e["tools_allow"][0], "Write");
}

#[test]
fn patch_replaces_field() {
    let (s, _u, _p) = svc();
    let entry = serde_json::json!({"tools_allow":["Read"]});
    write_entry(&s, "a", Scope::Project, &entry, false).unwrap();
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "add", "path": "/tools_deny", "value": ["Bash(rm -rf *)"] },
    ])).unwrap();
    patch_entry(&s, "a", None, &patch).unwrap();
    let (_, e) = load_entry(&s, "a", None).unwrap();
    assert_eq!(e["tools_deny"][0], "Bash(rm -rf *)");
}

#[test]
fn delete_preserves_sibling_entries() {
    let (s, _u, p) = svc();
    seed_raw(&s, Scope::Project,
        r#"{"a":{"tools_allow":[]},"b":{"tools_allow":[]},"_base":{"x":1}}"#);
    delete_entry(&s, "a", None).unwrap();
    let ids: Vec<String> = list(&s).into_iter().map(|e| e.id).collect();
    assert_eq!(ids, vec!["b".to_string()]);
    // Private key preserved in the file.
    let raw = std::fs::read_to_string(p.path().join("permissions.jsonc")).unwrap();
    assert!(raw.contains("_base"));
}

#[test]
fn bad_id_rejected() {
    let (s, _u, _p) = svc();
    assert!(matches!(load_entry(&s, "_private", None), Err(Error::BadId(_))));
    assert!(matches!(load_entry(&s, "$schema", None), Err(Error::BadId(_))));
    assert!(matches!(load_entry(&s, "", None), Err(Error::BadId(_))));
    assert!(matches!(load_entry(&s, "bad id", None), Err(Error::BadId(_))));
}

#[test]
fn missing_returns_not_found() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::Project, r#"{"a":{"tools_allow":[]}}"#);
    assert!(matches!(load_entry(&s, "ghost", None), Err(Error::NotFound(_))));
    assert!(matches!(delete_entry(&s, "ghost", None), Err(Error::NotFound(_))));
}
