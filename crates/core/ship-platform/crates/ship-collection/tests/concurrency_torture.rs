//! Scenario 5 — concurrency torture.
//!
//! Interleave `collection.schema.changed` with item writes/reads on the same
//! collection. Master property: no corruption, projection stays consistent.

use std::sync::Arc;

use serde_json::json;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test_with;
use ship_substrate::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

mod common;

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router =
        new_isolated_for_test_with(ship_dir, common::collection_projection_factory()).unwrap();
    Arc::new(KernelLog::new(router))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    let ship_dir = tmp.path().join(".ship");
    ship_substrate::testutil::register_space(&ship_dir, "test").unwrap();
    let space_db = ship_substrate::db::space_db_path(&ship_dir, "test");
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, "test"));
    Arc::new(SpaceContext::new(SpaceId::new("test"), store, kernel))
}

fn service_for(ctx: Arc<SpaceContext>, schemas: Arc<SchemaRegistry>) -> Arc<CollectionService> {
    Arc::new(CollectionService { ctx, schemas })
}

/// Concurrent item creates against a stable schema — no interleaved schema
/// change here, establishes a baseline: N concurrent writers must each
/// succeed and produce N distinct, uncorrupted rows.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_item_creates_produce_no_corruption() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let schemas = Arc::new(SchemaRegistry::new());
    let svc = service_for(ctx.clone(), schemas.clone());

    svc.create_schema(
        "concur",
        &serde_json::to_vec(&json!({
            "id": "concur",
            "fields": { "n": { "type": "number", "required": true } }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    let n_writers = 40;
    let mut handles = Vec::new();
    for i in 0..n_writers {
        let svc = svc.clone();
        handles.push(tokio::spawn(async move {
            svc.create_item("concur", &serde_json::to_vec(&json!({ "n": i })).unwrap())
                .await
        }));
    }
    let mut ok = 0;
    for h in handles {
        if h.await.unwrap().is_ok() {
            ok += 1;
        }
    }
    assert_eq!(ok, n_writers, "every concurrent create_item must succeed");

    let (items, _) = svc
        .query_items_by_query(ship_substrate::ports::Query {
            table: "collection_concur".into(),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        items.len(),
        n_writers,
        "no lost or duplicated rows under concurrency"
    );

    let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n_writers, "all item ids must be distinct");

    let mut ns: Vec<i64> = items
        .iter()
        .filter_map(|i| i.data.get("n").and_then(|v| v.as_f64()).map(|f| f as i64))
        .collect();
    ns.sort();
    assert_eq!(
        ns,
        (0..n_writers as i64).collect::<Vec<_>>(),
        "no value corruption under concurrency"
    );
}

/// Interleave `update_schema` (adding a new column) with concurrent
/// `create_item` calls that straddle the change. Every item — pre-, mid-, and
/// post-evolution — must still be readable afterward with no corruption, and
/// the final schema/table must reflect the last write.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn schema_change_interleaved_with_item_writes_stays_consistent() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let schemas = Arc::new(SchemaRegistry::new());
    let svc = service_for(ctx.clone(), schemas.clone());

    svc.create_schema(
        "evolve_concur",
        &serde_json::to_vec(&json!({
            "id": "evolve_concur",
            "fields": { "title": { "type": "string", "required": true } }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    // Writers using only the original field — these run throughout, both
    // before and racing against the schema change.
    let mut handles = Vec::new();
    for i in 0..20 {
        let svc = svc.clone();
        handles.push(tokio::spawn(async move {
            svc.create_item(
                "evolve_concur",
                &serde_json::to_vec(&json!({ "title": format!("pre-{i}") })).unwrap(),
            )
            .await
        }));
    }

    // The schema change itself, racing the writers above.
    let svc_for_schema = svc.clone();
    let schema_handle = tokio::spawn(async move {
        svc_for_schema
            .update_schema(
                "evolve_concur",
                &serde_json::to_vec(&json!({
                    "id": "evolve_concur",
                    "fields": {
                        "title": { "type": "string", "required": true },
                        "tag": { "type": "string" }
                    }
                }))
                .unwrap(),
            )
            .await
    });

    let schema_result = schema_handle.await.unwrap();
    assert!(
        schema_result.is_ok(),
        "concurrent update_schema must succeed: {schema_result:?}"
    );

    let mut pre_ok = 0;
    for h in handles {
        if h.await.unwrap().is_ok() {
            pre_ok += 1;
        }
    }
    assert_eq!(
        pre_ok, 20,
        "every writer racing the schema change must still succeed"
    );

    // Post-change writers exercising the new field.
    let mut post_handles = Vec::new();
    for i in 0..20 {
        let svc = svc.clone();
        post_handles.push(tokio::spawn(async move {
            svc.create_item(
                "evolve_concur",
                &serde_json::to_vec(&json!({ "title": format!("post-{i}"), "tag": "new" }))
                    .unwrap(),
            )
            .await
        }));
    }
    let mut post_ok = 0;
    for h in post_handles {
        if h.await.unwrap().is_ok() {
            post_ok += 1;
        }
    }
    assert_eq!(post_ok, 20);

    let (items, _) = svc
        .query_items_by_query(ship_substrate::ports::Query {
            table: "collection_evolve_concur".into(),
            limit: 200,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        items.len(),
        40,
        "all pre- and post-evolution writes must survive, none lost"
    );

    let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        40,
        "no duplicated ids under interleaved schema change + writes"
    );

    // Every item's `title` must be intact (non-empty, matches its pre-/post-
    // prefix) — a corrupted projection would show truncated/garbled/blank
    // values for rows written mid-ALTER.
    for item in &items {
        let title = item.data.get("title").and_then(|v| v.as_str());
        assert!(
            title
                .map(|t| t.starts_with("pre-") || t.starts_with("post-"))
                .unwrap_or(false),
            "corrupted title for item {}: {:?}",
            item.id,
            item.data
        );
    }

    let final_schema = svc
        .list()
        .into_iter()
        .find(|s| s.id == "evolve_concur")
        .unwrap();
    assert!(
        final_schema.fields.contains_key("tag"),
        "final schema must reflect the update"
    );
}

/// Concurrent reads racing concurrent writes on the same collection — reads
/// must never observe a torn/partial row (every row that IS visible has all
/// its base columns populated), even while writers are actively inserting.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_reads_never_observe_torn_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let schemas = Arc::new(SchemaRegistry::new());
    let svc = service_for(ctx.clone(), schemas.clone());

    svc.create_schema(
        "torn",
        &serde_json::to_vec(&json!({
            "id": "torn",
            "fields": {
                "a": { "type": "string", "required": true },
                "b": { "type": "number", "required": true }
            }
        }))
        .unwrap(),
    )
    .await
    .unwrap();

    let mut writer_handles = Vec::new();
    for i in 0..30 {
        let svc = svc.clone();
        writer_handles.push(tokio::spawn(async move {
            svc.create_item(
                "torn",
                &serde_json::to_vec(&json!({ "a": format!("val-{i}"), "b": i })).unwrap(),
            )
            .await
        }));
    }

    let mut reader_handles = Vec::new();
    for _ in 0..10 {
        let svc = svc.clone();
        reader_handles.push(tokio::spawn(async move {
            let mut violations = Vec::new();
            for _ in 0..5 {
                if let Ok((items, _)) = svc
                    .query_items_by_query(ship_substrate::ports::Query {
                        table: "collection_torn".into(),
                        limit: 100,
                        ..Default::default()
                    })
                    .await
                {
                    for item in items {
                        let has_a = item.data.get("a").and_then(|v| v.as_str()).is_some();
                        let has_b = item.data.get("b").is_some();
                        if has_a != has_b {
                            violations.push(item.id.clone());
                        }
                    }
                }
            }
            violations
        }));
    }

    for h in writer_handles {
        h.await.unwrap().unwrap();
    }
    for h in reader_handles {
        let violations = h.await.unwrap();
        assert!(violations.is_empty(), "torn rows observed: {violations:?}");
    }
}
