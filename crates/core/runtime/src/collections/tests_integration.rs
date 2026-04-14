//! Integration tests — store + query working together against real SQLite.

#[cfg(test)]
mod tests {
    use crate::collections::query::QueryRequest;
    use crate::collections::schema::parse_schema;
    use crate::collections::store::{
        create_item, delete_item, get_item, open_docs_db, query_items, update_item,
        ensure_indexes,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    fn setup() -> (sqlx::SqliteConnection, crate::collections::schema::CollectionSchema) {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let ship_dir = Box::leak(Box::new(ship_dir));
        let conn = open_docs_db(ship_dir).unwrap();
        let schema = parse_schema(r#"{
            "id": "task",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["todo", "doing", "done"] },
                "priority": { "type": "number" },
                "tags": { "type": "array" }
            }
        }"#).unwrap();
        (conn, schema)
    }

    fn seed(conn: &mut sqlx::SqliteConnection, schema: &crate::collections::schema::CollectionSchema) {
        create_item(conn, schema, "t1", &json!({"title": "Write tests", "status": "todo", "priority": 1}), Some("alice")).unwrap();
        create_item(conn, schema, "t2", &json!({"title": "Fix bug", "status": "doing", "priority": 3}), Some("bob")).unwrap();
        create_item(conn, schema, "t3", &json!({"title": "Ship release", "status": "done", "priority": 2}), None).unwrap();
        create_item(conn, schema, "t4", &json!({"title": "Update docs", "status": "todo", "priority": 1}), Some("alice")).unwrap();
    }

    #[test]
    fn query_all_items() {
        let (mut conn, schema) = setup();
        seed(&mut conn, &schema);

        let req = QueryRequest {
            collection: "task".to_string(),
            filter: BTreeMap::new(),
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn query_by_equality() {
        let (mut conn, schema) = setup();
        seed(&mut conn, &schema);

        let mut filter = BTreeMap::new();
        filter.insert("status".to_string(), json!("todo"));
        let req = QueryRequest {
            collection: "task".to_string(),
            filter,
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.data["status"] == "todo"));
    }

    #[test]
    fn query_by_gte() {
        let (mut conn, schema) = setup();
        seed(&mut conn, &schema);

        let mut filter = BTreeMap::new();
        filter.insert("priority".to_string(), json!({"$gte": 2}));
        let req = QueryRequest {
            collection: "task".to_string(),
            filter,
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert_eq!(results.len(), 2); // priority 2 and 3
    }

    #[test]
    fn query_by_in() {
        let (mut conn, schema) = setup();
        seed(&mut conn, &schema);

        let mut filter = BTreeMap::new();
        filter.insert("status".to_string(), json!({"$in": ["todo", "doing"]}));
        let req = QueryRequest {
            collection: "task".to_string(),
            filter,
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn query_with_sort() {
        let (mut conn, schema) = setup();
        seed(&mut conn, &schema);

        let mut sort = BTreeMap::new();
        sort.insert("priority".to_string(), -1i8); // desc
        let req = QueryRequest {
            collection: "task".to_string(),
            filter: BTreeMap::new(),
            sort,
            limit: 50,
            offset: 0,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert_eq!(results[0].data["priority"], 3);
    }

    #[test]
    fn query_with_limit_and_offset() {
        let (mut conn, schema) = setup();
        seed(&mut conn, &schema);

        let req = QueryRequest {
            collection: "task".to_string(),
            filter: BTreeMap::new(),
            sort: BTreeMap::new(),
            limit: 2,
            offset: 1,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_empty_collection() {
        let (mut conn, _) = setup();

        let req = QueryRequest {
            collection: "task".to_string(),
            filter: BTreeMap::new(),
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn query_wrong_collection() {
        let (mut conn, schema) = setup();
        seed(&mut conn, &schema);

        let req = QueryRequest {
            collection: "nonexistent".to_string(),
            filter: BTreeMap::new(),
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        };
        let results = query_items(&mut conn, &req).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn full_crud_lifecycle() {
        let (mut conn, schema) = setup();

        // Create
        let item = create_item(&mut conn, &schema, "lifecycle-1",
            &json!({"title": "Lifecycle test", "status": "todo"}), Some("tester")).unwrap();
        assert_eq!(item.data["status"], "todo");

        // Read
        let fetched = get_item(&mut conn, "task", "lifecycle-1").unwrap().unwrap();
        assert_eq!(fetched.data["title"], "Lifecycle test");

        // Update
        let updated = update_item(&mut conn, &schema, "lifecycle-1",
            &json!({"status": "done"})).unwrap();
        assert_eq!(updated.data["status"], "done");
        assert_eq!(updated.data["title"], "Lifecycle test"); // preserved

        // Query
        let mut filter = BTreeMap::new();
        filter.insert("status".to_string(), json!("done"));
        let results = query_items(&mut conn, &QueryRequest {
            collection: "task".to_string(),
            filter,
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        }).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "lifecycle-1");

        // Delete
        assert!(delete_item(&mut conn, "task", "lifecycle-1").unwrap());
        assert!(get_item(&mut conn, "task", "lifecycle-1").unwrap().is_none());
    }

    #[test]
    fn indexes_improve_queries() {
        let (mut conn, schema) = setup();
        ensure_indexes(&mut conn, &schema).unwrap();
        seed(&mut conn, &schema);

        // Query should still work with indexes
        let mut filter = BTreeMap::new();
        filter.insert("status".to_string(), json!("done"));
        let results = query_items(&mut conn, &QueryRequest {
            collection: "task".to_string(),
            filter,
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        }).unwrap();
        assert_eq!(results.len(), 1);
    }
}
