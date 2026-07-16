//! Schema registry survives a daemon restart: API-created schemas land in
//! `_schema_registry` and hydrate back into the in-memory `SchemaRegistry`
//! on open. Regression: `c-2026-07-02T01-40-33` — `_schema_registry` was a
//! write-only orphan (Create had no `persist_schema` call, hydrate read from
//! built-ins + .ship overlay only). Until pre-launch fixed Create, an
//! API-created schema lived only in the in-memory map and silently
//! disappeared on restart.

use std::sync::Arc;

use ship_collection::projection::{SchemaRegistry, hydrate_from_store};
use ship_collection::service::CollectionService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::KernelLog;
use ship_substrate::{SingleBackendDataStore, SpaceContext, SpaceId};
use ship_substrate::testutil::init_project_minimal;
use tempfile::tempdir;

mod common;

fn make_service(ship_dir: &std::path::Path, space_id: &str) -> CollectionService {
    common::install_collection_projection();
    // I-2: register the fabricated `space_id` so `PartitionedSink::route()`
    // doesn't refuse this suite's Space-scoped `collection.*` events.
    ship_substrate::testutil::register_space(ship_dir, space_id).unwrap();
    let router = new_isolated_for_test(ship_dir.to_path_buf()).unwrap();
    let kernel = Arc::new(KernelLog::new(router));
    let db_path = ship_substrate::db::space_db_path(ship_dir, space_id);
    let data_svc = DataService::open_at(&db_path).unwrap();
    let data_store: Arc<dyn ship_substrate::DataStore> =
        Arc::new(SingleBackendDataStore::new(data_svc, space_id));
    let ctx = Arc::new(SpaceContext::new(
        SpaceId::new(space_id.to_string()),
        data_store,
        kernel,
    ));
    CollectionService {
        ctx,
        schemas: Arc::new(SchemaRegistry::new()),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn api_created_schema_survives_restart() {
    let tmp = tempdir().unwrap();
    init_project_minimal(tmp.path()).unwrap();
    let ship_dir = tmp.path().join(".ship");

    // ── Phase 1: daemon session that creates a schema via the API ──────
    let svc1 = make_service(&ship_dir, "space-restart-test");
    let schema_json = br#"{
        "id": "tasks",
        "title": "Tasks",
        "fields": {
            "title": { "type": "string", "required": true },
            "done":  { "type": "boolean" }
        }
    }"#;
    svc1
        .create_schema("tasks", schema_json)
        .await
        .expect("create schema");
    assert!(svc1.schemas.get("tasks").is_some(), "in-memory registry sees the new schema");

    // ── Phase 2: fresh daemon session — a NEW SchemaRegistry against the
    // SAME space.db, simulating daemon restart. Built-in + file-overlay
    // hydration alone would NOT restore `tasks` because no .ship file
    // declares it. With `hydrate_from_store`, the durable row comes back.
    let router2 = new_isolated_for_test(ship_dir.clone()).unwrap();
    let kernel2 = Arc::new(KernelLog::new(router2));
    let db_path = ship_substrate::db::space_db_path(&ship_dir, "space-restart-test");
    let data_svc2 = DataService::open_at(&db_path).unwrap();
    let data_store2: Arc<dyn ship_substrate::DataStore> =
        Arc::new(SingleBackendDataStore::new(data_svc2, "space-restart-test"));

    let registry_after_restart = Arc::new(SchemaRegistry::new());
    hydrate_from_store(&registry_after_restart, data_store2.as_ref())
        .await
        .expect("hydrate_from_store reads _schema_registry");

    let restored = registry_after_restart
        .get("tasks")
        .expect("tasks schema must hydrate from _schema_registry on restart (claim c-2026-07-02T01-40-33)");
    assert_eq!(restored.id, "tasks");

    assert!(
        restored.title.as_deref() == Some("Tasks"),
        "hydrated doc carries its title: got {:?}", restored.title
    );
}
