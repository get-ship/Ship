//! BlobStore conformance suite, run against both the on-disk
//! [`LocalFsBlobStore`] and the `object_store` in-memory backend so the two
//! impls share one contract.

use std::sync::Arc;

use blobstore::{AccountId, BlobError, BlobHash, BlobScope, BlobStore, LocalFsBlobStore};
use crate::s3_blob_store::S3BlobStore;
use ship_types::SpaceId;

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
    // Second put is a no-op; content is still intact and addressable once.
    assert!(store.exists(&s, &h1).await.unwrap());
    assert_eq!(store.get(&s, &h1).await.unwrap(), b"hello world");
}

/// B1 — scope isolation: a blob put in one scope is invisible (get + exists)
/// from every other scope, yet the SAME bytes put under distinct scopes are
/// independently addressable under the one content hash.
async fn scope_isolation(store: &dyn BlobStore) {
    let bytes = b"scope-isolated payload";
    let hash = BlobHash::of(bytes);
    let home = BlobScope::Space(SpaceId::new("a"));
    let h = store.put(&home, bytes).await.unwrap();
    assert_eq!(h, hash);

    // No other scope can see the blob written to `home`.
    let others = [
        BlobScope::Space(SpaceId::new("b")),
        BlobScope::Account(AccountId::new("a")),
        BlobScope::Device,
    ];
    for other in &others {
        match store.get(other, &hash).await {
            Err(BlobError::NotFound(_)) => {}
            res => panic!("cross-scope get from {other:?} must be NotFound, got {res:?}"),
        }
        assert!(
            !store.exists(other, &hash).await.unwrap(),
            "cross-scope exists from {other:?} must be false"
        );
    }

    // The same bytes put under a second scope are independently addressable:
    // identical hash, each retrievable only from its own scope.
    let elsewhere = BlobScope::Account(AccountId::new("a"));
    let h2 = store.put(&elsewhere, bytes).await.unwrap();
    assert_eq!(h2, hash, "content address is scope-independent");
    assert_eq!(store.get(&home, &hash).await.unwrap(), bytes);
    assert_eq!(store.get(&elsewhere, &hash).await.unwrap(), bytes);
    // The bystander scopes still see nothing.
    for other in [BlobScope::Space(SpaceId::new("b")), BlobScope::Device] {
        assert!(!store.exists(&other, &hash).await.unwrap());
    }
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
    scope_isolation(store).await;
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

#[tokio::test]
async fn local_fs_dedup_is_structural() {
    // B3 — content-addressing makes dedup free and structural: re-putting the
    // same bytes mints no second on-disk object. (The generic suite proves the
    // hash/no-op contract on both backends; this pins the on-disk count.)
    let (store, dir) = local();
    let s = scope();

    fn blob_files(root: &std::path::Path) -> usize {
        fn walk(p: &std::path::Path, n: &mut usize) {
            for entry in std::fs::read_dir(p).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, n);
                } else {
                    *n += 1;
                }
            }
        }
        let mut n = 0;
        walk(root, &mut n);
        n
    }

    let h1 = store.put(&s, b"dedup me").await.unwrap();
    let after_first = blob_files(dir.path());
    assert_eq!(after_first, 1, "first put writes exactly one object");
    let h2 = store.put(&s, b"dedup me").await.unwrap();
    assert_eq!(h1, h2);
    assert_eq!(
        blob_files(dir.path()),
        after_first,
        "re-put of identical bytes must add no new on-disk object"
    );
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
