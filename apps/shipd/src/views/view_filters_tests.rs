use super::*;
use minijinja::Value;

fn make_item(id: &str, data: serde_json::Value) -> Value {
    Value::from_serialize(&serde_json::json!({
        "id": id,
        "data": data,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    }))
}

#[test]
fn where_equality_filter() {
    let items = vec![
        make_item("1", serde_json::json!({"status": "active"})),
        make_item("2", serde_json::json!({"status": "done"})),
        make_item("3", serde_json::json!({"status": "active"})),
    ];
    let result = filter_where(items, "status".into(), Value::from("active"), None);
    assert_eq!(result.len(), 2);
}

#[test]
fn where_comparison_gte() {
    let items = vec![
        make_item("1", serde_json::json!({"priority": 1})),
        make_item("2", serde_json::json!({"priority": 3})),
        make_item("3", serde_json::json!({"priority": 5})),
    ];
    let result = filter_where(
        items,
        "priority".into(),
        Value::from(">="),
        Some(Value::from(3)),
    );
    assert_eq!(result.len(), 2);
}

#[test]
fn where_comparison_lt() {
    let items = vec![
        make_item("1", serde_json::json!({"priority": 1})),
        make_item("2", serde_json::json!({"priority": 5})),
    ];
    let result = filter_where(
        items,
        "priority".into(),
        Value::from("<"),
        Some(Value::from(3)),
    );
    assert_eq!(result.len(), 1);
}

#[test]
fn where_missing_field_excluded() {
    let items = vec![
        make_item("1", serde_json::json!({"status": "active"})),
        make_item("2", serde_json::json!({"title": "no status"})),
    ];
    let result = filter_where(items, "status".into(), Value::from("active"), None);
    assert_eq!(result.len(), 1);
}

#[test]
fn sort_by_numeric() {
    let items = vec![
        make_item("a", serde_json::json!({"priority": 3})),
        make_item("b", serde_json::json!({"priority": 1})),
        make_item("c", serde_json::json!({"priority": 2})),
    ];
    let sorted = filter_sort_by(items, "priority".into(), None);
    let ids: Vec<_> = sorted
        .iter()
        .map(|v| v.get_attr("id").unwrap().to_string())
        .collect();
    assert_eq!(ids, vec!["b", "c", "a"]);
}

#[test]
fn sort_by_reverse() {
    let items = vec![
        make_item("a", serde_json::json!({"priority": 1})),
        make_item("b", serde_json::json!({"priority": 3})),
    ];
    let sorted = filter_sort_by(items, "priority".into(), Some(true));
    let first_id = sorted[0].get_attr("id").unwrap().to_string();
    assert_eq!(first_id, "b");
}

#[test]
fn take_limits_items() {
    let items = vec![
        make_item("1", serde_json::json!({})),
        make_item("2", serde_json::json!({})),
        make_item("3", serde_json::json!({})),
    ];
    let result = filter_take(items, 2);
    assert_eq!(result.len(), 2);
}

#[test]
fn take_more_than_available() {
    let items = vec![make_item("1", serde_json::json!({}))];
    let result = filter_take(items, 10);
    assert_eq!(result.len(), 1);
}

#[test]
fn ship_items_nonexistent_returns_empty() {
    let result = ship_items("nonexistent_collection_xyz".into());
    assert!(result.is_empty());
}

#[test]
fn where_ne_operator() {
    let items = vec![
        make_item("1", serde_json::json!({"status": "active"})),
        make_item("2", serde_json::json!({"status": "done"})),
    ];
    let result = filter_where(
        items,
        "status".into(),
        Value::from("!="),
        Some(Value::from("done")),
    );
    assert_eq!(result.len(), 1);
}

#[test]
fn ship_query_nonexistent_returns_empty() {
    let result = ship_query("nonexistent_collection_xyz".into(), None);
    assert!(result.is_empty());
}

#[test]
fn build_query_request_parses_opts() {
    use super::build_query_request;
    let opts = Value::from_serialize(&serde_json::json!({
        "status": "active",
        "priority__gte": 3,
        "_sort": "-priority,created_at",
        "_limit": 20,
        "_offset": 5,
    }));
    let req = build_query_request("task", &Some(opts));
    assert_eq!(req.collection, "task");
    assert_eq!(req.limit, 20);
    assert_eq!(req.offset, 5);
    assert_eq!(req.filter.get("status").unwrap(), "active");
    assert_eq!(req.filter.get("priority__gte").unwrap(), 3);
    assert_eq!(req.sort.get("priority"), Some(&-1));
    assert_eq!(req.sort.get("created_at"), Some(&1));
}

#[test]
fn build_query_request_defaults_without_opts() {
    use super::build_query_request;
    let req = build_query_request("task", &None);
    assert_eq!(req.collection, "task");
    assert_eq!(req.limit, 200);
    assert_eq!(req.offset, 0);
    assert!(req.filter.is_empty());
    assert!(req.sort.is_empty());
}

