/// Integration tests — artifact store / get / delete / patch / list.
///
/// Each test:
///   1. Spins an ephemeral project via `setup()`.
///   2. Calls the service method under test.
///   3. Asserts the return value.
///   4. For event-emit tests, queries the kernel's own event store via
///      `SqliteEventStore::new_at(ship_dir)` — the same DB that KernelRouter
///      writes to (not the per-thread auto-isolated store, which is a
///      different path from the kernel's base_dir).
use anyhow::Result;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::events::filter::EventFilter;
use ship_types::event_types::{ARTIFACT_CREATED, ARTIFACT_DELETED, ARTIFACT_UPDATED};
use tempfile::tempdir;

use crate::artifacts::{ArtifactScope, ArtifactService, sha256_hex};
use crate::collections::query::QueryRequest;
use crate::artifacts::ARTIFACTS_COLLECTION_ID;
use ship_substrate::data::DataService;
use ship_substrate::events::KernelRouter;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_kernel(ship_dir: &std::path::Path) -> Arc<Mutex<KernelRouter>> {
    ship_substrate::db::ensure_db_at(ship_dir).unwrap();
    let owned: PathBuf = ship_dir.to_path_buf();
    let leaked = Box::leak(Box::new(owned));
    Arc::new(Mutex::new(KernelRouter::new(leaked.clone()).unwrap()))
}

/// Returns (svc, data, kernel, ship_dir, _tmp_keep_alive).
fn setup() -> (ArtifactService, DataService, Arc<Mutex<KernelRouter>>, PathBuf, tempfile::TempDir) {
    let tmp = tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    crate::install_defaults();
    let svc = ArtifactService::new(&ship_dir);
    let data = DataService::open(&ship_dir).unwrap();
    let kernel = make_kernel(&ship_dir);
    (svc, data, kernel, ship_dir, tmp)
}

fn project_scope(slug: &str) -> ArtifactScope {
    ArtifactScope::Project { space_id: slug.to_string() }
}

fn run_scope(slug: &str, run: &str) -> ArtifactScope {
    ArtifactScope::Run { space_id: slug.to_string(), run_id: run.to_string() }
}

fn user_scope(uid: &str) -> ArtifactScope {
    ArtifactScope::User { user_id: uid.to_string() }
}

/// Query events from the same DB the kernel writes to.
/// KernelRouter uses `SqliteEventStore::new_at(&base_dir)` — so must we.
fn query_events_at(
    ship_dir: &std::path::Path,
    event_type: &str,
) -> Vec<ship_types::envelope::EventEnvelope> {
    let store = SqliteEventStore::new_at(ship_dir).expect("open test event store at ship_dir");
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .expect("query events")
}

// ── store — project scope ─────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_store_project_scope_creates_record() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("myproj");
    let rec = svc
        .store(&mut data, &kernel, "hello.txt", "text/plain", b"hi", &scope,
               None, None, None, None)
        .await?;

    assert_eq!(rec.scope, "project");
    assert_eq!(rec.space_id.as_deref(), Some("myproj"));
    assert_eq!(rec.name, "hello.txt");
    assert_eq!(rec.mime, "text/plain");
    assert_eq!(rec.size, 2);
    assert!(rec.content_hash.is_some());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_store_project_emits_artifact_created() -> Result<()> {
    let (svc, mut data, kernel, ship_dir, _tmp) = setup();
    let scope = project_scope("evt-proj");
    let rec = svc
        .store(&mut data, &kernel, "evt.txt", "text/plain", b"event", &scope,
               None, None, None, None)
        .await?;

    let events = query_events_at(&ship_dir, ARTIFACT_CREATED);
    assert!(
        events.iter().any(|e| e.entity_id == rec.id),
        "expected artifact.created event with entity_id={}; got {events:?}", rec.id
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_store_run_scope_sets_run_id() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = run_scope("proj", "run-001");
    let rec = svc
        .store(&mut data, &kernel, "output.json", "application/json", b"{}", &scope,
               None, None, None, None)
        .await?;

    assert_eq!(rec.scope, "run");
    assert_eq!(rec.space_id.as_deref(), Some("proj"));
    assert_eq!(rec.run_id.as_deref(), Some("run-001"));
    assert!(rec.user_id.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_store_user_scope_sets_user_id() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = user_scope("alice");
    let rec = svc
        .store(&mut data, &kernel, "note.md", "text/markdown", b"# Hi", &scope,
               None, None, Some("alice"), None)
        .await?;

    assert_eq!(rec.scope, "user");
    assert_eq!(rec.user_id.as_deref(), Some("alice"));
    assert_eq!(rec.owner_actor.as_deref(), Some("alice"));
    assert!(rec.run_id.is_none());
    assert!(rec.space_id.is_none());
    Ok(())
}

// ── get ───────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_get_returns_metadata_and_bytes() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("g");
    let rec = svc
        .store(&mut data, &kernel, "data.bin", "application/octet-stream", b"CONTENT",
               &scope, None, None, None, None)
        .await?;

    let (meta, bytes) = svc.get(&mut data, &rec.id)?.unwrap();
    assert_eq!(meta.id, rec.id);
    assert_eq!(meta.name, "data.bin");
    assert_eq!(bytes, b"CONTENT");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_get_missing_returns_none() -> Result<()> {
    let (svc, mut data, _k, _dir, _tmp) = setup();
    assert!(svc.get(&mut data, "nonexistent-id")?.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_get_record_returns_metadata_only() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("gr");
    let rec = svc
        .store(&mut data, &kernel, "meta.txt", "text/plain", b"x", &scope,
               None, None, None, None)
        .await?;

    let record = svc.get_record(&mut data, &rec.id)?.unwrap();
    assert_eq!(record.id, rec.id);
    assert_eq!(record.name, "meta.txt");
    Ok(())
}

// ── delete ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_delete_removes_record_and_blob() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("del");
    let rec = svc
        .store(&mut data, &kernel, "bye.txt", "text/plain", b"bye", &scope,
               None, None, None, None)
        .await?;

    assert!(svc.delete(&mut data, &kernel, &rec.id).await?);
    assert!(svc.get_record(&mut data, &rec.id)?.is_none());
    assert!(svc.get(&mut data, &rec.id)?.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_delete_emits_artifact_deleted() -> Result<()> {
    let (svc, mut data, kernel, ship_dir, _tmp) = setup();
    let scope = project_scope("del-evt");
    let rec = svc
        .store(&mut data, &kernel, "gone.txt", "text/plain", b"bye", &scope,
               None, None, None, None)
        .await?;
    let id = rec.id.clone();

    svc.delete(&mut data, &kernel, &id).await?;

    let events = query_events_at(&ship_dir, ARTIFACT_DELETED);
    assert!(
        events.iter().any(|e| e.entity_id == id),
        "expected artifact.deleted event with entity_id={id}; got {events:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_delete_missing_returns_false() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    assert!(!svc.delete(&mut data, &kernel, "no-such-id").await?);
    Ok(())
}

// ── patch ─────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_patch_updates_name() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("patch");
    let rec = svc
        .store(&mut data, &kernel, "old.txt", "text/plain", b"data", &scope,
               None, None, None, None)
        .await?;

    let updated = svc
        .patch(&mut data, &kernel, &rec.id, Some("new.txt"), None, None)
        .await?
        .unwrap();

    assert_eq!(updated.name, "new.txt");
    assert_eq!(updated.mime, "text/plain");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_patch_emits_artifact_updated() -> Result<()> {
    let (svc, mut data, kernel, ship_dir, _tmp) = setup();
    let scope = project_scope("patch-evt");
    let rec = svc
        .store(&mut data, &kernel, "before.txt", "text/plain", b"data", &scope,
               None, None, None, None)
        .await?;
    let id = rec.id.clone();

    svc.patch(&mut data, &kernel, &id, Some("after.txt"), None, None).await?;

    let events = query_events_at(&ship_dir, ARTIFACT_UPDATED);
    assert!(
        events.iter().any(|e| e.entity_id == id),
        "expected artifact.updated event with entity_id={id}; got {events:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_patch_missing_returns_none() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let result = svc.patch(&mut data, &kernel, "ghost-id", Some("name"), None, None).await?;
    assert!(result.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_patch_updates_owner_actor() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("patch-owner");
    let rec = svc
        .store(&mut data, &kernel, "f.txt", "text/plain", b"x", &scope,
               None, None, None, None)
        .await?;

    let updated = svc
        .patch(&mut data, &kernel, &rec.id, None, Some("agent-42"), None)
        .await?
        .unwrap();

    assert_eq!(updated.owner_actor.as_deref(), Some("agent-42"));
    Ok(())
}

// ── list ─────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_list_filter_by_scope() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    svc.store(&mut data, &kernel, "a", "text/plain", b"a", &project_scope("list-p"),
              None, None, None, None).await?;
    svc.store(&mut data, &kernel, "b", "text/plain", b"b", &run_scope("list-p", "r1"),
              None, None, None, None).await?;

    let mut filter = BTreeMap::new();
    filter.insert("scope".into(), serde_json::json!("project"));
    let req = QueryRequest {
        collection: ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort: BTreeMap::new(),
        limit: 50,
        offset: 0,
    };
    let results = svc.list(&mut data, req)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "a");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_list_filter_by_collection_id() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    svc.store(&mut data, &kernel, "linked", "text/plain", b"x", &project_scope("cid"),
              Some("my-coll"), Some("item-1"), None, None).await?;
    svc.store(&mut data, &kernel, "unlinked", "text/plain", b"y", &project_scope("cid"),
              None, None, None, None).await?;

    let mut filter = BTreeMap::new();
    filter.insert("collection_id".into(), serde_json::json!("my-coll"));
    let req = QueryRequest {
        collection: ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort: BTreeMap::new(),
        limit: 50,
        offset: 0,
    };
    let results = svc.list(&mut data, req)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "linked");
    Ok(())
}

// ── content-addressed dedup ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_store_dedup_same_hash_same_scope_shares_blob() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("dedup");
    let bytes = b"identical bytes";
    let r1 = svc.store(&mut data, &kernel, "a.txt", "text/plain", bytes, &scope,
                        None, None, None, None).await?;
    let r2 = svc.store(&mut data, &kernel, "b.txt", "text/plain", bytes, &scope,
                        None, None, None, None).await?;

    assert_ne!(r1.id, r2.id, "different ULID per store call");
    assert_eq!(r1.content_hash, r2.content_hash);
    assert_eq!(r1.path, r2.path, "dedup must share the same blob file");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_store_dedup_delete_first_keeps_blob_for_second() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let scope = project_scope("dedup-gc");
    let bytes = b"shared payload";
    let r1 = svc.store(&mut data, &kernel, "a.txt", "text/plain", bytes, &scope,
                        None, None, None, None).await?;
    let r2 = svc.store(&mut data, &kernel, "b.txt", "text/plain", bytes, &scope,
                        None, None, None, None).await?;
    assert_eq!(r1.path, r2.path, "sanity: same blob path");

    // Deleting the first row must leave the blob intact for r2.
    svc.delete(&mut data, &kernel, &r1.id).await?;
    let (_rec, fetched) = svc.get(&mut data, &r2.id)?.expect("r2 still resolvable");
    assert_eq!(fetched, bytes);

    // Deleting the last row drops the blob.
    svc.delete(&mut data, &kernel, &r2.id).await?;
    assert!(svc.get(&mut data, &r2.id)?.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn artifacts_store_content_hash_is_sha256() -> Result<()> {
    let (svc, mut data, kernel, _dir, _tmp) = setup();
    let bytes = b"deterministic content";
    let expected = sha256_hex(bytes);
    let rec = svc
        .store(&mut data, &kernel, "f.txt", "text/plain", bytes, &project_scope("hash"),
               None, None, None, None)
        .await?;
    assert_eq!(rec.content_hash.as_deref(), Some(expected.as_str()));
    Ok(())
}
