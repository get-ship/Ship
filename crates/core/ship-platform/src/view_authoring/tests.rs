//! Integration tests for the view-authoring snapshot service.
//!
//! Exercises the round-trip path through the artifact store + collection,
//! plus deletion semantics used by the atomic-write rollback in
//! `view.write_source`.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::ArtifactService;
use crate::view_authoring::snapshot::{
    delete_snapshot, ensure_view_snapshots_collection, list_snapshots, read_snapshot,
    snapshot_view,
};

async fn fixture()
-> (tempfile::TempDir, ship_substrate::data::DataService, Arc<Mutex<ship_substrate::events::KernelRouter>>, ArtifactService) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let mut svc = ship_substrate::data::DataService::open(&ship_dir).unwrap();
    crate::artifacts::ensure_artifacts_collection(&mut svc).unwrap();
    ensure_view_snapshots_collection(&mut svc).unwrap();
    let kernel = Arc::new(Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let artifacts = ArtifactService::new(&ship_dir);
    (tmp, svc, kernel, artifacts)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_then_list_then_read_round_trips() {
    let (_tmp, mut svc, kernel, artifacts) = fixture().await;
    let meta = snapshot_view(
        &mut svc,
        &kernel,
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

    let (snaps, cursor) = list_snapshots(&mut svc, "dashboard", 10, None).unwrap();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].snapshot_id, meta.snapshot_id);
    assert!(cursor.is_none());

    let bytes = read_snapshot(&mut svc, &artifacts, &meta.snapshot_id).unwrap();
    assert_eq!(bytes, "<page>v1</page>");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_snapshots_filters_by_view_name() {
    let (_tmp, mut svc, kernel, artifacts) = fixture().await;
    snapshot_view(&mut svc, &kernel, &artifacts, "p", "a", "src-a", "t1", "agt", None).await.unwrap();
    snapshot_view(&mut svc, &kernel, &artifacts, "p", "b", "src-b", "t2", "agt", None).await.unwrap();

    let (snaps_a, _) = list_snapshots(&mut svc, "a", 10, None).unwrap();
    assert_eq!(snaps_a.len(), 1);
    assert_eq!(snaps_a[0].view_name, "a");

    let (snaps_b, _) = list_snapshots(&mut svc, "b", 10, None).unwrap();
    assert_eq!(snaps_b.len(), 1);
    assert_eq!(snaps_b[0].view_name, "b");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_snapshot_removes_row_and_blob() {
    let (_tmp, mut svc, kernel, artifacts) = fixture().await;
    let meta = snapshot_view(
        &mut svc, &kernel, &artifacts, "p", "v", "data", "t", "agent", None,
    )
    .await
    .unwrap();

    let removed = delete_snapshot(&mut svc, &kernel, &artifacts, &meta.snapshot_id)
        .await
        .unwrap();
    assert!(removed);
    let (snaps, _) = list_snapshots(&mut svc, "v", 10, None).unwrap();
    assert!(snaps.is_empty());

    // Second delete is a no-op.
    let removed_again =
        delete_snapshot(&mut svc, &kernel, &artifacts, &meta.snapshot_id).await.unwrap();
    assert!(!removed_again);

    // Reading the snapshot after delete fails.
    let err = read_snapshot(&mut svc, &artifacts, &meta.snapshot_id).unwrap_err();
    assert!(err.to_string().contains("not found"), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_snapshots_paginates_via_cursor() {
    let (_tmp, mut svc, kernel, artifacts) = fixture().await;
    for i in 0..5 {
        snapshot_view(
            &mut svc,
            &kernel,
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
    }
    let (page1, cursor) = list_snapshots(&mut svc, "v", 3, None).unwrap();
    assert_eq!(page1.len(), 3);
    assert!(cursor.is_some(), "cursor should be present when page is full");

    let (page2, cursor2) = list_snapshots(&mut svc, "v", 3, cursor.as_deref()).unwrap();
    assert_eq!(page2.len(), 2);
    assert!(cursor2.is_none(), "cursor exhausted on tail page");

    // No id appears on both pages.
    for s in &page2 {
        assert!(!page1.iter().any(|p| p.snapshot_id == s.snapshot_id));
    }
}
