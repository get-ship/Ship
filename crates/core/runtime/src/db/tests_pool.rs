//! Tests for S2: connection pool handles concurrent access.

use crate::db::{ensure_db_at, pool};
use ship_types::envelope::EventEnvelope;
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

#[tokio::test(flavor = "multi_thread")]
async fn pool_creates_and_acquires() {
    let (_tmp, ship_dir) = setup();
    let db_path = ship_dir.join("platform.db");
    let p = pool::create_pool(&db_path, 5).unwrap();
    let mut conn = p.acquire().await.unwrap();

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM events")
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!(count.0, 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn pool_handles_concurrent_reads() {
    let (_tmp, ship_dir) = setup();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    for i in 0..10 {
        let ev = EventEnvelope::new(
            "test.pool",
            &format!("entity-{i}"),
            &serde_json::json!({"i": i}),
        )
        .unwrap();
        store.append(&ev).unwrap();
    }

    let db_path = ship_dir.join("platform.db");
    let p = pool::create_pool(&db_path, 4).unwrap();

    // 10 concurrent reads through the pool.
    let mut handles = Vec::new();
    for _ in 0..10 {
        let pool = p.clone();
        handles.push(tokio::spawn(async move {
            let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
                .fetch_one(&pool)
                .await
                .unwrap();
            row.0
        }));
    }

    for h in handles {
        let count = h.await.unwrap();
        assert_eq!(count, 10, "all concurrent reads must succeed");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn pooled_event_store_append_and_get() {
    let (_tmp, ship_dir) = setup();
    let store = SqliteEventStore::new_pooled(&ship_dir).unwrap();

    let ev = EventEnvelope::new(
        "test.pooled_append",
        "entity-1",
        &serde_json::json!({"pooled": true}),
    )
    .unwrap();
    store.append(&ev).unwrap();

    let got = store.get(&ev.id).unwrap();
    assert!(got.is_some(), "event appended via pool must be retrievable");
    assert_eq!(got.unwrap().event_type, "test.pooled_append");
}
