//! Integration tests for the view-authoring snapshot service.
//!
//! Exercises the round-trip path through the artifact store + collection,
//! plus deletion semantics used by the atomic-write rollback in
//! `view.write_source`. Post-C-2 these run over a `SpaceContext` + the new
//! `ship-artifacts` / `ship-collection` services.

use std::sync::Arc;
use tokio::sync::Mutex;

use ship_artifacts::service::ArtifactService;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

use crate::view_authoring::snapshot::{
    delete_snapshot, ensure_view_snapshots_collection, list_snapshots, read_snapshot,
    snapshot_view,
};

/// Register the domain projections needed by the snapshot service.
///
/// Both `ArtifactProjection` (artifact metadata) and `CollectionItemProjection`
/// (collection row materialisation) moved from the L0 substrate to their
/// respective L1 crates at keystone-A (2026-06-10).  In production
/// `ship_platform::install_defaults()` wires them into the default bus; in
/// tests that don't pull in the full platform we install them here manually,
/// mirroring what ship-artifacts/ship-collection do in their own test helpers.
///
/// Must be called before any `KernelRouter` is constructed so the factories
/// are present when `SqliteEventStore::default_bus` reads `extra_projections`.
fn install_projections() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![
                Box::new(ship_artifacts::metadata_projection::ArtifactProjection)
                    as Box<dyn ship_substrate::projections::Projection>,
                Box::new(ship_collection::item_projection::CollectionItemProjection)
                    as Box<dyn ship_substrate::projections::Projection>,
            ]
        }));
    });
}

async fn fixture() -> (tempfile::TempDir, Arc<SpaceContext>, ArtifactService) {
    // Install domain projections before constructing the kernel so that
    // SqliteEventStore::default_bus picks them up when it opens each scope db.
    install_projections();

    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    // Event-sourced collections: the projection writes into the per-space
    // `space.db`, so the read store must open that exact file (not the legacy
    // shared `data.db` via `open`) or reads never see projected rows.
    let data_svc = ship_substrate::data::DataService::open_at(
        &ship_substrate::db::space_db_path(&ship_dir, "global"),
    )
    .unwrap();
    let kernel = Arc::new(Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let store = Arc::new(SingleBackendDataStore::new(data_svc, "global"));
    let kernel_log = Arc::new(KernelLog::new(kernel));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new("global"), store, kernel_log));
    ensure_view_snapshots_collection(&ctx).await.unwrap();
    let artifacts = ArtifactService::new(ctx.clone(), ship_dir.clone());
    (tmp, ctx, artifacts)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_then_list_then_read_round_trips() {
    let (_tmp, ctx, artifacts) = fixture().await;
    let meta = snapshot_view(
        ctx.clone(),
        &artifacts,
        "project-x",
        "dashboard",
        "<page>v1</page>",
        "turn-1",
        "agent-1",
        Some("first version"),
    )
    .await
    .unwrap();

    assert_eq!(meta.view_name, "dashboard");
    assert_eq!(meta.turn_id, "turn-1");
    assert_eq!(meta.label.as_deref(), Some("first version"));
    assert!(!meta.snapshot_id.is_empty());
    assert!(!meta.content_hash.is_empty());

    let (snaps, cursor) = list_snapshots(ctx.clone(), "dashboard", 10, None).await.unwrap();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].snapshot_id, meta.snapshot_id);
    assert!(cursor.is_none());

    let bytes = read_snapshot(ctx, &artifacts, &meta.snapshot_id).await.unwrap();
    assert_eq!(bytes, "<page>v1</page>");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_snapshots_filters_by_view_name() {
    let (_tmp, ctx, artifacts) = fixture().await;
    snapshot_view(ctx.clone(), &artifacts, "p", "a", "src-a", "t1", "agt", None).await.unwrap();
    snapshot_view(ctx.clone(), &artifacts, "p", "b", "src-b", "t2", "agt", None).await.unwrap();

    let (snaps_a, _) = list_snapshots(ctx.clone(), "a", 10, None).await.unwrap();
    assert_eq!(snaps_a.len(), 1);
    assert_eq!(snaps_a[0].view_name, "a");

    let (snaps_b, _) = list_snapshots(ctx, "b", 10, None).await.unwrap();
    assert_eq!(snaps_b.len(), 1);
    assert_eq!(snaps_b[0].view_name, "b");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_snapshot_removes_row_and_blob() {
    let (_tmp, ctx, artifacts) = fixture().await;
    let meta = snapshot_view(ctx.clone(), &artifacts, "p", "v", "data", "t", "agent", None)
        .await
        .unwrap();

    let removed = delete_snapshot(ctx.clone(), &artifacts, &meta.snapshot_id).await.unwrap();
    assert!(removed);
    let (snaps, _) = list_snapshots(ctx.clone(), "v", 10, None).await.unwrap();
    assert!(snaps.is_empty());

    // Second delete is a no-op.
    let removed_again =
        delete_snapshot(ctx.clone(), &artifacts, &meta.snapshot_id).await.unwrap();
    assert!(!removed_again);

    // Reading the snapshot after delete fails.
    let err = read_snapshot(ctx, &artifacts, &meta.snapshot_id).await.unwrap_err();
    assert!(err.to_string().contains("not found"), "got: {err}");
}

/// Mirror of the `view.write_source` atomic-rollback semantics.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn atomic_write_rollback_purges_snapshot_on_failure() {
    let (_tmp, ctx, artifacts) = fixture().await;
    let candidate = snapshot_view(
        ctx.clone(), &artifacts, "p", "v", "broken-src", "t1", "agt", Some("bad write"),
    )
    .await
    .unwrap();

    let removed = delete_snapshot(ctx.clone(), &artifacts, &candidate.snapshot_id).await.unwrap();
    assert!(removed, "rollback must remove the candidate snapshot");

    let (snaps, _) = list_snapshots(ctx, "v", 10, None).await.unwrap();
    assert!(
        snaps.is_empty(),
        "list_snapshots must not return the rolled-back snapshot, got {snaps:?}"
    );
}

/// The atomic happy path: snapshot the new source, leave it in place on success.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn atomic_write_happy_path_persists_snapshot_on_success() {
    let (_tmp, ctx, artifacts) = fixture().await;
    let meta = snapshot_view(
        ctx.clone(), &artifacts, "p", "v", "good-src", "t1", "agt", Some("good write"),
    )
    .await
    .unwrap();

    let (snaps, _) = list_snapshots(ctx.clone(), "v", 10, None).await.unwrap();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].snapshot_id, meta.snapshot_id);
    let bytes = read_snapshot(ctx, &artifacts, &meta.snapshot_id).await.unwrap();
    assert_eq!(bytes, "good-src");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_snapshots_paginates_via_cursor() {
    let (_tmp, ctx, artifacts) = fixture().await;
    for i in 0..5 {
        snapshot_view(
            ctx.clone(),
            &artifacts,
            "p",
            "v",
            &format!("src-{i}"),
            &format!("t{i}"),
            "agent",
            None,
        )
        .await
        .unwrap();
        // Snapshots are paged by `created_at desc`; nudge the clock so the
        // rfc3339 timestamps are strictly ordered within the test's tight loop.
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    }
    let (page1, cursor) = list_snapshots(ctx.clone(), "v", 3, None).await.unwrap();
    assert_eq!(page1.len(), 3);
    assert!(cursor.is_some(), "cursor should be present when page is full");

    let (page2, cursor2) = list_snapshots(ctx, "v", 3, cursor.as_deref()).await.unwrap();
    assert_eq!(page2.len(), 2);
    assert!(cursor2.is_none(), "cursor exhausted on tail page");

    // No id appears on both pages.
    for s in &page2 {
        assert!(!page1.iter().any(|p| p.snapshot_id == s.snapshot_id));
    }
}
