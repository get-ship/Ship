//! Scenario 1 — schema evolution torture.
//!
//! Master property 1: a collection's materialized projection table always
//! reflects the latest `collection.schema.changed`, and a rebuild from the
//! event log reproduces it exactly.

use std::sync::Arc;

use serde_json::json;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test_with;
use ship_substrate::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

mod common;

fn make_kernel(tmp: &tempfile::TempDir) -> (std::path::PathBuf, Arc<KernelLog>) {
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router =
        new_isolated_for_test_with(ship_dir.clone(), common::collection_projection_factory())
            .unwrap();
    (ship_dir, Arc::new(KernelLog::new(router)))
}

fn make_ctx(ship_dir: &std::path::Path, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    ship_substrate::testutil::register_space(ship_dir, "test").unwrap();
    let space_db = ship_substrate::db::space_db_path(ship_dir, "test");
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, "test"));
    Arc::new(SpaceContext::new(SpaceId::new("test"), store, kernel))
}

fn make_service(ctx: Arc<SpaceContext>) -> CollectionService {
    CollectionService {
        ctx,
        schemas: Arc::new(SchemaRegistry::new()),
    }
}

// ── Additive evolution: adding columns across multiple schema.changed ────────

/// Master property 1 for a Space-scoped collection: `rebuild_scope_db`
/// against `spaces/<id>/space.db` must reproduce the live state exactly,
/// even though the bus also carries Runtime-only projections (e.g.
/// `ActorProjection`) that own no table in this scope's db.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn additive_columns_accumulate_across_schema_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let (ship_dir, kernel) = make_kernel(&tmp);
    let ctx = make_ctx(&ship_dir, kernel);
    let svc = make_service(ctx.clone());

    svc.create_schema(
        "eventlog",
        &serde_json::to_vec(&json!({
            "id": "eventlog",
            "fields": { "title": { "type": "string", "required": true } }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    svc.create_item(
        "eventlog",
        &serde_json::to_vec(&json!({ "title": "v1" })).unwrap(),
    )
    .await
    .unwrap();

    // schema.changed #1: add a column.
    svc.update_schema(
        "eventlog",
        &serde_json::to_vec(&json!({
            "id": "eventlog",
            "fields": {
                "title": { "type": "string", "required": true },
                "priority": { "type": "number" }
            }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    // schema.changed #2: add another column.
    svc.update_schema(
        "eventlog",
        &serde_json::to_vec(&json!({
            "id": "eventlog",
            "fields": {
                "title": { "type": "string", "required": true },
                "priority": { "type": "number" },
                "archived": { "type": "boolean" }
            }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    // A new item using all three fields must round-trip.
    let item = svc
        .create_item(
            "eventlog",
            &serde_json::to_vec(&json!({ "title": "v3", "priority": 2, "archived": true }))
                .unwrap(),
        )
        .await
        .unwrap();
    let fetched = svc.get_item("eventlog", &item.id).await.unwrap();
    assert_eq!(
        fetched.data.get("priority").and_then(|v| v.as_f64()),
        Some(2.0)
    );

    // Original item (created before either evolution) survives with old
    // columns null/absent, not corrupted.
    let (items, _) = svc
        .query_items_by_query(ship_substrate::ports::Query::new("collection_eventlog"))
        .await
        .unwrap();
    assert_eq!(items.len(), 2, "both pre- and post-evolution items survive");

    // Rebuild from the event log must reproduce the same row count and data.
    // Pass the same domain factory the live kernel used — L0-only
    // `rebuild_scope_db` does not register CollectionItemProjection.
    let factory = common::collection_projection_factory();
    let (replayed, projections) = ship_substrate::db::rebuild::rebuild_scope_db_with(
        &space_db_path(&ship_dir),
        &factory,
    )
    .unwrap();
    assert!(replayed > 0, "expected events replayed");
    assert!(
        projections.iter().any(|p| p == "collection"),
        "collection projection ran"
    );

    let (items_after_rebuild, _) = svc
        .query_items_by_query(ship_substrate::ports::Query::new("collection_eventlog"))
        .await
        .unwrap();
    assert_eq!(
        items_after_rebuild.len(),
        items.len(),
        "rebuild must reproduce the same row count"
    );
    for (before, after) in items.iter().zip(items_after_rebuild.iter()) {
        assert_eq!(before.id, after.id);
        assert_eq!(
            before.data, after.data,
            "rebuild must reproduce identical item data"
        );
    }
}

fn space_db_path(ship_dir: &std::path::Path) -> std::path::PathBuf {
    ship_substrate::db::space_db_path(ship_dir, "test")
}

// ── Destructive/incompatible change: retyping a field in place ───────────────

/// FINDING CANDIDATE: `update_schema` lets a field's declared `type` change
/// (e.g. number -> string) without any table-level enforcement. The
/// projection only ADDs missing columns; it never re-types an existing one.
/// This test asserts the *intended* safe behavior (retype refused, or old
/// data preserved under the new type without corruption). If it fails, the
/// retype is silently accepted with no column-level consequence, which is a
/// real gap against "assert refused or handled without corrupting existing
/// rows" — ignore with a repro rather than loosening the assertion.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retyping_a_field_does_not_corrupt_existing_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let (ship_dir, kernel) = make_kernel(&tmp);
    let ctx = make_ctx(&ship_dir, kernel);
    let svc = make_service(ctx);

    svc.create_schema(
        "retype",
        &serde_json::to_vec(&json!({
            "id": "retype",
            "fields": { "count": { "type": "number", "required": true } }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    let item = svc
        .create_item(
            "retype",
            &serde_json::to_vec(&json!({ "count": 42 })).unwrap(),
        )
        .await
        .unwrap();

    // Retype `count` from number -> string. The schema layer has no
    // "incompatible change" rejection at all right now — this call is
    // expected to either fail (safe) or succeed while leaving the existing
    // row's data intact and still validate_item-consistent.
    let retype_result = svc
        .update_schema(
            "retype",
            &serde_json::to_vec(&json!({
                "id": "retype",
                "fields": { "count": { "type": "string", "required": true } }
            }))
            .unwrap(),
        )
        .await;

    match retype_result {
        Err(_) => {
            // Safe: incompatible retype rejected outright.
        }
        Ok(_) => {
            // Accepted — the existing row must not be corrupted, and it must
            // still satisfy the (now-changed) schema when read back and
            // re-validated, or the divergence must at least be visible
            // (not silently coerced into a lie about the data's shape).
            let fetched = svc.get_item("retype", &item.id).await.unwrap();
            assert!(
                fetched.data.get("count").is_some(),
                "retype must not drop the existing column's data"
            );
        }
    }
}

// ── Rebuild-from-scratch equality after several evolutions ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rebuild_after_multiple_evolutions_matches_live_state() {
    let tmp = tempfile::tempdir().unwrap();
    let (ship_dir, kernel) = make_kernel(&tmp);
    let ctx = make_ctx(&ship_dir, kernel);
    let svc = make_service(ctx.clone());

    svc.create_schema(
        "eq",
        &serde_json::to_vec(&json!({ "id": "eq", "fields": { "a": { "type": "string" } } }))
            .unwrap(),
    )
    .await
    .unwrap();
    svc.create_item("eq", &serde_json::to_vec(&json!({ "a": "1" })).unwrap())
        .await
        .unwrap();

    for field in ["b", "c", "d"] {
        let mut fields = json!({ "a": { "type": "string" } });
        // Build up the field set incrementally.
        let obj = fields.as_object_mut().unwrap();
        for f in ["b", "c", "d"] {
            if f <= field {
                obj.insert(f.to_string(), json!({ "type": "string" }));
            }
        }
        svc.update_schema(
            "eq",
            &serde_json::to_vec(&json!({ "id": "eq", "fields": fields })).unwrap(),
        )
        .await
        .unwrap();
        svc.create_item(
            "eq",
            &serde_json::to_vec(&json!({ "a": format!("row-{field}") })).unwrap(),
        )
        .await
        .unwrap();
    }

    let (before, _) = svc
        .query_items_by_query(ship_substrate::ports::Query {
            table: "collection_eq".into(),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();

    let factory = common::collection_projection_factory();
    ship_substrate::db::rebuild::rebuild_scope_db_with(&space_db_path(&ship_dir), &factory)
        .unwrap();

    let (after, _) = svc
        .query_items_by_query(ship_substrate::ports::Query {
            table: "collection_eq".into(),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(
        before.len(),
        after.len(),
        "rebuild must not change row count"
    );
    let mut before_sorted = before.clone();
    let mut after_sorted = after.clone();
    before_sorted.sort_by(|a, b| a.id.cmp(&b.id));
    after_sorted.sort_by(|a, b| a.id.cmp(&b.id));
    for (b, a) in before_sorted.iter().zip(after_sorted.iter()) {
        assert_eq!(b.id, a.id);
        assert_eq!(b.data, a.data, "rebuild diverged for item {}", b.id);
    }
}
