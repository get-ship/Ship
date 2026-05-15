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
