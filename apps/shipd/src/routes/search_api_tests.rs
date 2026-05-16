//! HTTP-level tests for `/api/v1/search`. The TestDaemon spins a real
//! kernel + SQLite, so we exercise the adapter snapshotting + the
//! aggregator end-to-end rather than mocking either.

#[cfg(test)]
mod tests {
    use crate::test_harness::TestDaemon;
    use serde_json::json;

    fn write_schema(daemon: &TestDaemon, id: &str, schema_json: &str) {
        let schemas_dir = ship_substrate::ShipHome::project_ship_dir(daemon.temp_dir.path())
            .join("collections");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        std::fs::write(schemas_dir.join(format!("{id}.jsonc")), schema_json).unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn empty_query_returns_response_envelope() {
        let daemon = TestDaemon::new().await;
        let resp = daemon.get("/api/v1/search?q=").await;
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body["ok"], true);
        assert!(resp.body["data"]["hits"].is_array());
        assert!(resp.body["data"]["total"].is_number());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn finds_collection_by_id() {
        let daemon = TestDaemon::new().await;
        write_schema(
            &daemon,
            "task",
            r#"{
                "id": "task",
                "name": "Tasks",
                "fields": { "title": { "type": "string", "required": true } }
            }"#,
        );

        let resp = daemon.get("/api/v1/search?q=task").await;
        assert_eq!(resp.status, 200);
        let hits = resp.body["data"]["hits"].as_array().unwrap();
        assert!(
            hits.iter().any(|h| h["kind"] == "collection" && h["id"] == "task"),
            "no task collection hit: {hits:#?}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn finds_collection_item_with_parent_id() {
        let daemon = TestDaemon::new().await;
        write_schema(
            &daemon,
            "note",
            r#"{
                "id": "note",
                "fields": {
                    "title": { "type": "string", "required": true },
                    "body": { "type": "markdown" }
                }
            }"#,
        );
        let create = daemon
            .post(
                "/api/v1/collections/note/items",
                &json!({ "id": "n1", "data": { "title": "Hello searchable world", "body": "..." } }),
            )
            .await;
        assert!(
            create.status == 200 || create.status == 201,
            "create response: {} {:?}", create.status, create.body
        );

        let resp = daemon.get("/api/v1/search?q=searchable").await;
        assert_eq!(resp.status, 200);
        let hits = resp.body["data"]["hits"].as_array().unwrap();
        let item_hit = hits.iter().find(|h| h["kind"] == "collection_item" && h["id"] == "n1");
        let h = item_hit.expect("expected note/n1 collection_item hit");
        assert_eq!(h["collection_id"], "note");
        assert_eq!(h["path"], "/collections/note/items/n1");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn types_filter_restricts_kinds() {
        let daemon = TestDaemon::new().await;
        let resp = daemon.get("/api/v1/search?q=&types=collection").await;
        assert_eq!(resp.status, 200);
        let hits = resp.body["data"]["hits"].as_array().unwrap();
        // Built-in collections (notes, specs, adrs, systems) come back —
        // never any other kind.
        assert!(hits.iter().all(|h| h["kind"] == "collection"));
        assert!(!hits.is_empty(), "expected built-in collections to surface");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pagination_cursor_advances() {
        let daemon = TestDaemon::new().await;
        // Built-in collections give us at least 4 entries to page through.
        let first = daemon.get("/api/v1/search?q=&types=collection&limit=2").await;
        assert_eq!(first.status, 200);
        let first_hits = first.body["data"]["hits"].as_array().unwrap();
        assert_eq!(first_hits.len(), 2);
        let cursor = first.body["data"]["cursor"]
            .as_str()
            .expect("expected cursor on truncated first page");

        let second = daemon
            .get(&format!(
                "/api/v1/search?q=&types=collection&limit=10&cursor={}",
                urlencoded(cursor)
            ))
            .await;
        assert_eq!(second.status, 200);
        let second_hits = second.body["data"]["hits"].as_array().unwrap();
        // Resumed page should hold the remaining built-ins; ids don't
        // overlap with the first page.
        let first_ids: Vec<&str> = first_hits.iter().map(|h| h["id"].as_str().unwrap()).collect();
        for h in second_hits {
            let id = h["id"].as_str().unwrap();
            assert!(!first_ids.contains(&id), "duplicate id across pages: {id}");
        }
    }

    fn urlencoded(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}
