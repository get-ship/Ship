//! Integration tests for ArtifactService.

use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::{ARTIFACTS_COLLECTION_ID, ArtifactService};
use crate::collections::query::QueryRequest;
use crate::data::DataService;
use crate::events::KernelRouter;

const TEST_SLUG: &str = "test-project";

fn test_kernel() -> Arc<Mutex<KernelRouter>> {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    crate::db::ensure_db_at(&ship_dir).unwrap();
    let ship_dir = Box::leak(Box::new(ship_dir));
    Arc::new(Mutex::new(KernelRouter::new(ship_dir.clone()).unwrap()))
}

fn setup() -> (ArtifactService, DataService, Arc<Mutex<KernelRouter>>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let svc = ArtifactService::new(&ship_dir, TEST_SLUG);
    let data = DataService::open(&ship_dir).unwrap();
    let kernel = test_kernel();
    (svc, data, kernel, tmp)
}

fn empty_query() -> QueryRequest {
    QueryRequest {
        collection: ARTIFACTS_COLLECTION_ID.to_string(),
        filter: BTreeMap::new(),
        sort: BTreeMap::new(),
        limit: 50,
        offset: 0,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn store_writes_blob_under_project_slug() {
    let (svc, mut data, kernel, _tmp) = setup();
    let record = svc
        .store(&mut data, &kernel, "hello.txt", "text/plain", b"hi there", None, None, None)
        .await
        .unwrap();

    assert_eq!(record.name, "hello.txt");
    assert_eq!(record.mime, "text/plain");
    assert_eq!(record.size, 8);
    assert!(record.path.starts_with("blobs/"));

    let blob_abs = svc.blob_path(&record.id);
    assert!(blob_abs.to_string_lossy().contains(&format!("artifacts/{TEST_SLUG}/blobs/")));
    let blob = std::fs::read(blob_abs).unwrap();
    assert_eq!(blob, b"hi there");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_returns_metadata_and_bytes() {
    let (svc, mut data, kernel, _tmp) = setup();
    let record = svc
        .store(&mut data, &kernel, "a.bin", "application/octet-stream", &[1, 2, 3], None, None, None)
        .await
        .unwrap();

    let (meta, bytes) = svc.get(&mut data, &record.id).unwrap().unwrap();
    assert_eq!(meta.id, record.id);
    assert_eq!(bytes, vec![1, 2, 3]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_missing_metadata_returns_none() {
    let (svc, mut data, _kernel, _tmp) = setup();
    assert!(svc.get(&mut data, "01HNO-SUCH-ID").unwrap().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_missing_blob_errors() {
    let (svc, mut data, kernel, _tmp) = setup();
    let record = svc
        .store(&mut data, &kernel, "x", "text/plain", b"x", None, None, None)
        .await
        .unwrap();

    std::fs::remove_file(svc.blob_path(&record.id)).unwrap();

    let err = svc.get(&mut data, &record.id).unwrap_err();
    assert!(format!("{err}").contains("blob missing"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_removes_blob_and_metadata() {
    let (svc, mut data, kernel, _tmp) = setup();
    let record = svc
        .store(&mut data, &kernel, "gone.txt", "text/plain", b"bye", None, None, None)
        .await
        .unwrap();
    let path = svc.blob_path(&record.id);
    assert!(path.exists());

    assert!(svc.delete(&mut data, &kernel, &record.id).await.unwrap());
    assert!(!path.exists());
    assert!(svc.get_record(&mut data, &record.id).unwrap().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_missing_returns_false() {
    let (svc, mut data, kernel, _tmp) = setup();
    assert!(!svc.delete(&mut data, &kernel, "nope").await.unwrap());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_returns_all_stored_artifacts() {
    let (svc, mut data, kernel, _tmp) = setup();
    for n in ["a.txt", "b.txt", "c.txt"] {
        svc.store(&mut data, &kernel, n, "text/plain", n.as_bytes(), None, None, None)
            .await
            .unwrap();
    }
    let results = svc.list(&mut data, empty_query()).unwrap();
    assert_eq!(results.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_filter_by_mime() {
    let (svc, mut data, kernel, _tmp) = setup();
    svc.store(&mut data, &kernel, "a", "text/plain", b"a", None, None, None).await.unwrap();
    svc.store(&mut data, &kernel, "b", "image/png", b"b", None, None, None).await.unwrap();

    let mut req = empty_query();
    req.filter.insert("mime".into(), serde_json::json!("image/png"));
    let results = svc.list(&mut data, req).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "b");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn roundtrip_store_list_get_delete() {
    let (svc, mut data, kernel, _tmp) = setup();
    let record = svc
        .store(&mut data, &kernel, "roundtrip.txt", "text/plain", b"payload",
               Some("notes"), Some("note-1"), Some("tester"))
        .await
        .unwrap();
    assert_eq!(record.collection_id.as_deref(), Some("notes"));
    assert_eq!(record.item_id.as_deref(), Some("note-1"));

    let listed = svc.list(&mut data, empty_query()).unwrap();
    assert_eq!(listed.len(), 1);

    let (meta, bytes) = svc.get(&mut data, &record.id).unwrap().unwrap();
    assert_eq!(meta.name, "roundtrip.txt");
    assert_eq!(bytes, b"payload");

    assert!(svc.delete(&mut data, &kernel, &record.id).await.unwrap());
    assert!(svc.list(&mut data, empty_query()).unwrap().is_empty());
}

#[test]
fn blob_path_uses_two_char_prefix_under_slug() {
    let tmp = tempfile::tempdir().unwrap();
    let svc = ArtifactService::new(tmp.path(), "demo-project");
    let path = svc.blob_path("01HABCDEFG");
    let s = path.to_string_lossy();
    assert!(s.contains("/artifacts/demo-project/blobs/01/01HABCDEFG"), "got: {s}");
}

#[test]
fn blobs_root_includes_project_slug() {
    let tmp = tempfile::tempdir().unwrap();
    let svc = ArtifactService::new(tmp.path(), "myproj-abc");
    let s = svc.blobs_dir().to_string_lossy().to_string();
    assert!(s.ends_with("artifacts/myproj-abc/blobs"), "got: {s}");
}
