//! Behavioral tests for the three abstraction ports: `SpaceContext`, `DataStore`,
//! `KernelLog`. All tests use in-process, temp-dir backends — no network, no
//! real keychain.

use ship_substrate::data::DataService;
use ship_substrate::events::CallerKind;
use ship_substrate::events::validator::EmitContext;
use ship_substrate::events::{EventFilter, new_isolated_for_test};
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{
    ColumnKind, ColumnSpec, CountQuery, DataStore, DataStoreError, KernelLog, PrimaryKey, Query,
    Row, SingleBackendDataStore, SpaceContext, SpaceId, TableSchema,
};
use ship_types::envelope::EventEnvelope;
use std::sync::Arc;

// ── Fixtures ─────────────────────────────────────────────────────────────────

fn open_data_svc(tmp: &tempfile::TempDir) -> DataService {
    DataService::open_at(&tmp.path().join("test-data.db")).unwrap()
}

fn test_store(tmp: &tempfile::TempDir, space_id: &str) -> SingleBackendDataStore {
    SingleBackendDataStore::new(open_data_svc(tmp), space_id)
}

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk),
        session_id: None,
    }
}

fn simple_schema(name: &str) -> TableSchema {
    TableSchema {
        name: name.to_string(),
        columns: vec![ColumnSpec {
            name: "value".to_string(),
            kind: ColumnKind::Text,
            required: false,
        }],
    }
}

// ── 1. space_context_exposes_data_and_kernel ──────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn space_context_exposes_data_and_kernel() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-1");
    let kernel_arc = new_isolated_for_test(tmp.path().join(".ship")).unwrap();
    let kernel_log = Arc::new(KernelLog::new(kernel_arc));

    let ctx = SpaceContext::new(SpaceId::new("space-1"), Arc::new(store), kernel_log.clone());

    assert_eq!(ctx.space_id().as_str(), "space-1");
    // data() and kernel() return valid references
    let _data: &dyn DataStore = ctx.data();
    let _kernel: &KernelLog = ctx.kernel();
}

// ── 2. data_store_insert_then_query_returns_row ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_insert_then_query_returns_row() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-2");
    let schema = simple_schema("items");

    store.ensure_table(&schema).await.unwrap();

    let row = Row::new("row-1").with("value", serde_json::json!("hello"));
    store.insert("items", row).await.unwrap();

    let result = store.query(Query::new("items")).await.unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].id, "row-1");
    assert_eq!(
        result.rows[0].fields.get("value"),
        Some(&serde_json::json!("hello"))
    );
}

// ── 3. data_store_delete_returns_true_when_present ────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_delete_returns_true_when_present() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-3");
    store.ensure_table(&simple_schema("items")).await.unwrap();
    store.insert("items", Row::new("to-delete")).await.unwrap();

    let deleted = store
        .delete("items", PrimaryKey("to-delete".to_string()))
        .await
        .unwrap();
    assert!(deleted);
}

// ── 4. data_store_delete_returns_false_when_absent ────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_delete_returns_false_when_absent() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-4");
    store.ensure_table(&simple_schema("items")).await.unwrap();

    let deleted = store
        .delete("items", PrimaryKey("ghost".to_string()))
        .await
        .unwrap();
    assert!(!deleted);
}

// ── 5. data_store_query_with_cursor_paginates_deterministically ───────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_query_with_cursor_paginates_deterministically() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-5");
    store.ensure_table(&simple_schema("items")).await.unwrap();

    // Insert rows with ULID-ordered ids so cursor ordering is deterministic.
    for i in 0..5u32 {
        let id = format!("{:026}", i); // zero-padded so lexicographic = numeric
        store.insert("items", Row::new(id)).await.unwrap();
    }

    // Page 1: first 3 rows.
    let page1 = store
        .query(Query {
            table: "items".into(),
            limit: 3,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page1.rows.len(), 3);
    let cursor = page1.next_cursor.clone().unwrap();

    // Insert more rows after fetching page 1 — must not shift page 2.
    for i in 10..15u32 {
        let id = format!("{:026}", i);
        store.insert("items", Row::new(id)).await.unwrap();
    }

    // Page 2: rows after cursor.
    let page2 = store
        .query(Query {
            table: "items".into(),
            limit: 10,
            after: Some(cursor.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    // Page 2 must not include rows from page 1.
    for r in &page2.rows {
        assert!(r.id > cursor, "row {} is not after cursor {}", r.id, cursor);
    }
    // The 2 remaining original rows (ids 3,4) must be present.
    let ids_in_page2: Vec<&str> = page2.rows.iter().map(|r| r.id.as_str()).collect();
    assert!(ids_in_page2.contains(&"00000000000000000000000003"));
    assert!(ids_in_page2.contains(&"00000000000000000000000004"));
}

// ── 6. data_store_count_matches_query_len ─────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_count_matches_query_len() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-6");
    store.ensure_table(&simple_schema("items")).await.unwrap();

    for i in 0..4u32 {
        store
            .insert("items", Row::new(format!("id-{i}")))
            .await
            .unwrap();
    }

    let result = store.query(Query::new("items")).await.unwrap();
    let count = store.count(CountQuery::new("items")).await.unwrap();
    assert_eq!(result.rows.len(), count);
}

// ── 7. kernel_log_emit_then_subscribe_receives_event ─────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_log_emit_then_subscribe_receives_event() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel_arc = new_isolated_for_test(tmp.path().join(".ship")).unwrap();
    let log = KernelLog::new(kernel_arc);

    let mut rx = log.subscribe();
    let envelope = EventEnvelope::new("test.emitted", "entity-7", &serde_json::json!({})).unwrap();
    log.emit(envelope.clone(), &runtime_ctx()).await.unwrap();

    let received = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("channel closed");
    assert_eq!(received.event_type, "test.emitted");
}

// ── 8. kernel_log_subscribe_filtered_excludes_nonmatching ────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_log_subscribe_filtered_excludes_nonmatching() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel_arc = new_isolated_for_test(tmp.path().join(".ship")).unwrap();
    let log = KernelLog::new(kernel_arc);

    let filter = EventFilter {
        event_type: Some("test.wanted".to_string()),
        ..Default::default()
    };
    let mut rx = log.subscribe_filtered(filter);

    // Emit a non-matching event first.
    let noise = EventEnvelope::new("test.noise", "e1", &serde_json::json!({})).unwrap();
    log.emit(noise, &runtime_ctx()).await.unwrap();

    // Then emit the matching event.
    let wanted = EventEnvelope::new("test.wanted", "e2", &serde_json::json!({})).unwrap();
    log.emit(wanted.clone(), &runtime_ctx()).await.unwrap();

    // Await the relay rather than sleeping: the channel is FIFO, so the FIRST
    // delivered event being `test.wanted` proves the earlier noise was
    // filtered (an unfiltered relay would deliver noise first).
    let received = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .expect("relay must deliver within the timeout")
        .expect("filtered channel should have the wanted event");
    assert_eq!(received.event_type, "test.wanted");

    // Channel must be empty now (noise was filtered).
    assert!(rx.try_recv().is_err(), "noise must have been filtered out");
}

// ── 9. data_store_rejects_unsafe_table_name ──────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_rejects_unsafe_table_name() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-9a");

    let schema = TableSchema {
        name: "notes; DROP TABLE notes".to_string(),
        columns: vec![],
    };
    let err = store.ensure_table(&schema).await.unwrap_err();
    assert!(
        matches!(err, DataStoreError::UnsafeIdent(_)),
        "expected UnsafeIdent, got {err:?}"
    );

    let err = store
        .insert("notes; DROP TABLE notes", Row::new("x"))
        .await
        .unwrap_err();
    assert!(matches!(err, DataStoreError::UnsafeIdent(_)));
}

// ── 10. data_store_rejects_unsafe_column_key ─────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_rejects_unsafe_column_key() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-10");
    store.ensure_table(&simple_schema("items")).await.unwrap();

    let row = Row::new("r1").with("bad-key!", serde_json::json!("x"));
    let err = store.insert("items", row).await.unwrap_err();
    assert!(
        matches!(err, DataStoreError::UnsafeIdent(_)),
        "expected UnsafeIdent, got {err:?}"
    );

    let q = Query::new("items").with_eq("bad-key!", serde_json::json!("x"));
    let err = store.query(q).await.unwrap_err();
    assert!(matches!(err, DataStoreError::UnsafeIdent(_)));
}

// ── 11. data_store_accepts_valid_idents ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_store_accepts_valid_idents() {
    let tmp = tempfile::tempdir().unwrap();
    let store = test_store(&tmp, "space-11");

    let schema = TableSchema {
        name: "_my_table_123".to_string(),
        columns: vec![
            ColumnSpec {
                name: "col_one".to_string(),
                kind: ColumnKind::Text,
                required: false,
            },
            ColumnSpec {
                name: "_col2".to_string(),
                kind: ColumnKind::Integer,
                required: false,
            },
        ],
    };
    store.ensure_table(&schema).await.unwrap();

    let row = Row::new("r1")
        .with("col_one", serde_json::json!("hello"))
        .with("_col2", serde_json::json!(42));
    store.insert("_my_table_123", row).await.unwrap();

    let mut q = Query::new("_my_table_123");
    q.sort = Some(ship_substrate::ports::Sort {
        column: "col_one".to_string(),
        descending: false,
    });
    let result = store.query(q).await.unwrap();
    assert_eq!(result.rows.len(), 1);

    let count = store.count(CountQuery::new("_my_table_123")).await.unwrap();
    assert_eq!(count, 1);
}

// ── 12. subscribe_filtered_relay_exits_when_receiver_dropped ─────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscribe_filtered_relay_exits_when_receiver_dropped() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel_arc = new_isolated_for_test(tmp.path().join(".ship")).unwrap();
    let log = KernelLog::new(kernel_arc);

    let baseline = log.source_receiver_count();

    let filter = EventFilter {
        event_type: Some("test.target".to_string()),
        ..Default::default()
    };
    let frx = log.subscribe_filtered(filter);
    assert_eq!(
        log.source_receiver_count(),
        baseline + 1,
        "relay task should hold one source subscription"
    );

    drop(frx);

    // Emit an event (non-matching) to wake the relay task so it detects receiver_count == 0.
    let env = EventEnvelope::new("test.other", "e1", &serde_json::json!({})).unwrap();
    log.emit(env, &runtime_ctx()).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert_eq!(
        log.source_receiver_count(),
        baseline,
        "relay task should have exited and released its source subscription"
    );
}

// ── 13. for_space mounts a PHYSICAL per-space db (substrate keystone) ─────────
//
// Proof of physical partition. Each `SpaceContext::for_space(dir, id)` binds its
// `DataStore` to `spaces/<id>/space.db` — a SEPARATE file per space, not a shared
// `data.db` with a logical `space_id` column. The test writes one row through
// space A's store and another through space B's store, then opens BOTH db files
// directly and asserts each row lives in its own space's file and is PHYSICALLY
// ABSENT from the other's. This fails on the old shared-db model (both rows would
// land in one file) and passes once `for_space` routes to per-space files.

/// Count rows with a given `id` in a collection table inside one space's
/// physical db file. Opens the file directly (not through any DataStore) so the
/// assertion is about bytes on disk, not the store's logical filter.
async fn rows_in_space_db(
    ship_dir: &std::path::Path,
    space_id: &str,
    table: &str,
    id: &str,
) -> i64 {
    use sqlx::Row as _;
    let mut conn = ship_substrate::db::open_space_db(ship_dir, space_id).unwrap();
    // The table may not exist in a space that never wrote to it — that is itself
    // proof of absence, so treat "no such table" as zero rows.
    let sql = format!("SELECT COUNT(*) AS cnt FROM `{table}` WHERE id = ?");
    match sqlx::query(&sql).bind(id).fetch_one(&mut conn).await {
        Ok(row) => row.get::<i64, _>("cnt"),
        Err(_) => 0,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn for_space_lands_rows_in_physically_separate_db_files() {
    use ship_substrate::testutil::init_project_minimal;

    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();

    // Two distinct spaces, same ship-home. Each `for_space` mounts its own
    // physical `spaces/<id>/space.db`. Shared kernel handle injected by the
    // composition site (no process-global router).
    let router = new_isolated_for_test(ship_dir.clone()).unwrap();
    let kernel = Arc::new(KernelLog::new(router));
    let ctx_a = ship_substrate::ports::SpaceContext::for_space(
        &ship_dir,
        SpaceId::new("A"),
        kernel.clone(),
    )
    .unwrap();
    let ctx_b =
        ship_substrate::ports::SpaceContext::for_space(&ship_dir, SpaceId::new("B"), kernel)
            .unwrap();

    let schema = simple_schema("collection_widgets");
    ctx_a.data().ensure_table(&schema).await.unwrap();
    ctx_b.data().ensure_table(&schema).await.unwrap();

    ctx_a
        .data()
        .insert(
            "collection_widgets",
            Row::new("only-in-a").with("value", serde_json::json!("a")),
        )
        .await
        .unwrap();
    ctx_b
        .data()
        .insert(
            "collection_widgets",
            Row::new("only-in-b").with("value", serde_json::json!("b")),
        )
        .await
        .unwrap();

    // The two physical files must be distinct paths.
    let path_a = ship_substrate::db::space_db_path(&ship_dir, "A");
    let path_b = ship_substrate::db::space_db_path(&ship_dir, "B");
    assert_ne!(path_a, path_b, "each space must resolve to its own db file");
    assert!(path_a.exists(), "space A db file must exist on disk");
    assert!(path_b.exists(), "space B db file must exist on disk");

    // A's row is in A's file and PHYSICALLY ABSENT from B's file.
    assert_eq!(
        rows_in_space_db(&ship_dir, "A", "collection_widgets", "only-in-a").await,
        1,
        "A's row must live in spaces/A/space.db"
    );
    assert_eq!(
        rows_in_space_db(&ship_dir, "B", "collection_widgets", "only-in-a").await,
        0,
        "A's row must NOT exist in spaces/B/space.db (physical isolation)"
    );

    // ...and symmetrically for B.
    assert_eq!(
        rows_in_space_db(&ship_dir, "B", "collection_widgets", "only-in-b").await,
        1,
        "B's row must live in spaces/B/space.db"
    );
    assert_eq!(
        rows_in_space_db(&ship_dir, "A", "collection_widgets", "only-in-b").await,
        0,
        "B's row must NOT exist in spaces/A/space.db (physical isolation)"
    );
}
