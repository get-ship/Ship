//! BlobStore conformance suite, run against both the on-disk
//! [`LocalFsBlobStore`] and the `object_store` in-memory backend so the two
//! impls share one contract.

use std::sync::Arc;

use super::blob_store::{BlobError, BlobHash, BlobScope, BlobStore};
use super::local_fs_blob_store::LocalFsBlobStore;
use super::s3_blob_store::S3BlobStore;
use super::space_context::SpaceId;

fn scope() -> BlobScope {
    BlobScope::Space(SpaceId::new("space-1"))
}

fn local() -> (LocalFsBlobStore, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    (LocalFsBlobStore::new(dir.path()), dir)
}

fn memory() -> S3BlobStore {
    S3BlobStore::new(Arc::new(object_store::memory::InMemory::new()))
}

async fn put_idempotency(store: &dyn BlobStore) {
    let s = scope();
    let h1 = store.put(&s, b"hello world").await.unwrap();
    let h2 = store.put(&s, b"hello world").await.unwrap();
    assert_eq!(h1, h2, "same bytes must yield same hash");
    assert_eq!(h1, BlobHash::of(b"hello world"));
    // Second put is a no-op; content is still intact.
    assert_eq!(store.get(&s, &h1).await.unwrap(), b"hello world");
}

async fn get_not_found(store: &dyn BlobStore) {
    let missing = BlobHash::of(b"never stored");
    match store.get(&scope(), &missing).await {
        Err(BlobError::NotFound(_)) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
}

async fn delete_tolerant_missing(store: &dyn BlobStore) {
    let missing = BlobHash::of(b"absent");
    store.delete(&scope(), &missing).await.unwrap();
    // Put, delete, and confirm gone.
    let h = store.put(&scope(), b"to delete").await.unwrap();
    store.delete(&scope(), &h).await.unwrap();
    assert!(!store.exists(&scope(), &h).await.unwrap());
}

async fn exists_reports(store: &dyn BlobStore) {
    let h = store.put(&scope(), b"present").await.unwrap();
    assert!(store.exists(&scope(), &h).await.unwrap());
    assert!(!store.exists(&scope(), &BlobHash::of(b"other")).await.unwrap());
}

async fn integrity_roundtrip(store: &dyn BlobStore) {
    // get() re-verifies sha256(bytes) == hash; a clean roundtrip must pass it.
    let bytes = b"integrity matters";
    let h = store.put(&scope(), bytes).await.unwrap();
    assert_eq!(store.get(&scope(), &h).await.unwrap(), bytes);
}

async fn run_suite(store: &dyn BlobStore) {
    put_idempotency(store).await;
    get_not_found(store).await;
    delete_tolerant_missing(store).await;
    exists_reports(store).await;
    integrity_roundtrip(store).await;
}

#[tokio::test]
async fn local_fs_blob_store_conformance() {
    let (store, _dir) = local();
    run_suite(&store).await;
}

#[tokio::test]
async fn object_store_memory_conformance() {
    run_suite(&memory()).await;
}

#[tokio::test]
async fn local_fs_integrity_detects_corruption() {
    // Tamper with the on-disk bytes and confirm get() rejects the mismatch.
    let (store, _dir) = local();
    let s = scope();
    let h = store.put(&s, b"original").await.unwrap();
    // Overwrite the blob file in place with different bytes of equal length.
    let path = _dir
        .path()
        .join(s.prefix())
        .join(&h.as_str()[..2])
        .join(h.as_str());
    std::fs::write(&path, b"tampered").unwrap();
    match store.get(&s, &h).await {
        Err(BlobError::Integrity { .. }) => {}
        other => panic!("expected Integrity error, got {other:?}"),
    }
}

#[test]
fn blob_hash_parse_validates_hex() {
    let good = BlobHash::of(b"x");
    assert!(BlobHash::parse(good.as_str()).is_ok());
    assert!(BlobHash::parse("nothex").is_err());
    assert!(BlobHash::parse("ABCDEF").is_err(), "uppercase rejected");
}

#[test]
fn scope_prefix_partitions() {
    use super::blob_store::AccountId;
    assert_eq!(
        BlobScope::Space(SpaceId::new("s1")).prefix(),
        "spaces/s1/blobs"
    );
    assert_eq!(
        BlobScope::Account(AccountId::new("a1")).prefix(),
        "account/a1/blobs"
    );
    assert_eq!(BlobScope::Device.prefix(), "device/blobs");
}
