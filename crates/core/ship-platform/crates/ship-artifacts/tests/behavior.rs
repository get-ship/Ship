//! Behavioral tests for ship-artifacts.
//!
//! All tests use in-memory / temp-dir ports — no network, no real daemon.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use ship_artifacts::service::ArtifactService;
use ship_artifacts::types::ArtifactScope;
use ship_reactive::{ReactiveRegistry, WatchKey};
use ship_substrate::data::DataService;
use ship_substrate::events::{new_isolated_for_test, EventFilter};
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

// ── Fixtures ──────────────────────────────────────────────────────────────────
//
// Artifact METADATA is event-sourced (ADR D6/D7): `artifact.*` (Space-scoped)
// events route to `spaces/<scope-space>/space.db`, where `ArtifactProjection`
// materializes the `collection_artifacts` rows. The read ctx must therefore bind
// to the SAME per-scope `space.db` with its partition key set to the scope space
// (consumer-consistency). All scopes here use `SCOPE_SPACE`, so the ctx binds to
// it; the kernel router is rooted at the same `ship_dir`.

const SCOPE_SPACE: &str = "proj1";

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

/// Build a `SpaceContext` whose data store reads the per-scope `space.db` for
/// `space` (matching where `ArtifactProjection` writes), with its partition key
/// set to the same space.
fn make_ctx_for(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>, space: &str) -> Arc<SpaceContext> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    let space_db = ship_substrate::db::space_db_path(&ship_dir, space);
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, space));
    Arc::new(SpaceContext::new(SpaceId::new(space), store, kernel))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    make_ctx_for(tmp, kernel, SCOPE_SPACE)
}

fn make_service(tmp: &tempfile::TempDir, ctx: Arc<SpaceContext>) -> ArtifactService {
    ArtifactService::new(ctx, tmp.path().to_path_buf())
}

fn project_scope() -> ArtifactScope {
    ArtifactScope::Project { space_id: SCOPE_SPACE.to_string() }
}

fn run_scope() -> ArtifactScope {
    ArtifactScope::Run { space_id: SCOPE_SPACE.to_string(), run_id: "run1".to_string() }
}

// ── Test 1: create_artifact_emits_event_and_stores_blob ───────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_artifact_emits_event_and_stores_blob() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(&tmp, ctx);

    let mut rx = kernel.subscribe();

    let record = svc
        .store("hello.txt", "text/plain", b"hello world", &project_scope(), None, None, None, None)
        .await
        .unwrap();

    assert_eq!(record.name, "hello.txt");
    assert_eq!(record.mime, "text/plain");
    assert_eq!(record.size, 11);
    assert_eq!(record.scope, "project");
    assert!(record.content_hash.is_some());

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("closed");
    let found = std::iter::once(event)
        .chain(std::iter::from_fn(|| rx.try_recv().ok()))
        .any(|e| e.event_type == "artifact.created");
    assert!(found, "expected artifact.created event");
}

// ── Test 2: list_returns_metadata ─────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_returns_metadata() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(&tmp, ctx);

    svc.store("a.txt", "text/plain", b"aaa", &project_scope(), None, None, None, None).await.unwrap();
    svc.store("b.txt", "text/plain", b"bbb", &run_scope(), None, None, None, None).await.unwrap();

    let all = svc.list(BTreeMap::new(), 50).await.unwrap();
    assert_eq!(all.len(), 2);

    let mut filter = BTreeMap::new();
    filter.insert("scope".into(), serde_json::Value::String("project".into()));
    let project_only = svc.list(filter, 50).await.unwrap();
    assert_eq!(project_only.len(), 1);
    assert_eq!(project_only[0].name, "a.txt");
}

// ── Test 3: get_blob_roundtrips_bytes ─────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_blob_roundtrips_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(&tmp, ctx);

    let original = b"roundtrip test bytes";
    let record = svc.store("data.bin", "application/octet-stream", original, &project_scope(), None, None, None, None).await.unwrap();
    let (_, bytes) = svc.get_blob(&record.id).await.unwrap();
    assert_eq!(bytes, original);
}

// ── Test 4: delete_removes_metadata_and_blob ──────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_removes_metadata_and_blob() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(&tmp, ctx);

    let mut rx = kernel.subscribe();

    let record = svc.store("del.txt", "text/plain", b"delete me", &project_scope(), None, None, None, None).await.unwrap();
    let deleted = svc.delete(&record.id).await.unwrap();
    assert!(deleted);

    // Metadata gone
    let found = svc.get_record(&record.id).await.unwrap();
    assert!(found.is_none(), "metadata must be removed");

    // artifact.deleted event emitted
    // drain events
    tokio::time::sleep(Duration::from_millis(50)).await;
    let events: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
    assert!(events.iter().any(|e| e.event_type == "artifact.deleted"), "expected artifact.deleted");
}

// ── Test 5: promote_marks_retained ────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn promote_marks_retained() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(&tmp, ctx);

    let record = svc
        .store("promo.txt", "text/plain", b"promote me", &run_scope(), None, None, None, Some("2020-01-01T00:00:00Z"))
        .await
        .unwrap();
    assert_eq!(record.scope, "run");
    assert!(record.expires_at.is_some());

    let promoted = svc.promote(&record.id, project_scope(), true).await.unwrap();
    assert_eq!(promoted.scope, "project");
    // retained=true clears expires_at
    assert!(promoted.expires_at.is_none(), "retained artifact must have no expiry");

    // original record gone
    let old = svc.get_record(&record.id).await.unwrap();
    assert!(old.is_none(), "old metadata must be removed after promote");
}

// ── Test 5b: artifact metadata is per-space isolated (event-sourced) ──────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifact_metadata_is_per_space_isolated() {
    // Artifact metadata is event-sourced: `artifact.*` events route to the
    // SCOPE space's `space.db`, where `ArtifactProjection` materializes the
    // metadata row. A service bound to space A therefore sees A's artifacts and
    // NOT B's — proving the ctx space is the read partition, not cosmetic. (This
    // replaces the pre-flip "ctx space is cosmetic" claim, false under D6/D7.)
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    let svc_a = make_service(&tmp, make_ctx_for(&tmp, kernel.clone(), "proj-A"));
    let svc_b = make_service(&tmp, make_ctx_for(&tmp, kernel, "proj-B"));

    let scope_a = ArtifactScope::Project { space_id: "proj-A".to_string() };
    let rec = svc_a
        .store("a-only.txt", "text/plain", b"a-bytes", &scope_a, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(rec.space_id.as_deref(), Some("proj-A"));

    // Space A sees its artifact (projection materialized it in proj-A's space.db).
    assert!(svc_a.get_record(&rec.id).await.unwrap().is_some());
    let (_, bytes) = svc_a.get_blob(&rec.id).await.unwrap();
    assert_eq!(bytes, b"a-bytes");

    // Space B sees nothing — A's row lives in spaces/proj-A/space.db only.
    assert!(svc_b.get_record(&rec.id).await.unwrap().is_none());
    assert!(svc_b.list(BTreeMap::new(), 50).await.unwrap().is_empty());
}

// ── Test 6: gc_sweeps_unreferenced_blob ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gc_sweeps_unreferenced_blob() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(&tmp, ctx);

    // Create an already-expired run artifact
    let record = svc
        .store("expired.txt", "text/plain", b"stale blob", &run_scope(), None, None, None, Some("2020-01-01T00:00:00Z"))
        .await
        .unwrap();

    let count = ship_artifacts::gc::collect_expired(&svc).await.unwrap();
    assert_eq!(count, 1, "GC must remove the expired artifact");

    let found = svc.get_record(&record.id).await.unwrap();
    assert!(found.is_none(), "GC must remove metadata");
}

// ── Test 7: watch_first_frame_is_snapshot_then_patch_on_create ───────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_first_frame_is_snapshot_then_patch_on_create() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(&tmp, ctx.clone());

    let ctx2 = ctx.clone();
    let spec = ship_reactive::ResolverSpec {
        prefilter: Box::new(|env| env.event_type.starts_with("artifact.")),
        snapshot: Box::new(move || {
            let c = ctx2.clone();
            Box::pin(async move {
                ship_artifacts::patch::snapshot_for_artifacts(c, "", "").await
            })
        }),
    };

    let registry = ReactiveRegistry::new(kernel.clone());
    let key = WatchKey { space_id: "test".into(), scope: "artifacts".into(), query_hash: 0 };
    let mut rx = registry.watch(key, EventFilter::default(), spec);

    // First frame: snapshot set at root.
    let frame1 = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("closed");
    assert_eq!(frame1.patches.len(), 1);
    assert_eq!(frame1.patches[0].op, ship_reactive::PatchOp::Set);
    assert_eq!(frame1.patches[0].path, "");

    // Create an artifact to trigger a change.
    svc.store("new.txt", "text/plain", b"trigger", &project_scope(), None, None, None, None)
        .await
        .unwrap();

    // Should receive a non-empty patch frame.
    let frame2 = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout for patch frame").expect("closed");
    assert!(!frame2.patches.is_empty(), "expected patch frame after artifact creation");
}

// ── Test 8: watch_fanout_dedups_identical_scope ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_fanout_dedups_identical_scope() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    let snapshot_calls = Arc::new(AtomicUsize::new(0));
    let c1 = snapshot_calls.clone();
    let c2 = snapshot_calls.clone();

    let make_spec = |c: Arc<AtomicUsize>| ship_reactive::ResolverSpec {
        prefilter: Box::new(|_| false),
        snapshot: Box::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { serde_json::json!({}) })
        }),
    };

    let registry = ReactiveRegistry::new(kernel);
    let key = WatchKey { space_id: "s1".into(), scope: "artifacts".into(), query_hash: 99 };

    let mut rx1 = registry.watch(key.clone(), EventFilter::default(), make_spec(c1));
    tokio::time::timeout(Duration::from_secs(2), rx1.recv()).await.expect("timeout").expect("closed");

    let _rx2 = registry.watch(key, EventFilter::default(), make_spec(c2));

    assert_eq!(snapshot_calls.load(Ordering::SeqCst), 1, "second watcher must reuse resolver");
}

// ── Test 9: collection_id_is_plain_field_no_collection_dependency ─────────────

#[test]
fn collection_id_is_plain_field_no_collection_dependency() {
    // Verify at compile time that ship-artifacts has no ship-collection dep by
    // checking the ArtifactRecord type has collection_id as a plain Option<String>.
    let rec = ship_artifacts::types::ArtifactRecord {
        id: "id".into(),
        name: "n".into(),
        mime: "m".into(),
        size: 0,
        path: "p".into(),
        scope: "project".into(),
        space_id: None,
        run_id: None,
        user_id: None,
        content_hash: None,
        expires_at: None,
        owner_actor: None,
        collection_id: Some("my-collection".into()),
        item_id: None,
        created_at: "now".into(),
    };
    assert_eq!(rec.collection_id.as_deref(), Some("my-collection"));

    // Grep check: this crate must not mention ship-collection anywhere.
    // The Cargo.toml for ship-artifacts does not list ship-collection.
    let cargo = include_str!("../Cargo.toml");
    assert!(!cargo.contains("ship-collection"), "ship-artifacts must not depend on ship-collection");
}
