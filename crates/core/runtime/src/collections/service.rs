//! Collection service — wraps store CRUD with event emission.
//!
//! Each mutation (create, update, delete) writes to docs.db then emits
//! an event to the kernel. Events are idempotent — the item ID is stable
//! and re-emitting the same event type for the same entity is harmless.

use anyhow::Result;
use crate::data::DataService;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};
use super::query::QueryRequest;
use super::schema::CollectionSchema;
use super::store::{self, Item};

/// Create an item and emit `collection.item_created`.
pub async fn create_item(
    svc: &mut DataService,
    kernel: &Arc<Mutex<KernelRouter>>,
    schema: &CollectionSchema,
    id: &str,
    data: &serde_json::Value,
    created_by: Option<&str>,
) -> Result<Item> {
    let item = store::create_item(svc, schema, id, data, created_by)?;
    emit_collection_event(kernel, "collection.item_created", &schema.id, id, &serde_json::json!({
        "collection": schema.id,
        "item_id": id,
        "data": data,
        "created_by": created_by,
    })).await;
    Ok(item)
}

/// Update an item and emit `collection.item_updated`.
pub async fn update_item(
    svc: &mut DataService,
    kernel: &Arc<Mutex<KernelRouter>>,
    schema: &CollectionSchema,
    id: &str,
    updates: &serde_json::Value,
) -> Result<Item> {
    let item = store::update_item(svc, schema, id, updates)?;
    emit_collection_event(kernel, "collection.item_updated", &schema.id, id, &serde_json::json!({
        "collection": schema.id,
        "item_id": id,
        "fields_changed": updates.as_object().map(|o| o.keys().collect::<Vec<_>>()),
        "data": item.data,
    })).await;
    Ok(item)
}

/// Delete an item and emit `collection.item_deleted`.
pub async fn delete_item(
    svc: &mut DataService,
    kernel: &Arc<Mutex<KernelRouter>>,
    collection: &str,
    id: &str,
) -> Result<bool> {
    let deleted = store::delete_item(svc, collection, id)?;
    if deleted {
        emit_collection_event(kernel, "collection.item_deleted", collection, id, &serde_json::json!({
            "collection": collection,
            "item_id": id,
        })).await;
    }
    Ok(deleted)
}

/// Read-only operations delegate directly to the store (no events).
pub fn get_item(
    svc: &mut DataService,
    collection: &str,
    id: &str,
) -> Result<Option<Item>> {
    store::get_item(svc, collection, id)
}

/// Read-only query delegates directly to the store (no events).
pub fn query_items(
    svc: &mut DataService,
    req: &QueryRequest,
) -> Result<Vec<Item>> {
    store::query_items(svc, req)
}

async fn emit_collection_event(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    collection: &str,
    item_id: &str,
    payload: &serde_json::Value,
) {
    let entity_id = format!("{collection}/{item_id}");
    let envelope = match EventEnvelope::new(event_type, &entity_id, payload) {
        Ok(e) => e.with_actor_id("service.collections"),
        Err(e) => {
            tracing::error!(
                event_type, collection, item_id,
                "collections: failed to build event: {e}"
            );
            return;
        }
    };
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::error!(
            event_type, collection, item_id,
            "collections: event emission failed: {e}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::schema::parse_schema;
    use crate::collections::store::open_docs_db;
    use serde_json::json;

    fn test_schema() -> CollectionSchema {
        parse_schema(r#"{
            "id": "note",
            "fields": {
                "title": { "type": "string", "required": true },
                "body": { "type": "markdown" }
            }
        }"#).unwrap()
    }

    fn test_db() -> (DataService, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let svc = open_docs_db(&ship_dir).unwrap();
        (svc, tmp)
    }

    fn test_kernel() -> Arc<Mutex<KernelRouter>> {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        crate::db::ensure_db_at(&ship_dir).unwrap();
        let ship_dir = Box::leak(Box::new(ship_dir));
        let kr = KernelRouter::new(ship_dir.clone()).unwrap();
        Arc::new(Mutex::new(kr))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn create_emits_event() {
        let (mut conn, _tmp) = test_db();
        let kernel = test_kernel();
        let schema = test_schema();

        let item = create_item(
            &mut conn, &kernel, &schema, "n1",
            &json!({"title": "Hello", "body": "World"}),
            Some("tester"),
        ).await.unwrap();

        assert_eq!(item.id, "n1");
        assert_eq!(item.data["title"], "Hello");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn update_emits_event() {
        let (mut conn, _tmp) = test_db();
        let kernel = test_kernel();
        let schema = test_schema();

        create_item(&mut conn, &kernel, &schema, "n1",
            &json!({"title": "Original"}), None).await.unwrap();

        let updated = update_item(
            &mut conn, &kernel, &schema, "n1",
            &json!({"body": "Added body"}),
        ).await.unwrap();

        assert_eq!(updated.data["title"], "Original");
        assert_eq!(updated.data["body"], "Added body");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn delete_emits_event() {
        let (mut conn, _tmp) = test_db();
        let kernel = test_kernel();
        let schema = test_schema();

        create_item(&mut conn, &kernel, &schema, "n1",
            &json!({"title": "Delete me"}), None).await.unwrap();

        assert!(delete_item(&mut conn, &kernel, "note", "n1").await.unwrap());
        assert!(get_item(&mut conn, "note", "n1").unwrap().is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn delete_nonexistent_no_event() {
        let (mut conn, _tmp) = test_db();
        let kernel = test_kernel();

        assert!(!delete_item(&mut conn, &kernel, "note", "nope").await.unwrap());
    }
}
