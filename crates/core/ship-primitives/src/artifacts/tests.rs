//! Integration tests for ArtifactService — three-scope model.

use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::{
    ARTIFACTS_COLLECTION_ID, ArtifactBackend, ArtifactScope, ArtifactService, LocalFsBackend, sha256_hex,
};
use crate::collections::query::QueryRequest;
use ship_substrate::data::DataService;
use ship_substrate::events::KernelRouter;

fn test_kernel() -> Arc<Mutex<KernelRouter>> {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let ship_dir = Box::leak(Box::new(ship_dir));
    Arc::new(Mutex::new(KernelRouter::new(ship_dir.clone()).unwrap()))
}

fn setup() -> (ArtifactService, DataService, Arc<Mutex<KernelRouter>>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let svc = ArtifactService::new(&ship_dir);
    let data = DataService::open(&ship_dir).unwrap();
    let kernel = test_kernel();
    (svc, data, kernel, tmp)
}

fn project_scope(slug: &str) -> ArtifactScope {
    ArtifactScope::Project { project_id: slug.to_string() }
}

fn run_scope(slug: &str, run: &str) -> ArtifactScope {
    ArtifactScope::Run { project_id: slug.to_string(), run_id: run.to_string() }
}

fn user_scope(uid: &str) -> ArtifactScope {
    ArtifactScope::User { user_id: uid.to_string() }
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

// ── store / get / delete in each scope ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn store_get_delete_project_scope() {
    let (svc, mut data, kernel, _tmp) = setup();
    let scope = project_scope("myproj");
    let rec = svc
        .store(&mut data, &kernel, "hello.txt", "text/plain", b"hi", &scope, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(rec.scope, "project");
    assert_eq!(rec.project_id.as_deref(), Some("myproj"));
    assert_eq!(rec.name, "hello.txt");
    assert!(rec.content_hash.is_some());

    let (meta, bytes) = svc.get(&mut data, &rec.id).unwrap().unwrap();
    assert_eq!(meta.id, rec.id);
    assert_eq!(bytes, b"hi");

    assert!(svc.delete(&mut data, &kernel, &rec.id).await.unwrap());
    assert!(svc.get_record(&mut data, &rec.id).unwrap().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn store_get_delete_run_scope() {
    let (svc, mut data, kernel, _tmp) = setup();
    let scope = run_scope("proj", "run-001");
    let rec = svc
        .store(&mut data, &kernel, "output.json", "application/json", b"{}", &scope, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(rec.scope, "run");
    assert_eq!(rec.project_id.as_deref(), Some("proj"));
    assert_eq!(rec.run_id.as_deref(), Some("run-001"));

    let (_, bytes) = svc.get(&mut data, &rec.id).unwrap().unwrap();
    assert_eq!(bytes, b"{}");

    assert!(svc.delete(&mut data, &kernel, &rec.id).await.unwrap());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn store_get_delete_user_scope() {
    let (svc, mut data, kernel, _tmp) = setup();
    let scope = user_scope("alice");
    let rec = svc
        .store(&mut data, &kernel, "note.md", "text/markdown", b"# Hi", &scope, None, None, Some("alice"), None)
        .await
        .unwrap();

    assert_eq!(rec.scope, "user");
    assert_eq!(rec.user_id.as_deref(), Some("alice"));
    assert_eq!(rec.owner_actor.as_deref(), Some("alice"));

    let (_, bytes) = svc.get(&mut data, &rec.id).unwrap().unwrap();
    assert_eq!(bytes, b"# Hi");

    assert!(svc.delete(&mut data, &kernel, &rec.id).await.unwrap());
}

// ── promote between scopes ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn promote_run_to_project() {
    let (svc, mut data, kernel, _tmp) = setup();
    let scope = run_scope("proj", "run-001");
    let rec = svc
        .store(&mut data, &kernel, "artifact.bin", "application/octet-stream", b"data", &scope, None, None, None, None)
        .await
        .unwrap();

    let promoted = svc
        .promote(&mut data, &kernel, &rec.id, project_scope("proj"))
        .await
        .unwrap();

    assert_eq!(promoted.scope, "project");
    assert_eq!(promoted.project_id.as_deref(), Some("proj"));
    assert!(promoted.run_id.is_none());

    // Old id is gone, new id exists.
    assert!(svc.get_record(&mut data, &rec.id).unwrap().is_none());
    let (_, bytes) = svc.get(&mut data, &promoted.id).unwrap().unwrap();
    assert_eq!(bytes, b"data");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn promote_project_to_user() {
    let (svc, mut data, kernel, _tmp) = setup();
    let rec = svc
        .store(&mut data, &kernel, "report.pdf", "application/pdf", b"pdf", &project_scope("proj"), None, None, None, None)
        .await
        .unwrap();

    let promoted = svc
        .promote(&mut data, &kernel, &rec.id, user_scope("bob"))
        .await
        .unwrap();

    assert_eq!(promoted.scope, "user");
    assert_eq!(promoted.user_id.as_deref(), Some("bob"));

    let (_, bytes) = svc.get(&mut data, &promoted.id).unwrap().unwrap();
    assert_eq!(bytes, b"pdf");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn promote_run_to_user() {
    let (svc, mut data, kernel, _tmp) = setup();
    let rec = svc
        .store(&mut data, &kernel, "x.bin", "application/octet-stream", b"x", &run_scope("p", "r"), None, None, None, None)
        .await
        .unwrap();

    let promoted = svc.promote(&mut data, &kernel, &rec.id, user_scope("alice")).await.unwrap();
    assert_eq!(promoted.scope, "user");
}

// ── content hash and dedup ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn content_hash_is_computed() {
    let (svc, mut data, kernel, _tmp) = setup();
    let bytes = b"deterministic content";
    let expected = sha256_hex(bytes);
    let rec = svc
        .store(&mut data, &kernel, "f.txt", "text/plain", bytes, &project_scope("p"), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(rec.content_hash.as_deref(), Some(expected.as_str()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dedup_same_hash_same_scope_produces_new_row() {
    let (svc, mut data, kernel, _tmp) = setup();
    let scope = project_scope("p");
    let bytes = b"identical bytes";
    let r1 = svc.store(&mut data, &kernel, "a.txt", "text/plain", bytes, &scope, None, None, None, None).await.unwrap();
    let r2 = svc.store(&mut data, &kernel, "b.txt", "text/plain", bytes, &scope, None, None, None, None).await.unwrap();
    // Different IDs, same hash, same on-disk blob path (dedup reuses bytes).
    assert_ne!(r1.id, r2.id);
    assert_eq!(r1.content_hash, r2.content_hash);
    assert_eq!(r1.path, r2.path, "dedup should point both rows at the same blob file");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dedup_delete_first_row_keeps_blob_for_second() {
    let (svc, mut data, kernel, _tmp) = setup();
    let scope = project_scope("p");
    let bytes = b"shared payload";
    let r1 = svc.store(&mut data, &kernel, "a.txt", "text/plain", bytes, &scope, None, None, None, None).await.unwrap();
    let r2 = svc.store(&mut data, &kernel, "b.txt", "text/plain", bytes, &scope, None, None, None, None).await.unwrap();
    assert_eq!(r1.path, r2.path, "sanity: dedup'd to the same path");

    // Delete the first row. The blob must survive because r2 still points at it.
    svc.delete(&mut data, &kernel, &r1.id).await.unwrap();
    let (_rec, fetched) = svc.get(&mut data, &r2.id).unwrap()
        .expect("r2 still resolvable after r1 delete");
    assert_eq!(fetched, bytes);

    // Deleting the last row drops the blob.
    svc.delete(&mut data, &kernel, &r2.id).await.unwrap();
    let re_fetch = svc.get(&mut data, &r2.id).unwrap();
    assert!(re_fetch.is_none(), "metadata gone");
}

// ── GC ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gc_removes_expired_run_scope() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let svc = ArtifactService::new(&ship_dir);
    let mut data = DataService::open(&ship_dir).unwrap();
    let kernel = test_kernel();

    // Store with a past expires_at.
    let past = "2020-01-01T00:00:00Z";
    let scope = run_scope("proj", "run-001");
    let rec = svc
        .store(&mut data, &kernel, "tmp.txt", "text/plain", b"x", &scope, None, None, None, Some(past))
        .await
        .unwrap();

    let gc_count = crate::artifacts::gc::collect_expired(&ship_dir, &mut data, &kernel)
        .await
        .unwrap();
    assert_eq!(gc_count, 1);
    assert!(svc.get_record(&mut data, &rec.id).unwrap().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gc_skips_non_expired() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let svc = ArtifactService::new(&ship_dir);
    let mut data = DataService::open(&ship_dir).unwrap();
    let kernel = test_kernel();

    let future = "2099-01-01T00:00:00Z";
    let scope = run_scope("proj", "run-002");
    svc.store(&mut data, &kernel, "keep.txt", "text/plain", b"y", &scope, None, None, None, Some(future))
        .await
        .unwrap();

    let gc_count = crate::artifacts::gc::collect_expired(&ship_dir, &mut data, &kernel)
        .await
        .unwrap();
    assert_eq!(gc_count, 0);
}

// ── blob path structure ───────────────────────────────────────────────────────

#[test]
fn project_scope_blob_path_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = LocalFsBackend::new(tmp.path().to_path_buf());
    let scope = ArtifactScope::Project { project_id: "demo".to_string() };
    let path = backend.blob_path(&scope, "01HABCDEF");
    let s = path.to_string_lossy();
    assert!(s.contains("/artifacts/demo/blobs/01/01HABCDEF"), "got: {s}");
}

#[test]
fn run_scope_blob_path_structure() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = LocalFsBackend::new(tmp.path().to_path_buf());
    let scope = ArtifactScope::Run { project_id: "p".to_string(), run_id: "r1".to_string() };
    let path = backend.blob_path(&scope, "01HABCDEF");
    let s = path.to_string_lossy();
    assert!(s.contains("/artifacts/p/runs/r1/01/01HABCDEF"), "got: {s}");
}

#[test]
fn user_scope_blob_path_structure() {
    let tmp = tempfile::tempdir().unwrap();
    let backend = LocalFsBackend::new(tmp.path().to_path_buf());
    let scope = ArtifactScope::User { user_id: "alice".to_string() };
    let path = backend.blob_path(&scope, "01HABCDEF");
    let s = path.to_string_lossy();
    assert!(s.contains("/artifacts/users/alice/01/01HABCDEF"), "got: {s}");
}

// ── list / filter ────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_filter_by_scope() {
    let (svc, mut data, kernel, _tmp) = setup();
    svc.store(&mut data, &kernel, "a", "text/plain", b"a", &project_scope("p"), None, None, None, None).await.unwrap();
    svc.store(&mut data, &kernel, "b", "text/plain", b"b", &run_scope("p", "r"), None, None, None, None).await.unwrap();

    let mut req = empty_query();
    req.filter.insert("scope".into(), serde_json::json!("project"));
    let results = svc.list(&mut data, req).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "a");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_missing_returns_false() {
    let (svc, mut data, kernel, _tmp) = setup();
    assert!(!svc.delete(&mut data, &kernel, "nope").await.unwrap());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_missing_metadata_returns_none() {
    let (svc, mut data, _k, _tmp) = setup();
    assert!(svc.get(&mut data, "no-such-id").unwrap().is_none());
}

// ── backfill: legacy rows without scope resolve as project ─────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn backfill_legacy_null_scope_rows_resolve_as_project() {
    use crate::artifacts::schema::ensure_artifacts_collection;
    use crate::collections::store::{create_item, open_data_db};
    use crate::artifacts::schema::artifacts_schema;

    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();

    let mut data = DataService::open(&ship_dir).unwrap();

    // Ensure the collection table exists before inserting the raw row.
    ensure_artifacts_collection(&mut data).unwrap();

    // Simulate a legacy row: insert directly without 'scope'.
    let schema = artifacts_schema();
    let bytes = b"legacy content";
    let hash = sha256_hex(bytes);
    let legacy_id = ship_substrate::gen_ulid();

    // Write the blob at the project-scope path manually.
    let svc = ArtifactService::new(&ship_dir);
    let backend = LocalFsBackend::new(ship_dir.clone());
    let project_scope = ArtifactScope::Project { project_id: "legacy-proj".to_string() };
    backend.put(&project_scope, &legacy_id, bytes).unwrap();
    let rel_path = backend.rel_blob_path(&project_scope, &legacy_id);

    let data_json = serde_json::json!({
        "name": "legacy.txt",
        "mime": "text/plain",
        "size": bytes.len(),
        "path": rel_path,
        "project_id": "legacy-proj",
        "content_hash": hash,
        // Note: no "scope" field — simulates a row written by the old service
    });
    create_item(&mut data, &schema, &legacy_id, &data_json, None).unwrap();

    // Now call ensure_artifacts_collection again to trigger backfill.
    ensure_artifacts_collection(&mut data).unwrap();

    // The backfill should have set scope = 'project' for the legacy row.
    let record = svc.get_record(&mut data, &legacy_id).unwrap().unwrap();
    assert_eq!(record.scope, "project", "backfilled legacy row should have scope=project");
    assert_eq!(record.project_id.as_deref(), Some("legacy-proj"));
}
