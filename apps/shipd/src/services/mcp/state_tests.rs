//! McpService state tests.

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

fn seed_raw(s: &FilesService, scope: Scope, body: &str) {
    s.write(scope, Path::new("mcp.jsonc"), body.as_bytes()).unwrap();
}

fn seed_server(s: &FilesService, scope: Scope, id: &str) {
    let doc = serde_json::json!({
        "mcp": { "servers": { id: {
            "name": id, "command": "/bin/true", "server_type": "stdio"
        } } }
    });
    s.write_jsonc(scope, Path::new("mcp.jsonc"), &doc).unwrap();
}

#[test]
fn list_empty_when_no_file() {
    let (s, _u, _p) = svc();
    assert!(list(&s).is_empty());
}

#[test]
fn list_enumerates_entries_with_scope() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::Project, r#"{"mcp":{"servers":{"a":{"name":"a","command":"x"},"b":{"name":"b","command":"y"}}}}"#);
    seed_raw(&s, Scope::User, r#"{"mcp":{"servers":{"c":{"name":"c","command":"z"}}}}"#);
    let items = list(&s);
    let ids: Vec<&str> = items.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b", "c"]);
    let a = items.iter().find(|e| e.id == "a").unwrap();
    assert_eq!(a.scope, ScopeTag::Project);
    let c = items.iter().find(|e| e.id == "c").unwrap();
    assert_eq!(c.scope, ScopeTag::User);
}

#[test]
fn list_dedupes_project_over_user() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::User, r#"{"mcp":{"servers":{"a":{"name":"user-a","command":"x"}}}}"#);
    seed_raw(&s, Scope::Project, r#"{"mcp":{"servers":{"a":{"name":"proj-a","command":"y"}}}}"#);
    let items = list(&s);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].scope, ScopeTag::Project);
}

#[test]
fn load_entry_default_prefers_project() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::User, r#"{"mcp":{"servers":{"a":{"name":"user","command":"x"}}}}"#);
    seed_raw(&s, Scope::Project, r#"{"mcp":{"servers":{"a":{"name":"proj","command":"y"}}}}"#);
    let (scope, entry) = load_entry(&s, "a", None).unwrap();
    assert_eq!(scope, Scope::Project);
    assert_eq!(entry["name"], "proj");
}

#[test]
fn load_entry_forced_scope() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::User, r#"{"mcp":{"servers":{"a":{"name":"user","command":"x"}}}}"#);
    seed_raw(&s, Scope::Project, r#"{"mcp":{"servers":{"a":{"name":"proj","command":"y"}}}}"#);
    let (_, entry) = load_entry(&s, "a", Some(Scope::User)).unwrap();
    assert_eq!(entry["name"], "user");
}

#[test]
fn write_creates_then_refuses_overwrite() {
    let (s, _u, _p) = svc();
    seed_server(&s, Scope::Project, "a");
    let new = serde_json::json!({"name":"a2","command":"/bin/false"});
    let r = write_entry(&s, "a", Scope::Project, &new, false);
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
}

#[test]
fn write_allow_overwrite_replaces_entry() {
    let (s, _u, _p) = svc();
    seed_server(&s, Scope::Project, "a");
    let new = serde_json::json!({"name":"a2","command":"/bin/false"});
    write_entry(&s, "a", Scope::Project, &new, true).unwrap();
    let (_, e) = load_entry(&s, "a", None).unwrap();
    assert_eq!(e["name"], "a2");
}

#[test]
fn write_seeds_empty_file() {
    let (s, _u, _p) = svc();
    let new = serde_json::json!({"name":"a","command":"/bin/x"});
    write_entry(&s, "a", Scope::Project, &new, false).unwrap();
    let (_, e) = load_entry(&s, "a", None).unwrap();
    assert_eq!(e["command"], "/bin/x");
}

#[test]
fn patch_applies_to_sub_entry() {
    let (s, _u, _p) = svc();
    seed_server(&s, Scope::Project, "a");
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "replace", "path": "/command", "value": "/bin/new" },
    ])).unwrap();
    patch_entry(&s, "a", None, &patch).unwrap();
    let (_, e) = load_entry(&s, "a", None).unwrap();
    assert_eq!(e["command"], "/bin/new");
}

#[test]
fn patch_creates_missing_intermediates() {
    let (s, _u, _p) = svc();
    seed_server(&s, Scope::Project, "a");
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "add", "path": "/env/FOO", "value": "bar" },
    ])).unwrap();
    patch_entry(&s, "a", None, &patch).unwrap();
    let (_, e) = load_entry(&s, "a", None).unwrap();
    assert_eq!(e["env"]["FOO"], "bar");
}

#[test]
fn delete_removes_only_that_entry() {
    let (s, _u, _p) = svc();
    seed_raw(&s, Scope::Project,
        r#"{"mcp":{"servers":{"a":{"name":"a","command":"x"},"b":{"name":"b","command":"y"}}}}"#);
    delete_entry(&s, "a", None).unwrap();
    let ids: Vec<String> = list(&s).into_iter().map(|e| e.id).collect();
    assert_eq!(ids, vec!["b".to_string()]);
}

#[test]
fn missing_returns_not_found() {
    let (s, _u, _p) = svc();
    seed_server(&s, Scope::Project, "a");
    assert!(matches!(load_entry(&s, "ghost", None), Err(Error::NotFound(_))));
    assert!(matches!(delete_entry(&s, "ghost", None), Err(Error::NotFound(_))));
}

#[test]
fn bad_id_rejected() {
    let (s, _u, _p) = svc();
    assert!(matches!(load_entry(&s, "", None), Err(Error::BadId(_))));
    assert!(matches!(load_entry(&s, "bad id", None), Err(Error::BadId(_))));
}
