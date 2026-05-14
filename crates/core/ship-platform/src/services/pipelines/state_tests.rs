//! PipelineService state tests. Pure logic over temp dirs.

use crate::services::pipelines::state::*;
use crate::services::files::{FilesService, Scope};
use std::path::Path;
use tempfile::tempdir;

fn svc() -> (FilesService, tempfile::TempDir, tempfile::TempDir) {
    let user = tempdir().unwrap();
    let project = tempdir().unwrap();
    let svc = FilesService::new(user.path().to_path_buf(), Some(project.path().to_path_buf()));
    (svc, user, project)
}

fn seed(s: &FilesService, scope: Scope, name: &str, ext: &str, body: &str) {
    s.write(scope, Path::new(&format!("pipelines/{name}.{ext}")), body.as_bytes()).unwrap();
}

fn pipeline_doc(name: &str) -> String {
    format!(
        r#"{{"name":"{name}","description":"test","steps":[{{"id":"s1","run":"echo hi"}}]}}"#
    )
}

#[test]
fn list_dedupes_project_over_user() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "ci", "jsonc", &pipeline_doc("ci"));
    seed(&s, Scope::Project, "ci", "jsonc", &pipeline_doc("ci"));
    seed(&s, Scope::User, "user-only", "jsonc", &pipeline_doc("user-only"));

    let items = list(&s);
    let names: Vec<&str> = items.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["ci", "user-only"]);
    let ci_scope = items.iter().find(|e| e.name == "ci").unwrap().scope;
    assert_eq!(ci_scope, ScopeTag::Project);
}

#[test]
fn list_ignores_non_jsonc_files() {
    let (s, _u, _p) = svc();
    // YAML pipelines are no longer supported; they're filtered out by list.
    seed(&s, Scope::Project, "legacy", "yaml",
         "name: legacy\nsteps:\n  - id: a\n    run: echo\n");
    let items = list(&s);
    assert!(items.is_empty());
}

#[test]
fn load_doc_default_prefers_project() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "x", "jsonc", r#"{"name":"x","description":"user","steps":[]}"#);
    seed(&s, Scope::Project, "x", "jsonc", r#"{"name":"x","description":"proj","steps":[]}"#);
    let (scope, _fmt, doc) = load_doc(&s, "x", None).unwrap();
    assert_eq!(scope, Scope::Project);
    assert_eq!(doc["description"], "proj");
}

#[test]
fn yaml_pipelines_are_unreadable() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "yml", "yaml",
         "name: yml\nsteps:\n  - id: a\n    run: echo\n");
    // The legacy .yaml file is on disk but the loader doesn't see it —
    // pipelines are jsonc-only now.
    assert!(load_doc(&s, "yml", None).is_err());
}

#[test]
fn bad_name_rejected() {
    let (s, _u, _p) = svc();
    let r = load_doc(&s, "BadName", None);
    assert!(matches!(r, Err(Error::BadId(_))));
}

#[test]
fn patch_applies_and_persists() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "p", "jsonc", &pipeline_doc("p"));
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "replace", "path": "/description", "value": "changed" },
    ])).unwrap();
    patch_doc(&s, "p", None, &patch).unwrap();
    let (_, _, doc) = load_doc(&s, "p", None).unwrap();
    assert_eq!(doc["description"], "changed");
}

#[test]
fn patch_creates_missing_intermediate_objects() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "p", "jsonc", r#"{"name":"p","steps":[]}"#);
    let patch: json_patch::Patch = serde_json::from_value(serde_json::json!([
        { "op": "add", "path": "/vars/foo", "value": "bar" },
    ])).unwrap();
    patch_doc(&s, "p", None, &patch).unwrap();
    let (_, _, doc) = load_doc(&s, "p", None).unwrap();
    assert_eq!(doc["vars"]["foo"], "bar");
}

#[test]
fn delete_trashes_not_removes() {
    let (s, _u, p) = svc();
    seed(&s, Scope::Project, "doomed", "jsonc", &pipeline_doc("doomed"));
    delete_doc(&s, "doomed", None).unwrap();
    assert!(load_doc(&s, "doomed", None).is_err());
    let trashed: Vec<String> = std::fs::read_dir(p.path().join(".trash"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(trashed.iter().any(|n| n.ends_with("-doomed.jsonc")), "trash: {trashed:?}");
}

#[test]
fn write_refuses_overwrite_when_not_allowed() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::Project, "a", "jsonc", &pipeline_doc("a"));
    let doc: serde_json::Value = serde_json::from_str(&pipeline_doc("a")).unwrap();
    let r = write_doc(&s, "a", Scope::Project, &doc, false);
    assert!(matches!(r, Err(Error::AlreadyExists(_))));
}

#[test]
fn load_doc_forced_user_scope() {
    let (s, _u, _p) = svc();
    seed(&s, Scope::User, "x", "jsonc", r#"{"name":"x","description":"user","steps":[]}"#);
    seed(&s, Scope::Project, "x", "jsonc", r#"{"name":"x","description":"proj","steps":[]}"#);
    let (_, _, doc) = load_doc(&s, "x", Some(Scope::User)).unwrap();
    assert_eq!(doc["description"], "user");
}
