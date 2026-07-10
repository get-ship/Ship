//! Snapshot drill — the restore + live-writes behavioural tests for the
//! `SpaceService.Snapshot` RPC + substrate `snapshot_space` / `restore_space`.
//!
//! These are the "drill" the job spec calls out: in-process proof that
//! snapshot → restore round-trips a real space end-to-end (collection
//! items, artifact blobs), and that snapshot-under-live-writes produces
//! a self-consistent bundle (the writer's later writes are simply not
//! in the snapshot — that is the contract, not a checkpointing system).
//!
//! Tests use temp-dir ports — no network, no real daemon. Each test
//! gets its own ship-home and kernel; projections are installed once
//! per test binary via the `install_*(...)` helpers below.

use std::sync::Arc;
use std::time::Duration;

use ship_artifacts::service::ArtifactService;
use ship_artifacts::types::ArtifactScope;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

// ── Projection installs ────────────────────────────────────────────────────

/// `tests/common/` is a subdirectory so Cargo does not compile it as a
/// standalone test binary. Each test that needs a projection calls the
/// install helper (idempotent via `Once`).
mod common;

const SPACE_ID: &str = "drill-space";

// ── Fixtures ───────────────────────────────────────────────────────────────

/// Test root. `data_root` is the XDG-data equivalent — the directory
/// the kernel + `LocalFsBlobStore` both root at, so the db lives at
/// `<data_root>/spaces/<id>/space.db` and the blobs live at
/// `<data_root>/spaces/<id>/blobs/<h[:2]>/<h>`. That collocation is
/// what `snapshot_space` (and the production layout) expects.
struct TestRig {
    _tmp: tempfile::TempDir,
    data_root: std::path::PathBuf,
    kernel: Arc<KernelLog>,
}

fn make_rig() -> TestRig {
    common::install_all_projections();
    let tmp = tempfile::tempdir().unwrap();
    let data_root = tmp.path().join(".ship");
    std::fs::create_dir_all(&data_root).unwrap();
    let router = new_isolated_for_test(data_root.clone()).unwrap();
    let kernel = Arc::new(KernelLog::new(router));
    TestRig {
        _tmp: tmp,
        data_root,
        kernel,
    }
}

fn make_ctx(rig: &TestRig, space_id: &str) -> Arc<SpaceContext> {
    let space_db = ship_substrate::db::space_db_path(&rig.data_root, space_id);
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, space_id));
    Arc::new(SpaceContext::new(SpaceId::new(space_id), store, rig.kernel.clone()))
}

fn make_collection_service(ctx: Arc<SpaceContext>) -> CollectionService {
    CollectionService {
        ctx,
        schemas: Arc::new(SchemaRegistry::new()),
    }
}

fn make_artifact_service(rig: &TestRig, ctx: Arc<SpaceContext>) -> ArtifactService {
    // Root the blob store at the SAME `data_root` the kernel uses, so
    // blobs land at `<data_root>/spaces/<id>/blobs/...` — exactly where
    // `snapshot_space` looks for them. This is the production layout;
    // existing artifacts tests root at `tmp.path()` only because they
    // don't also wire a kernel's space-db, and the test doesn't care.
    ArtifactService::new(ctx, rig.data_root.clone())
}

fn make_collection_schema_json(id: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "id": id,
        "title": id,
        "system": false,
        "fields": {
            "title": { "type": "string", "required": true },
            "done":  { "type": "boolean" }
        }
    }))
    .unwrap()
}

// ── The drill ─────────────────────────────────────────────────────────────

/// Snapshot → corrupt → restore → assert query parity + blob byte-parity.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_restore_drill_round_trips_collection_items_and_artifacts() {
    let rig = make_rig();
    let ctx = make_ctx(&rig, SPACE_ID);
    let collections = make_collection_service(ctx.clone());
    let artifacts = make_artifact_service(&rig, ctx.clone());

    // 1. Create a collection schema + 3 items.
    let schema_json = make_collection_schema_json("tasks");
    collections.create_schema("tasks", &schema_json).await.unwrap();
    for (i, title) in ["alpha", "beta", "gamma"].iter().enumerate() {
        let body = serde_json::to_vec(&serde_json::json!({
            "title": *title,
            "done": i % 2 == 0
        }))
        .unwrap();
        collections.create_item("tasks", &body).await.unwrap();
    }
    // Wait briefly for the projection to materialise the rows.
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 2. Store 2 artifacts (real content-addressed blobs on disk).
    let blob1: &[u8] = b"the quick brown fox jumps over the lazy dog";
    let blob2: &[u8] = b"a much longer blob for round-trip parity - content-addressed, sha256-pinned";
    let scope = ArtifactScope::Project { space_id: SPACE_ID.to_string() };
    let rec1 = artifacts
        .store("readme.txt", "text/plain", blob1, &scope, None, None, None, None)
        .await
        .unwrap();
    let rec2 = artifacts
        .store("paper.md", "text/markdown", blob2, &scope, None, None, None, None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 3. Snapshot to a fresh target dir.
    let target = tempfile::tempdir().unwrap();
    let res =
        ship_substrate::snapshot_space(&rig.data_root, SPACE_ID, target.path()).expect("snapshot");
    assert_eq!(res.space_id, SPACE_ID);
    assert_eq!(res.blob_count, 2);
    assert_eq!(res.blob_bytes as usize, blob1.len() + blob2.len());
    assert!(res.db_bytes > 0);
    assert_eq!(res.db_sha256.len(), 64);
    eprintln!(
        "[drill] snapshot: space_id={} path={} db_bytes={} db_sha256={} \
         blob_count={} blob_bytes={}",
        res.space_id,
        res.path.display(),
        res.db_bytes,
        res.db_sha256,
        res.blob_count,
        res.blob_bytes,
    );

    // 4. CORRUPT + DELETE the original space on disk — the source must
    // be gone before we restore (the drill proves we can recover from
    // destruction, not a side-by-side consistency check).
    let live_space_dir = rig.data_root.join("spaces").join(SPACE_ID);
    std::fs::remove_dir_all(&live_space_dir).expect("remove live space");
    assert!(!live_space_dir.exists());

    // 5. Restore to a fresh data root.
    let dest = tempfile::tempdir().unwrap();
    let dest_root = dest.path().join(".ship");
    let r = ship_substrate::restore_space(&res.path, &dest_root, SPACE_ID, false)
        .expect("restore");
    assert_eq!(r.space_id, SPACE_ID);
    assert_eq!(r.blob_count, 2);
    assert_eq!(r.blob_bytes as usize, blob1.len() + blob2.len());

    // 6. Open the restored space via a fresh kernel + data store and
    // assert query parity + blob byte-parity.
    let restored_kernel = {
        let router = new_isolated_for_test(dest_root.clone()).unwrap();
        Arc::new(KernelLog::new(router))
    };
    let restored_db = ship_substrate::db::space_db_path(&dest_root, SPACE_ID);
    let svc = DataService::open_at(&restored_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, SPACE_ID));
    let rctx = Arc::new(SpaceContext::new(
        SpaceId::new(SPACE_ID),
        store,
        restored_kernel,
    ));

    // 6a. Bundled db must pass integrity_check.
    {
        let mut conn = ship_substrate::db::open_db_at(&restored_db).unwrap();
        let rows: Vec<(String,)> = sqlx::query_as("PRAGMA integrity_check")
            .fetch_all(&mut conn)
            .await
            .unwrap();
        assert!(
            rows.len() == 1 && rows[0].0 == "ok",
            "restored db must be integrity-clean; got {rows:?}"
        );
    }

    // 6b. Collection items round-trip — query the projection's table.
    {
        let rcoll = CollectionService {
            ctx: rctx.clone(),
            schemas: Arc::new(SchemaRegistry::new()),
        };
        // Manually query the collection_tasks table the projection
        // populated. The schema registry doesn't matter for the query
        // path; we go straight through the data store.
        let q = ship_substrate::ports::Query::new("collection_tasks");
        let (items, _) = rcoll.query_items_by_query(q).await.unwrap();
        assert_eq!(items.len(), 3, "restored space must contain 3 items");
        let titles: std::collections::BTreeSet<String> = items
            .iter()
            .filter_map(|i| i.data.get("title").and_then(|v| v.as_str()).map(str::to_string))
            .collect();
        let expected: std::collections::BTreeSet<String> =
            ["alpha", "beta", "gamma"].iter().map(|s| s.to_string()).collect();
        assert_eq!(titles, expected);
    }

    // 6c. Artifact blobs round-trip — bytes match the originals.
    let rarts = ArtifactService::new(rctx.clone(), dest_root.clone());
    let (_rec1_restored, b1) = rarts.get_blob(&rec1.id).await.unwrap();
    let (_rec2_restored, b2) = rarts.get_blob(&rec2.id).await.unwrap();
    assert_eq!(b1, blob1, "blob1 bytes must round-trip");
    assert_eq!(b2, blob2, "blob2 bytes must round-trip");
}

/// Snapshot a space under concurrent writes — the bundled db must be
/// self-consistent (`PRAGMA integrity_check` clean) and contain a
/// coherent prefix of the writer's row stream. This is the consistency
/// contract: `VACUUM INTO` is crash-consistent, NOT a checkpoint of
/// the writer at the moment of the snapshot call.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_under_live_writes_is_self_consistent() {
    let rig = make_rig();
    let ctx = make_ctx(&rig, SPACE_ID);
    let collections = make_collection_service(ctx.clone());
    let schema_json = make_collection_schema_json("tasks");
    collections.create_schema("tasks", &schema_json).await.unwrap();
    // Seed a few rows so the snapshot has content.
    for (i, title) in ["one", "two", "three"].iter().enumerate() {
        let body = serde_json::to_vec(&serde_json::json!({
            "title": *title,
            "done": i == 0
        }))
        .unwrap();
        collections.create_item("tasks", &body).await.unwrap();
    }
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Writer task hammers the live space while we snapshot.
    let writer = {
        let kernel = rig.kernel.clone();
        tokio::spawn(async move {
            for i in 0..40 {
                let ctx = make_ctx_via(&kernel, SPACE_ID);
                let svc = CollectionService {
                    ctx,
                    schemas: Arc::new(SchemaRegistry::new()),
                };
                let body = serde_json::to_vec(&serde_json::json!({
                    "title": format!("writer-{i}"),
                    "done": false
                }))
                .unwrap();
                let _ = svc.create_item("tasks", &body).await;
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
    };

    let target = tempfile::tempdir().unwrap();
    let res = ship_substrate::snapshot_space(&rig.data_root, SPACE_ID, target.path())
        .expect("snapshot under live writes");
    let _ = writer.await;
    let mut conn = ship_substrate::db::open_db_at(&res.path.join("space.db")).unwrap();
    let row_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM collection_tasks")
        .fetch_one(&mut conn)
        .await
        .unwrap_or((-1,));
    eprintln!(
        "[drill] snapshot-under-writes: space_id={} path={} db_bytes={} db_sha256={} \
         blob_count={} blob_bytes={} task_rows={}",
        res.space_id,
        res.path.display(),
        res.db_bytes,
        res.db_sha256,
        res.blob_count,
        res.blob_bytes,
        row_count.0,
    );
    // Bundled db: integrity-clean + at least the seeded rows.
    let mut conn = ship_substrate::db::open_db_at(&res.path.join("space.db")).unwrap();
    let rows: Vec<(String,)> = sqlx::query_as("PRAGMA integrity_check")
        .fetch_all(&mut conn)
        .await
        .unwrap();
    assert!(
        rows.len() == 1 && rows[0].0 == "ok",
        "bundled db must be integrity-clean; got {rows:?}"
    );
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM collection_tasks")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert!(
        count.0 >= 3,
        "bundled db must contain the seeded prefix; got {} rows",
        count.0
    );
}

/// `SpaceService::snapshot` (the wire backing) rejects a non-absolute
/// `target_dir` with a typed invalid-argument error, as the spec demands.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn snapshot_service_rejects_relative_target_dir() {
    let rig = make_rig();
    let ctx = make_ctx(&rig, SPACE_ID);
    let svc = ship_space::service::SpaceService::new(ctx);
    let rel = std::path::PathBuf::from("relative/dir");
    let err = svc.snapshot(SPACE_ID, &rel).await.expect_err("must reject relative");
    assert!(
        matches!(err, ship_space::error::SpaceError::BadArgs(_)),
        "expected BadArgs, got {err:?}"
    );
}

/// `SpaceService::snapshot` rejects an empty `space_id` when there is
/// no served space on the request's ctx — "snapshot of nothing" is not
/// a meaningful op, typed invalid-argument.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn snapshot_service_requires_a_target_space() {
    let rig = make_rig();
    // ctx bound to the empty/space-less slot.
    let db_path = rig.data_root.join("runtime.db");
    let svc_data = DataService::open_at(&db_path).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc_data, ""));
    let ctx = Arc::new(SpaceContext::new(
        SpaceId::new(""),
        store,
        rig.kernel.clone(),
    ));
    let svc = ship_space::service::SpaceService::new(ctx);
    let target = tempfile::tempdir().unwrap();
    let err = svc
        .snapshot("", target.path())
        .await
        .expect_err("must reject empty space_id without served space");
    assert!(
        matches!(err, ship_space::error::SpaceError::BadArgs(_)),
        "expected BadArgs, got {err:?}"
    );
}

/// Helper for the writer task in the live-writes test — same layout
/// as `make_ctx` but takes the kernel by ref so the closure stays Send.
fn make_ctx_via(kernel: &Arc<KernelLog>, space_id: &str) -> Arc<SpaceContext> {
    // The writer re-uses the data_root the kernel was rooted at:
    // `make_rig` set `rig.data_root` to the same path the kernel
    // routes to, so the writer resolves it via `try_ship_dir`.
    let space_db =
        ship_substrate::db::space_db_path(&kernel.try_ship_dir().expect("kernel ship_dir"), space_id);
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, space_id));
    Arc::new(SpaceContext::new(
        SpaceId::new(space_id),
        store,
        kernel.clone(),
    ))
}

