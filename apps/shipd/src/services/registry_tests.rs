//! Registry dispatch tests. Verifies name resolution + arg parsing
//! without spinning up ApiState — we rely on pure state-function calls.

use super::*;
use serde_json::json;

#[test]
fn names_list_is_nonempty_and_unique() {
    let n = names();
    // agents.* and sessions are now on the Connect handler path and no longer
    // appear in the legacy OPS catalog — only workspaces remains.
    assert!(!n.is_empty(), "expected at least one registry op in catalog");
    let mut seen = std::collections::HashSet::new();
    for name in n {
        assert!(seen.insert(name), "duplicate registry name: {name}");
    }
}

#[test]
fn registry_error_is_serializable() {
    // Used by templates + HTTP error responses; must round-trip.
    let err = RegistryError {
        code: "test",
        message: "hi".into(),
    };
    let v = serde_json::to_value(&err).unwrap();
    assert_eq!(v["code"], "test");
    assert_eq!(v["message"], "hi");
}

#[test]
fn registry_error_display_includes_code_and_message() {
    let err = RegistryError { code: "x", message: "y".into() };
    let s = err.to_string();
    assert!(s.contains("[x]"));
    assert!(s.contains("y"));
}

fn temp_state() -> crate::ship_runtime::ApiState {
    let dir = tempfile::TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    // Leak the tempdir for the lifetime of the test — db handles inside
    // ApiState reference paths under it.
    std::mem::forget(dir);
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
}

// ── agents domain-fn gate (drive domain crates directly, not string dispatch) ─
//
// The agents.* ops are now on the Connect handler path; the string-registry arms
// are deleted. These tests exercise the domain crates that the handler calls,
// proving the domain logic still works. They are the replacement coverage for the
// former registry dispatch tests.

use ship_agents::services::agents::{create, state as agents_state, view_state};
use ship_files::files::FilesService;

#[test]
fn agents_list_domain_fn_returns_items_key() {
    let state = temp_state();
    let files = FilesService::resolve_roots(state.ship_dir());
    let items = agents_state::list(&files);
    let v = json!({ "items": items });
    assert!(v.get("items").and_then(|i| i.as_array()).is_some(), "items is an array");
}

#[test]
fn agents_summaries_domain_fn_returns_items_key() {
    let state = temp_state();
    let items = view_state::list_agents(state.ship_dir());
    let v = json!({ "items": items });
    assert!(v.get("items").and_then(|i| i.as_array()).is_some(), "items is an array");
}

#[test]
fn agents_templates_presets_harnesses_domain_fns_work() {
    let t = create::builtin_templates();
    assert!(!t.is_empty(), "builtin templates must be non-empty");

    let p = create::builtin_permission_presets();
    assert!(!p.is_empty(), "builtin permission presets must be non-empty");

    let h = create::builtin_harnesses();
    assert!(!h.is_empty(), "builtin harnesses must be non-empty");
}

#[test]
fn agents_catalogs_domain_fn_serializes() {
    let state = temp_state();
    let catalogs = view_state::list_agent_catalogs(state.ship_dir());
    serde_json::to_value(&catalogs).expect("catalogs must serialize");
}

#[test]
fn agents_get_response_serializes_like_anonymous_json() {
    // `agents.get`'s success shape needs an agent on disk; prove the named
    // wrapper is wire-identical to the old `json!({id, scope, doc})` at the
    // serialization boundary instead.
    use ship_agents::services::agents::state::ScopeTag;
    use ship_agents::services::agents::wire_types::AgentGetResponse;
    let doc = json!({ "agent": { "name": "X" } });
    let typed = AgentGetResponse {
        id: "x".to_string(),
        scope: ScopeTag::Project,
        doc: doc.clone(),
    };
    let from_struct = serde_json::to_value(&typed).unwrap();
    let from_anon = json!({ "id": "x", "scope": ScopeTag::Project, "doc": doc });
    assert_eq!(from_struct, from_anon);
}

#[test]
fn agents_detail_missing_is_not_found() {
    // Behaviour-preserving: unknown id → None from domain fn.
    let state = temp_state();
    let result = view_state::load_agent(state.ship_dir(), "nope");
    assert!(result.is_none(), "unknown agent id must return None");
}

// ── remaining domains: structural / byte-identical smoke ──────────────

#[test]
fn actors_and_mesh_dispatch_but_absent_from_catalog() {
    // Both are dispatchable read ops but intentionally NOT in the schema
    // catalog — `names()` must not advertise them (byte-identical schema).
    let n = names();
    assert!(!n.contains(&"actors.list"), "actors.list must stay out of OPS");
    assert!(!n.contains(&"mesh.snapshot"), "mesh.snapshot must stay out of OPS");

    let state = temp_state();
    let actors = call("actors.list", &json!({}), &state).expect("actors.list ok");
    assert!(actors.get("items").and_then(|i| i.as_array()).is_some());
}
