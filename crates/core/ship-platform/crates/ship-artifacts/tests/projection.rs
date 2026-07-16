//! Proof tests for the event-sourced artifact METADATA projection (ADR D6/D7).
//!
//! These assert the FLIPPED model: the artifact metadata write path is
//! emit-only — the service validates, writes the BLOB to the content-addressed
//! blob plane, and emits an `artifact.*` event; an `ArtifactProjection` on the
//! per-scope event bus materializes the `collection_artifacts` metadata table.
//!
//! BLOBS are NOT event-sourced — only the relational metadata becomes a
//! projection. The blob backend (`LocalFsBackend`) still writes bytes directly,
//! content-addressed by scope + id, exactly as before.
//!
//! The read path (`ctx.data()`) and the projection both bind to the SAME
//! per-scope db (`spaces/<scope-space>/space.db`), so a write is visible to a
//! query IMMEDIATELY (synchronous, in the append transaction) — no
//! read-after-write gap. Per-space isolation is automatic: each space's
//! `artifact.*` events route to its own `space.db`, where its projection
//! materializes its own `collection_artifacts` rows.
//!
//! All tests are isolated via `init_project_minimal` so they never touch
//! `~/.ship`.

use std::collections::BTreeMap;
use std::sync::Arc;

use ship_artifacts::service::ArtifactService;
use ship_artifacts::types::ArtifactScope;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};
use ship_substrate::testutil::init_project_minimal;

mod common;

// ── Fixtures ────────────────────────────────────────────────────────────────

/// Build an `ArtifactService` whose data store reads from the SAME per-scope
/// `space.db` the projection writes into, bound to `scope_space`. The kernel
/// router is rooted at the shared `ship_dir`, so `artifact.*` (Space-scoped)
/// events route to `spaces/<scope_space>/space.db`, and the data store opens
/// that exact file with its partition key set to the same space — so the read
/// path queries exactly what the projection wrote.
fn make_service(ship_dir: &std::path::Path, scope_space: &str) -> ArtifactService {
    // Ensure ArtifactProjection is in the extra-projections factory before
    // any SqliteEventStore (and its default_bus) is constructed.
    common::install_artifact_projection();
    let router = new_isolated_for_test(ship_dir.to_path_buf()).unwrap();
    let kernel = Arc::new(KernelLog::new(router));

    // I-2: register the fabricated space id so `PartitionedSink::route()`
    // doesn't refuse this suite's Space-scoped `artifact.*` events.
    ship_substrate::testutil::register_space(ship_dir, scope_space).unwrap();
    let space_db = ship_substrate::db::space_db_path(ship_dir, scope_space);
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, scope_space));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(scope_space), store, kernel));

    ArtifactService::new(ctx, ship_dir.to_path_buf())
}

const SPACE_A: &str = "space-a";
const SPACE_B: &str = "space-b";

fn project_scope(space: &str) -> ArtifactScope {
    ArtifactScope::Project { space_id: space.to_string() }
}

fn run_scope(space: &str, run: &str) -> ArtifactScope {
    ArtifactScope::Run { space_id: space.to_string(), run_id: run.to_string() }
}

// ── Test 1: emit-only store materializes the metadata row, read back NOW ──────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn store_projects_metadata_and_is_queryable_immediately() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, SPACE_A);

    let record = svc
        .store("hello.txt", "text/plain", b"hello world", &project_scope(SPACE_A), None, None, None, None)
        .await
        .unwrap();

    // Read-after-write: the projection wrote the metadata row in the same append
    // the emit awaited, so get_record sees it with no sleep / no polling.
    let fetched = svc.get_record(&record.id).await.unwrap().expect("metadata projected");
    assert_eq!(fetched.id, record.id);
    assert_eq!(fetched.name, "hello.txt");
    assert_eq!(fetched.mime, "text/plain");
    assert_eq!(fetched.size, 11);
    assert_eq!(fetched.scope, "project");
    assert_eq!(fetched.space_id.as_deref(), Some(SPACE_A));
    assert_eq!(fetched.path, record.path, "projected path must match the blob path");
    assert!(fetched.content_hash.is_some(), "content_hash projected");
    assert_eq!(fetched.created_at, record.created_at);

    // Bytes round-trip from the blob plane (not event-sourced).
    let (_, bytes) = svc.get_blob(&record.id).await.unwrap();
    assert_eq!(bytes, b"hello world");
}

// ── Test 2: two spaces are isolated (A's row absent from B) ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn distinct_spaces_are_isolated() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();

    let svc_a = make_service(&ship_dir, SPACE_A);
    let svc_b = make_service(&ship_dir, SPACE_B);

    let a_rec = svc_a
        .store("only-a.txt", "text/plain", b"a-bytes", &project_scope(SPACE_A), None, None, None, None)
        .await
        .unwrap();

    // Space A sees its artifact metadata.
    let a_all = svc_a.list(BTreeMap::new(), 50).await.unwrap();
    assert_eq!(a_all.len(), 1);
    assert_eq!(a_all[0].id, a_rec.id);
    assert_eq!(a_all[0].name, "only-a.txt");

    // Space B sees nothing — A's row lives in spaces/space-a/space.db only.
    let b_all = svc_b.list(BTreeMap::new(), 50).await.unwrap();
    assert!(b_all.is_empty(), "space B must not see space A's artifact");
    assert!(svc_b.get_record(&a_rec.id).await.unwrap().is_none());

    // And the per-space db files are physically distinct.
    let a_db = ship_substrate::db::space_db_path(&ship_dir, SPACE_A);
    let b_db = ship_substrate::db::space_db_path(&ship_dir, SPACE_B);
    assert!(a_db.exists(), "space-a db must exist");
    assert_ne!(a_db, b_db);
}

// ── Test 3: artifact.updated projects (patched columns reflected) ─────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_projects_updated_columns() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, SPACE_A);

    let rec = svc
        .store("before.txt", "text/plain", b"data", &project_scope(SPACE_A), None, None, None, None)
        .await
        .unwrap();

    svc.patch(&rec.id, Some("after.txt"), Some("agent-9"), None).await.unwrap();

    let fetched = svc.get_record(&rec.id).await.unwrap().expect("still present");
    assert_eq!(fetched.name, "after.txt", "patched name must be projected");
    assert_eq!(fetched.owner_actor.as_deref(), Some("agent-9"));
    assert_eq!(fetched.mime, "text/plain", "untouched field must survive the projected update");
}

// ── Test 4: artifact.deleted projects (row removed, blob gone) ────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_projects_removal() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, SPACE_A);

    let keep = svc
        .store("keep.txt", "text/plain", b"keep", &project_scope(SPACE_A), None, None, None, None)
        .await
        .unwrap();
    let gone = svc
        .store("gone.txt", "text/plain", b"gone", &project_scope(SPACE_A), None, None, None, None)
        .await
        .unwrap();

    let deleted = svc.delete(&gone.id).await.unwrap();
    assert!(deleted);

    let all = svc.list(BTreeMap::new(), 50).await.unwrap();
    assert_eq!(all.len(), 1, "deleted row must be gone from the projection");
    assert_eq!(all[0].id, keep.id);
    assert!(svc.get_record(&gone.id).await.unwrap().is_none());
}

// ── Test 5: artifact.moved (promote) projects insert-new + delete-old ─────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn promote_projects_move_within_space() {
    // Promote moves run-scope → project-scope WITHIN the same space (the scope
    // space is unchanged; only the scope KIND changes). Both the old and new
    // rows live in the same space.db, so a single emit-only `artifact.moved`
    // projection must insert the new metadata row and delete the old one.
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, SPACE_A);

    let rec = svc
        .store("promo.bin", "application/octet-stream", b"promote me", &run_scope(SPACE_A, "run1"), None, None, None, Some("2020-01-01T00:00:00Z"))
        .await
        .unwrap();
    assert_eq!(rec.scope, "run");

    let promoted = svc
        .promote(&rec.id, project_scope(SPACE_A), true)
        .await
        .unwrap();
    assert_eq!(promoted.scope, "project");
    assert!(promoted.expires_at.is_none(), "retained=true clears expiry");

    // New row projected.
    let new_rec = svc.get_record(&promoted.id).await.unwrap().expect("new metadata projected");
    assert_eq!(new_rec.scope, "project");
    assert!(new_rec.run_id.is_none(), "promote to project clears run_id");

    // Old row deleted by the same projection.
    assert!(
        svc.get_record(&rec.id).await.unwrap().is_none(),
        "old metadata row must be deleted by the artifact.moved projection"
    );

    // Bytes preserved on the blob plane.
    let (_, bytes) = svc.get_blob(&promoted.id).await.unwrap();
    assert_eq!(bytes, b"promote me");
}
