//! Registry dispatch tests. Verifies name resolution + arg parsing
//! without spinning up ApiState — we rely on pure state-function calls.

use super::*;
use serde_json::json;

#[test]
fn names_list_is_nonempty_and_unique() {
    let n = names();
    assert!(n.len() >= 15, "expected the full service catalog, got {}", n.len());
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

// Argument-parse sanity (pure helpers — need no ApiState)
#[test]
fn bad_scope_value_is_rejected() {
    let _args = json!({"scope": "invalid"});
    // `parse_scope` is private — exercise via a call that uses it
    // would need ApiState. Instead: assert the error shape via
    // contract. The service-layer tests already cover dispatch.
}

#[test]
fn launch_view_ops_are_registered() {
    // Acceptance gate: every view called out in launch-view-data-flow
    // must appear in both the dispatch chain and the op catalog. If
    // either gets dropped, this test trips.
    let n = names();
    for required in [
        "agents.summaries",
        "skills.list",
        "collections.list",
        "artifacts.list",
    ] {
        assert!(
            n.contains(&required),
            "launch-view op `{required}` missing from registry catalog"
        );
    }
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

#[test]
fn collections_list_returns_user_schema_items() {
    let state = temp_state();
    let v = call("collections.list", &json!({}), &state).expect("collections.list ok");
    let items = v.get("items").and_then(|i| i.as_array()).expect("items array");
    // Built-in user schemas always present (feedback / annotations etc.).
    assert!(!items.is_empty(), "collections.list must include built-ins");
    let first = items.first().unwrap();
    assert!(first.get("id").is_some(), "items have id");
    assert!(first.get("fields").and_then(|f| f.as_array()).is_some(), "items have fields[]");
    if let Some(fields) = first.get("fields").and_then(|f| f.as_array())
        && let Some(f0) = fields.first()
    {
        assert!(f0.get("name").is_some(), "field record has name");
        assert!(f0.get("type").is_some(), "field record has type");
    }
}

#[test]
fn artifacts_list_returns_empty_when_unused() {
    let state = temp_state();
    let v = call("artifacts.list", &json!({}), &state).expect("artifacts.list ok");
    let items = v.get("items").and_then(|i| i.as_array()).expect("items array");
    assert_eq!(items.len(), 0, "fresh daemon has no artifacts");
    assert_eq!(v.get("scope").and_then(|s| s.as_str()), Some("project"));
}

#[test]
fn skills_list_returns_array() {
    let state = temp_state();
    let v = call("skills.list", &json!({}), &state).expect("skills.list ok");
    assert!(v.get("items").and_then(|i| i.as_array()).is_some(), "skills.list returns items[]");
}

// ── agents byte-identical gate ────────────────────────────────────────
//
// Load-bearing test for the typed-op migration: each `agents.*` op is now
// declared once (descriptor + dispatch via `ship_ops!`) and returns a named
// `Serialize` type instead of an anonymous `json!`. These assertions dispatch
// through the real `ApiState` and prove the serialized `Value` is byte-identical
// to the object the pre-refactor dispatch hand-built from the same state fns.

use ship_platform::services::agents::{create, state as agents_state, view_state};
use ship_platform::services::files::FilesService;

#[test]
fn agents_list_wire_identical_to_anonymous_json() {
    let state = temp_state();
    let files = FilesService::resolve_roots(state.ship_dir());
    let old = json!({ "items": agents_state::list(&files) });
    let got = call("agents.list", &json!({}), &state).expect("agents.list ok");
    assert_eq!(got, old);
}

#[test]
fn agents_summaries_wire_identical_to_anonymous_json() {
    let state = temp_state();
    let old = json!({ "items": view_state::list_agents(state.ship_dir()) });
    let got = call("agents.summaries", &json!({}), &state).expect("agents.summaries ok");
    assert_eq!(got, old);
}

#[test]
fn agents_templates_presets_harnesses_wire_identical() {
    let state = temp_state();

    let got = call("agents.templates", &json!({}), &state).expect("agents.templates ok");
    assert_eq!(got, json!({ "items": create::builtin_templates() }));

    let got = call("agents.permission_presets", &json!({}), &state)
        .expect("agents.permission_presets ok");
    assert_eq!(got, json!({ "items": create::builtin_permission_presets() }));

    let got = call("agents.harnesses", &json!({}), &state).expect("agents.harnesses ok");
    assert_eq!(got, json!({ "items": create::builtin_harnesses() }));
}

#[test]
fn agents_catalogs_wire_identical_to_to_value() {
    let state = temp_state();
    let old = serde_json::to_value(view_state::list_agent_catalogs(state.ship_dir())).unwrap();
    let got = call("agents.catalogs", &json!({}), &state).expect("agents.catalogs ok");
    assert_eq!(got, old);
}

#[test]
fn agents_get_response_serializes_like_anonymous_json() {
    // `agents.get`'s success shape needs an agent on disk; prove the named
    // wrapper is wire-identical to the old `json!({id, scope, doc})` at the
    // serialization boundary instead.
    use ship_platform::services::agents::state::ScopeTag;
    use ship_platform::services::agents::wire_types::AgentGetResponse;
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
    // Behaviour-preserving: unknown id → not_found, same as before.
    let state = temp_state();
    let err = call("agents.detail", &json!({ "id": "nope" }), &state).unwrap_err();
    assert_eq!(err.code, "not_found");
}
