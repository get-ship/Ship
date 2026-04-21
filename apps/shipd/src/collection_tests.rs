//! End-to-end tests for the collections REST API.
//!
//! Uses TestDaemon — real kernel, real SQLite, real HTTP handlers.
//! No mocks. Tests the full path from HTTP request to database to response.

#[cfg(test)]
mod tests {
    use crate::test_harness::TestDaemon;
    use serde_json::json;

    /// Write a schema JSONC file into the test daemon's .ship/collections/ dir.
    fn write_schema(daemon: &TestDaemon, id: &str, schema_json: &str) {
        let schemas_dir = daemon.temp_dir.path().join(".ship").join("collections");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        std::fs::write(schemas_dir.join(format!("{id}.jsonc")), schema_json).unwrap();
    }

    fn task_schema_json() -> &'static str {
        r#"{
            "id": "task",
            "name": "Task",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["todo", "doing", "done"] },
                "priority": { "type": "number" },
                "description": { "type": "markdown" },
                "done": { "type": "boolean" },
                "attachment": { "type": "file" },
                "tags": { "type": "array" }
            }
        }"#
    }

    // ── List collections ─────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn list_collections_includes_builtins() {
        let daemon = TestDaemon::new().await;
        let resp = daemon.get("/api/collections").await;
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["ok"], true);
        // 4 built-in collections: specs, adrs, notes, feedback
        let data = resp.body["data"].as_array().unwrap();
        assert_eq!(data.len(), 4);
        let ids: Vec<&str> = data.iter().map(|s| s["id"].as_str().unwrap()).collect();
        assert!(ids.contains(&"specs"));
        assert!(ids.contains(&"adrs"));
        assert!(ids.contains(&"notes"));
        assert!(ids.contains(&"feedback"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_collections_with_user_schema() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        let resp = daemon.get("/api/collections").await;
        assert_eq!(resp.status, 200);
        let data = resp.body["data"].as_array().unwrap();
        // 4 built-in + 1 user-defined
        assert_eq!(data.len(), 5);
        assert!(data.iter().any(|s| s["id"] == "task" && s["field_count"] == 7));
    }

    // ── Get schema ───────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn get_schema_returns_fields() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        let resp = daemon.get("/api/collections/task/schema").await;
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["data"]["id"], "task");
        assert!(resp.body["data"]["fields"]["title"].is_object());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_schema_not_found() {
        let daemon = TestDaemon::new().await;
        let resp = daemon.get("/api/collections/nonexistent/schema").await;
        assert_eq!(resp.status, 404);
    }

    // ── Create item ──────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn create_item_returns_201() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        let resp = daemon.post_json("/api/collections/task/items", &json!({
            "id": "t1",
            "data": { "title": "Write tests", "status": "todo", "priority": 1 }
        })).await;

        assert_eq!(resp.status, 201, "body: {}", resp.body);
        assert_eq!(resp.body["ok"], true);
        assert_eq!(resp.body["data"]["id"], "t1");
        assert_eq!(resp.body["data"]["data"]["title"], "Write tests");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_item_validation_error() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        // Missing required field "title".
        let resp = daemon.post_json("/api/collections/task/items", &json!({
            "data": { "status": "todo" }
        })).await;

        assert_eq!(resp.status, 400);
        assert!(resp.body["error"].as_str().unwrap().contains("title"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_item_invalid_enum() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        let resp = daemon.post_json("/api/collections/task/items", &json!({
            "data": { "title": "Test", "status": "invalid" }
        })).await;

        assert_eq!(resp.status, 400);
        assert!(resp.body["error"].as_str().unwrap().contains("must be one of"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_generates_id_when_omitted() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        let resp = daemon.post_json("/api/collections/task/items", &json!({
            "data": { "title": "Auto ID" }
        })).await;

        assert_eq!(resp.status, 201);
        let id = resp.body["data"]["id"].as_str().unwrap();
        assert!(!id.is_empty(), "auto-generated ID should not be empty");
    }

    // ── Get item ─────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn get_item_after_create() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        daemon.post_json("/api/collections/task/items", &json!({
            "id": "g1",
            "data": { "title": "Fetch me", "status": "doing" }
        })).await;

        let resp = daemon.get("/api/collections/task/items/g1").await;
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["data"]["data"]["title"], "Fetch me");
        assert_eq!(resp.body["data"]["data"]["status"], "doing");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_item_not_found() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        // Trigger table creation by any request first.
        daemon.post_json("/api/collections/task/items", &json!({
            "id": "x", "data": { "title": "Seed" }
        })).await;

        let resp = daemon.get("/api/collections/task/items/nonexistent").await;
        assert_eq!(resp.status, 404);
    }

    // ── Update item ──────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn update_merges_fields() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        daemon.post_json("/api/collections/task/items", &json!({
            "id": "u1",
            "data": { "title": "Original", "status": "todo", "priority": 5 }
        })).await;

        let resp = daemon.put_json("/api/collections/task/items/u1", &json!({
            "data": { "status": "done" }
        })).await;

        assert_eq!(resp.status, 200, "body: {}", resp.body);
        assert_eq!(resp.body["data"]["data"]["title"], "Original", "untouched field preserved");
        assert_eq!(resp.body["data"]["data"]["status"], "done", "updated field changed");
    }

    // ── Delete item ──────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_item_removes_it() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        daemon.post_json("/api/collections/task/items", &json!({
            "id": "d1", "data": { "title": "Delete me" }
        })).await;

        let resp = daemon.delete("/api/collections/task/items/d1").await;
        assert_eq!(resp.status, 200);

        let check = daemon.get("/api/collections/task/items/d1").await;
        assert_eq!(check.status, 404);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_nonexistent_returns_404() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        // Trigger table creation.
        daemon.post_json("/api/collections/task/items", &json!({
            "id": "x", "data": { "title": "Seed" }
        })).await;

        let resp = daemon.delete("/api/collections/task/items/ghost").await;
        assert_eq!(resp.status, 404);
    }

    // ── Query ────────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn query_returns_all_items() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        for i in 1..=3 {
            daemon.post_json("/api/collections/task/items", &json!({
                "id": format!("q{i}"),
                "data": { "title": format!("Task {i}"), "status": "todo" }
            })).await;
        }

        let resp = daemon.post_json("/api/collections/task/query", &json!({
            "collection": "task"
        })).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["count"], 3);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn query_with_filter() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        daemon.post_json("/api/collections/task/items", &json!({
            "id": "f1", "data": { "title": "A", "status": "todo" }
        })).await;
        daemon.post_json("/api/collections/task/items", &json!({
            "id": "f2", "data": { "title": "B", "status": "done" }
        })).await;

        let resp = daemon.post_json("/api/collections/task/query", &json!({
            "collection": "task",
            "filter": { "status": "done" }
        })).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["count"], 1);
        assert_eq!(resp.body["data"][0]["data"]["title"], "B");
    }

    // ── File reference field ─────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn file_reference_round_trips() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        daemon.post_json("/api/collections/task/items", &json!({
            "id": "file1",
            "data": {
                "title": "With attachment",
                "attachment": ".artifacts/design.png"
            }
        })).await;

        let resp = daemon.get("/api/collections/task/items/file1").await;
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["data"]["data"]["attachment"], ".artifacts/design.png");
    }

    // ── Boolean field ────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn boolean_field_round_trips() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        daemon.post_json("/api/collections/task/items", &json!({
            "id": "bool1",
            "data": { "title": "Done task", "done": true }
        })).await;

        let resp = daemon.get("/api/collections/task/items/bool1").await;
        assert_eq!(resp.status, 200);
        // SQLite stores booleans as integers (0/1). true may round-trip as 1.
        let done = &resp.body["data"]["data"]["done"];
        assert!(done == true || done == 1, "expected true or 1, got {done}");
    }

    // ── Multiple collections ─────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn multiple_collections_isolated() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());
        write_schema(&daemon, "note", r#"{
            "id": "note",
            "fields": { "body": { "type": "markdown", "required": true } }
        }"#);

        daemon.post_json("/api/collections/task/items", &json!({
            "id": "t1", "data": { "title": "Task" }
        })).await;
        daemon.post_json("/api/collections/note/items", &json!({
            "id": "n1", "data": { "body": "Note content" }
        })).await;

        let tasks = daemon.post_json("/api/collections/task/query", &json!({
            "collection": "task"
        })).await;
        let notes = daemon.post_json("/api/collections/note/query", &json!({
            "collection": "note"
        })).await;

        assert_eq!(tasks.body["count"], 1);
        assert_eq!(notes.body["count"], 1);
    }

    // ── Full lifecycle ───────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn full_crud_lifecycle_via_rest() {
        let daemon = TestDaemon::new().await;
        write_schema(&daemon, "task", task_schema_json());

        // Create
        let create_resp = daemon.post_json("/api/collections/task/items", &json!({
            "id": "lifecycle",
            "data": { "title": "E2E Test", "status": "todo", "priority": 3 }
        })).await;
        assert_eq!(create_resp.status, 201);

        // Read
        let get_resp = daemon.get("/api/collections/task/items/lifecycle").await;
        assert_eq!(get_resp.body["data"]["data"]["title"], "E2E Test");

        // Update
        let update_resp = daemon.put_json("/api/collections/task/items/lifecycle", &json!({
            "data": { "status": "done" }
        })).await;
        assert_eq!(update_resp.status, 200);
        assert_eq!(update_resp.body["data"]["data"]["status"], "done");
        assert_eq!(update_resp.body["data"]["data"]["title"], "E2E Test");

        // Query
        let query_resp = daemon.post_json("/api/collections/task/query", &json!({
            "collection": "task",
            "filter": { "status": "done" }
        })).await;
        assert_eq!(query_resp.body["count"], 1);

        // Delete
        let delete_resp = daemon.delete("/api/collections/task/items/lifecycle").await;
        assert_eq!(delete_resp.status, 200);

        // Verify gone
        let gone_resp = daemon.get("/api/collections/task/items/lifecycle").await;
        assert_eq!(gone_resp.status, 404);

        // Verify query empty
        let empty_resp = daemon.post_json("/api/collections/task/query", &json!({
            "collection": "task",
            "filter": { "status": "done" }
        })).await;
        assert_eq!(empty_resp.body["count"], 0);
    }
}
