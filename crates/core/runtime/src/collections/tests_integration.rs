//! Integration tests — full CRUD + query + schema migration + access control
//! against real SQLite. These test the system as a user would experience it.

#[cfg(test)]
mod tests {
    use crate::collections::query::QueryRequest;
    use crate::collections::schema::{self, CollectionSchema};
    use crate::collections::store::{
        self, create_item, delete_item, ensure_collection_table, get_item,
        open_docs_db, query_items, update_item,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    // ── Test helpers ─────────────────────────────────────────────────────

    fn task_schema() -> CollectionSchema {
        schema::parse_schema(r#"{
            "id": "task",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["todo", "doing", "done"] },
                "priority": { "type": "number" },
                "tags": { "type": "array" }
            }
        }"#).unwrap()
    }

    fn setup() -> (crate::data::DataService, CollectionSchema, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = open_docs_db(&ship_dir).unwrap();
        let schema = task_schema();
        ensure_collection_table(&mut svc, &schema).unwrap();
        (svc, schema, tmp)
    }

    fn seed(svc: &mut crate::data::DataService, schema: &CollectionSchema) {
        create_item(svc, schema, "t1", &json!({"title": "Write tests", "status": "todo", "priority": 1}), Some("alice")).unwrap();
        create_item(svc, schema, "t2", &json!({"title": "Fix bug", "status": "doing", "priority": 3}), Some("bob")).unwrap();
        create_item(svc, schema, "t3", &json!({"title": "Ship release", "status": "done", "priority": 2}), None).unwrap();
        create_item(svc, schema, "t4", &json!({"title": "Update docs", "status": "todo", "priority": 1}), Some("alice")).unwrap();
    }

    fn make_query(collection: &str, filter: serde_json::Value) -> QueryRequest {
        QueryRequest {
            collection: collection.to_string(),
            filter: serde_json::from_value(filter).unwrap(),
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        }
    }

    // ── CRUD ─────────────────────────────────────────────────────────────

    #[test]
    fn create_item_and_read_back() {
        let (mut svc, schema, _tmp) = setup();
        let item = create_item(&mut svc, &schema, "new-1",
            &json!({"title": "Hello", "status": "todo"}), Some("me")).unwrap();

        assert_eq!(item.id, "new-1");
        assert_eq!(item.data["title"], "Hello");
        assert_eq!(item.created_by.as_deref(), Some("me"));

        let fetched = get_item(&mut svc, &schema, "new-1").unwrap().unwrap();
        assert_eq!(fetched.data["title"], "Hello");
        assert_eq!(fetched.data["status"], "todo");
    }

    #[test]
    fn create_rejects_missing_required_field() {
        let (mut svc, schema, _tmp) = setup();
        let err = create_item(&mut svc, &schema, "bad", &json!({"status": "todo"}), None)
            .unwrap_err();
        assert!(err.to_string().contains("'title' is required"), "got: {err}");
    }

    #[test]
    fn create_rejects_invalid_enum_value() {
        let (mut svc, schema, _tmp) = setup();
        let err = create_item(&mut svc, &schema, "bad",
            &json!({"title": "X", "status": "invalid"}), None).unwrap_err();
        assert!(err.to_string().contains("must be one of"), "got: {err}");
    }

    #[test]
    fn create_rejects_wrong_field_type() {
        let (mut svc, schema, _tmp) = setup();
        let err = create_item(&mut svc, &schema, "bad",
            &json!({"title": 42}), None).unwrap_err();
        assert!(err.to_string().contains("must be a string"), "got: {err}");
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let (mut svc, schema, _tmp) = setup();
        assert!(get_item(&mut svc, &schema, "nope").unwrap().is_none());
    }

    #[test]
    fn update_merges_fields_preserving_untouched() {
        let (mut svc, schema, _tmp) = setup();
        create_item(&mut svc, &schema, "u1",
            &json!({"title": "Original", "status": "todo", "priority": 5}), None).unwrap();

        let updated = update_item(&mut svc, &schema, "u1",
            &json!({"status": "done"})).unwrap();

        assert_eq!(updated.data["title"], "Original", "untouched field preserved");
        assert_eq!(updated.data["status"], "done", "updated field changed");
    }

    #[test]
    fn update_validates_after_merge() {
        let (mut svc, schema, _tmp) = setup();
        create_item(&mut svc, &schema, "u2",
            &json!({"title": "Test", "status": "todo"}), None).unwrap();

        let err = update_item(&mut svc, &schema, "u2",
            &json!({"status": "invalid"})).unwrap_err();
        assert!(err.to_string().contains("must be one of"), "got: {err}");
    }

    #[test]
    fn update_nonexistent_item_errors() {
        let (mut svc, schema, _tmp) = setup();
        let err = update_item(&mut svc, &schema, "ghost",
            &json!({"title": "X"})).unwrap_err();
        assert!(err.to_string().contains("not found"), "got: {err}");
    }

    #[test]
    fn delete_removes_item() {
        let (mut svc, schema, _tmp) = setup();
        create_item(&mut svc, &schema, "d1", &json!({"title": "Delete me"}), None).unwrap();

        assert!(delete_item(&mut svc, "task", "d1").unwrap());
        assert!(get_item(&mut svc, &schema, "d1").unwrap().is_none());
    }

    #[test]
    fn delete_nonexistent_returns_false() {
        let (mut svc, _, _tmp) = setup();
        assert!(!delete_item(&mut svc, "task", "ghost").unwrap());
    }

    // ── Queries ──────────────────────────────────────────────────────────

    #[test]
    fn query_all_items_in_collection() {
        let (mut svc, schema, _tmp) = setup();
        seed(&mut svc, &schema);

        let results = query_items(&mut svc, &schema, &make_query("task", json!({}))).unwrap();
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn query_filter_by_equality() {
        let (mut svc, schema, _tmp) = setup();
        seed(&mut svc, &schema);

        let results = query_items(&mut svc, &schema,
            &make_query("task", json!({"status": "todo"}))).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.data["status"] == "todo"));
    }

    #[test]
    fn query_filter_by_gte() {
        let (mut svc, schema, _tmp) = setup();
        seed(&mut svc, &schema);

        let results = query_items(&mut svc, &schema,
            &make_query("task", json!({"priority__gte": 2}))).unwrap();
        assert_eq!(results.len(), 2, "priority 2 and 3");
    }

    #[test]
    fn query_filter_by_in() {
        let (mut svc, schema, _tmp) = setup();
        seed(&mut svc, &schema);

        let results = query_items(&mut svc, &schema,
            &make_query("task", json!({"status__in": ["todo", "doing"]}))).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn query_with_sort_descending() {
        let (mut svc, schema, _tmp) = setup();
        seed(&mut svc, &schema);

        let mut req = make_query("task", json!({}));
        req.sort.insert("priority".to_string(), -1);
        let results = query_items(&mut svc, &schema, &req).unwrap();
        // Numbers round-trip as f64 through REAL columns.
        assert_eq!(results[0].data["priority"], json!(3.0));
    }

    #[test]
    fn query_with_limit_and_offset() {
        let (mut svc, schema, _tmp) = setup();
        seed(&mut svc, &schema);

        let mut req = make_query("task", json!({}));
        req.limit = 2;
        req.offset = 1;
        let results = query_items(&mut svc, &schema, &req).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_empty_collection_returns_empty() {
        let (mut svc, schema, _tmp) = setup();
        let results = query_items(&mut svc, &schema, &make_query("task", json!({}))).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn query_nonexistent_collection_returns_empty() {
        let (mut svc, _, _tmp) = setup();
        let dummy = schema::parse_schema(
            r#"{"id": "nonexistent", "fields": {"x": {"type": "string"}}}"#
        ).unwrap();
        let results = query_items(&mut svc, &dummy, &make_query("nonexistent", json!({}))).unwrap();
        assert!(results.is_empty());
    }

    // ── Schema migration ─────────────────────────────────────────────────

    #[test]
    fn adding_a_field_to_schema_migrates_without_data_loss() {
        let (mut svc, schema, _tmp) = setup();
        create_item(&mut svc, &schema, "m1",
            &json!({"title": "Before migration"}), None).unwrap();

        // Schema v2 adds an "assignee" field.
        let schema_v2 = schema::parse_schema(r#"{
            "id": "task",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["todo", "doing", "done"] },
                "priority": { "type": "number" },
                "tags": { "type": "array" },
                "assignee": { "type": "string" }
            }
        }"#).unwrap();
        ensure_collection_table(&mut svc, &schema_v2).unwrap();

        // Old item readable with null for new field.
        let item = get_item(&mut svc, &schema_v2, "m1").unwrap().unwrap();
        assert_eq!(item.data["title"], "Before migration");
        assert!(item.data.get("assignee").is_none() || item.data["assignee"].is_null());

        // New field works for new items.
        create_item(&mut svc, &schema_v2, "m2",
            &json!({"title": "After migration", "assignee": "alice"}), None).unwrap();
        let item2 = get_item(&mut svc, &schema_v2, "m2").unwrap().unwrap();
        assert_eq!(item2.data["assignee"], "alice");
    }

    // ── Full lifecycle ───────────────────────────────────────────────────

    #[test]
    fn full_crud_lifecycle() {
        let (mut svc, schema, _tmp) = setup();

        // Create
        let item = create_item(&mut svc, &schema, "life-1",
            &json!({"title": "Lifecycle test", "status": "todo"}), Some("tester")).unwrap();
        assert_eq!(item.data["status"], "todo");

        // Read
        let fetched = get_item(&mut svc, &schema, "life-1").unwrap().unwrap();
        assert_eq!(fetched.data["title"], "Lifecycle test");

        // Update
        let updated = update_item(&mut svc, &schema, "life-1",
            &json!({"status": "done"})).unwrap();
        assert_eq!(updated.data["status"], "done");
        assert_eq!(updated.data["title"], "Lifecycle test");

        // Query
        let results = query_items(&mut svc, &schema,
            &make_query("task", json!({"status": "done"}))).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "life-1");

        // Delete
        assert!(delete_item(&mut svc, "task", "life-1").unwrap());
        assert!(get_item(&mut svc, &schema, "life-1").unwrap().is_none());

        // Verify deletion sticks
        let results = query_items(&mut svc, &schema,
            &make_query("task", json!({"status": "done"}))).unwrap();
        assert!(results.is_empty());
    }

    // ── Schema validation ────────────────────────────────────────────────

    #[test]
    fn schema_rejects_empty_id() {
        let err = schema::parse_schema(r#"{"id": "", "fields": {"x": {"type": "string"}}}"#)
            .unwrap_err();
        assert!(err.to_string().contains("id is required"));
    }

    #[test]
    fn schema_rejects_no_fields() {
        let err = schema::parse_schema(r#"{"id": "empty", "fields": {}}"#).unwrap_err();
        assert!(err.to_string().contains("at least one field"));
    }

    #[test]
    fn schema_rejects_enum_without_values() {
        let err = schema::parse_schema(r#"{"id": "bad", "fields": {"x": {"type": "enum"}}}"#)
            .unwrap_err();
        assert!(err.to_string().contains("must have 'values'"));
    }

    #[test]
    fn schema_loads_from_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        let schemas_dir = ship_dir.join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        std::fs::write(
            schemas_dir.join("note.jsonc"),
            r#"{"id": "note", "fields": {"body": {"type": "markdown", "required": true}}}"#,
        ).unwrap();

        let schemas = schema::load_schemas(&ship_dir).unwrap();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].id, "note");
    }

    #[test]
    fn schema_load_skips_missing_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let schemas = schema::load_schemas(&tmp.path().join(".ship")).unwrap();
        assert!(schemas.is_empty());
    }

    // ── Multiple collections ─────────────────────────────────────────────

    #[test]
    fn multiple_collections_are_isolated() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = open_docs_db(&ship_dir).unwrap();

        let tasks = schema::parse_schema(r#"{
            "id": "task",
            "fields": {"title": {"type": "string", "required": true}}
        }"#).unwrap();
        let notes = schema::parse_schema(r#"{
            "id": "note",
            "fields": {"body": {"type": "markdown", "required": true}}
        }"#).unwrap();

        ensure_collection_table(&mut svc, &tasks).unwrap();
        ensure_collection_table(&mut svc, &notes).unwrap();

        create_item(&mut svc, &tasks, "t1", &json!({"title": "Task 1"}), None).unwrap();
        create_item(&mut svc, &notes, "n1", &json!({"body": "Note 1"}), None).unwrap();

        let task_results = query_items(&mut svc, &tasks,
            &make_query("task", json!({}))).unwrap();
        let note_results = query_items(&mut svc, &notes,
            &make_query("note", json!({}))).unwrap();

        assert_eq!(task_results.len(), 1);
        assert_eq!(note_results.len(), 1);
        assert_eq!(task_results[0].data["title"], "Task 1");
        assert_eq!(note_results[0].data["body"], "Note 1");
    }

    // ── SQL injection prevention ─────────────────────────────────────────

    #[test]
    fn query_rejects_unsafe_field_names() {
        let (mut svc, schema, _tmp) = setup();
        seed(&mut svc, &schema);

        // Malicious field name in filter should be skipped, not injected.
        let results = query_items(&mut svc, &schema, &QueryRequest {
            collection: "task".to_string(),
            filter: {
                let mut f = BTreeMap::new();
                f.insert("'; DROP TABLE collection_task; --".to_string(), json!("x"));
                f
            },
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        }).unwrap();

        // Should return all items (unsafe filter was skipped, not executed).
        assert_eq!(results.len(), 4);
    }
}
