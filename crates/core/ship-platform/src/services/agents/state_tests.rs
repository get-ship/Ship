//! AgentsService state tests. Pure logic, def-backed via temp dirs.
//!
//! Every read/write rides the same FsBroker door the service uses (S5b) — the
//! tests seed through `write_doc` rather than a raw fs write, so they exercise
//! exactly the one-door path. Tempdir roots are canonicalized: on WSL/macOS
//! `/tmp` is a symlink and the broker's escape guard canonicalizes the deepest
//! existing ancestor before comparing it to the cap root.

use crate::services::agents::state::*;
use crate::services::files::{FilesService, Scope};
use std::path::PathBuf;
use tempfile::tempdir;

fn svc() -> (FilesService, PathBuf, PathBuf, tempfile::TempDir, tempfile::TempDir) {
    let user_tmp = tempdir().unwrap();
    let project_tmp = tempdir().unwrap();
    let user = std::fs::canonicalize(user_tmp.path()).unwrap();
    let project = std::fs::canonicalize(project_tmp.path()).unwrap();
    let svc = FilesService::new(user.clone(), Some(project.clone()));
    (svc, user, project, user_tmp, project_tmp)
}

fn seed(s: &FilesService, scope: Scope, id: &str, body: &str) {
    let doc: serde_json::Value = serde_json::from_str(body).unwrap();
    write_doc(s, id, scope, &doc, true).unwrap();
}

fn agent_doc(id: &str) -> String {
    format!(r#"{{"agent":{{"id":"{id}","name":"{id}"}},"skills":{{"refs":[]}}}}"#)
}

#[test]
fn list_dedupes_project_over_user() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::User, "dev", &agent_doc("dev"));
    seed(&s, Scope::Project, "dev", &agent_doc("dev"));
    seed(&s, Scope::User, "user-only", &agent_doc("user-only"));

    let items = list(&s);
    let ids: Vec<&str> = items.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, vec!["dev", "user-only"]);
    let dev_scope = items.iter().find(|e| e.id == "dev").unwrap().scope;
    assert_eq!(dev_scope, ScopeTag::Project);
}

#[test]
fn load_doc_default_prefers_project() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::User, "x", r#"{"agent":{"id":"x","name":"user"}}"#);
    seed(&s, Scope::Project, "x", r#"{"agent":{"id":"x","name":"proj"}}"#);

    let (scope, doc) = load_doc(&s, "x", None).unwrap();
    assert_eq!(scope, Scope::Project);
    assert_eq!(doc["agent"]["name"], "proj");
}

#[test]
fn load_doc_forced_user_scope() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::User, "x", r#"{"agent":{"id":"x","name":"user"}}"#);
    seed(&s, Scope::Project, "x", r#"{"agent":{"id":"x","name":"proj"}}"#);
    let (_, doc) = load_doc(&s, "x", Some(Scope::User)).unwrap();
    assert_eq!(doc["agent"]["name"], "user");
}

#[test]
fn bad_id_rejected() {
    let (s, _u, _p, _ut, _pt) = svc();
    let r = load_doc(&s, "BadID", None);
    assert!(matches!(r, Err(Error::BadId(_))));
}

#[test]
fn patch_applies_and_persists() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "pat", &agent_doc("pat"));
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "add", "path": "/skills/refs/-", "value": "tdd" },
    ])).unwrap();
    patch_doc(&s, "pat", None, &patch).unwrap();
    let (_, doc) = load_doc(&s, "pat", None).unwrap();
    assert_eq!(doc["skills"]["refs"], serde_json::json!(["tdd"]));
}

#[test]
fn patch_creates_missing_intermediate_objects() {
    // Doc has no `mcp` key at all; single `add` op should succeed.
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "p", r#"{"agent":{"id":"p","name":"p"}}"#);
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "add", "path": "/mcp/servers/-", "value": "ship" },
    ])).unwrap();
    patch_doc(&s, "p", None, &patch).unwrap();
    let (_, doc) = load_doc(&s, "p", None).unwrap();
    assert_eq!(doc["mcp"]["servers"], serde_json::json!(["ship"]));
}

#[test]
fn patch_replace_at_top_level_works_without_pre_existing_key() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "p", r#"{"agent":{"id":"p","name":"p"}}"#);
    // `add` at `/permissions` replaces-or-creates at root level — no
    // intermediate segments needed.
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "add", "path": "/permissions",
          "value": { "preset": "ship-standard", "tools_allow": ["Read"] } },
    ])).unwrap();
    patch_doc(&s, "p", None, &patch).unwrap();
    let (_, doc) = load_doc(&s, "p", None).unwrap();
    assert_eq!(doc["permissions"]["preset"], "ship-standard");
}

#[test]
fn rename_preserves_content_and_updates_agent_id() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "old-id", r#"{"agent":{"id":"old-id","name":"x"},"skills":{"refs":["a"]}}"#);
    rename(&s, "old-id", "new-id", None).unwrap();
    assert!(load_doc(&s, "old-id", None).is_err());
    let (_, doc) = load_doc(&s, "new-id", None).unwrap();
    assert_eq!(doc["agent"]["id"], "new-id");
    assert_eq!(doc["skills"]["refs"], serde_json::json!(["a"]));
}

#[test]
fn rename_refuses_collision() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "a", &agent_doc("a"));
    seed(&s, Scope::Project, "b", &agent_doc("b"));
    let r = rename(&s, "a", "b", None);
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
}

#[test]
fn duplicate_creates_copy() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "a", &agent_doc("a"));
    duplicate(&s, "a", "a-copy", None).unwrap();
    let (_, original) = load_doc(&s, "a", None).unwrap();
    let (_, copy) = load_doc(&s, "a-copy", None).unwrap();
    assert_eq!(original["agent"]["id"], "a");
    assert_eq!(copy["agent"]["id"], "a-copy");
}

#[test]
fn move_scope_transfers() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "m", &agent_doc("m"));
    move_scope(&s, "m", Scope::Project, Scope::User).unwrap();
    assert!(load_doc(&s, "m", Some(Scope::Project)).is_err());
    let (scope, _) = load_doc(&s, "m", None).unwrap();
    assert_eq!(scope, Scope::User);
}

#[test]
fn delete_trashes_not_removes() {
    let (s, _u, p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "doomed", &agent_doc("doomed"));
    delete_doc(&s, "doomed", None).unwrap();
    assert!(load_doc(&s, "doomed", None).is_err());
    let trashed: Vec<String> = std::fs::read_dir(p.join(".trash"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(trashed.iter().any(|n| n.ends_with("-doomed.jsonc")), "trash: {trashed:?}");
}

#[test]
fn write_refuses_overwrite_when_not_allowed() {
    let (s, _u, _p, _ut, _pt) = svc();
    seed(&s, Scope::Project, "a", &agent_doc("a"));
    let doc: serde_json::Value = serde_json::from_str(&agent_doc("a")).unwrap();
    let r = write_doc(&s, "a", Scope::Project, &doc, false);
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
}
