//! Parity proofs — the ported scars from the legacy `account/store.rs` tests
//! (`store.rs:127-182`), now proving the same token-store behavior over the
//! substrate `config_doc` seam: None-on-missing, write->read roundtrip, chmod
//! 600 (via write_secret_doc), idempotent clear, parent-create, clear removes.
//!
//! Each was a real invariant the file store had to hold; re-proven, not nursed.

use ship_account::store::{AccountTokenStore, FileTokenStore};
use ship_account::types::AccountToken;
use tempfile::TempDir;

fn sample_token() -> AccountToken {
    AccountToken {
        key: "ship_test_key".into(),
        user_id: "user-test-1".into(),
        expires_at: 1_924_992_000,
        stored_at: 1_700_000_000,
    }
}

fn store_at(tmp: &TempDir) -> FileTokenStore {
    FileTokenStore::new(tmp.path().join("daemon/account.jsonc"))
}

#[tokio::test]
async fn read_returns_none_when_file_missing() {
    let tmp = TempDir::new().unwrap();
    let store = store_at(&tmp);
    assert!(store.read().await.unwrap().is_none());
}

#[tokio::test]
async fn write_then_read_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let store = store_at(&tmp);
    store.write(sample_token()).await.unwrap();
    let got = store.read().await.unwrap().unwrap();
    assert_eq!(got.key, "ship_test_key");
    assert_eq!(got.user_id, "user-test-1");
    assert_eq!(got.expires_at, 1_924_992_000);
}

#[cfg(unix)]
#[tokio::test]
async fn write_sets_mode_600_on_unix() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new().unwrap();
    let store = store_at(&tmp);
    store.write(sample_token()).await.unwrap();
    let meta = std::fs::metadata(store.path()).unwrap();
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);
}

#[tokio::test]
async fn clear_is_idempotent_on_missing() {
    let tmp = TempDir::new().unwrap();
    let store = store_at(&tmp);
    store.clear().await.unwrap();
    store.clear().await.unwrap();
}

#[tokio::test]
async fn write_creates_parent_dir() {
    let tmp = TempDir::new().unwrap();
    // Path two levels deeper than the temp root — write must create parents.
    let store = FileTokenStore::new(tmp.path().join("nested/deeper/account.jsonc"));
    store.write(sample_token()).await.unwrap();
    assert!(store.read().await.unwrap().is_some());
}

#[tokio::test]
async fn clear_removes_token() {
    let tmp = TempDir::new().unwrap();
    let store = store_at(&tmp);
    store.write(sample_token()).await.unwrap();
    store.clear().await.unwrap();
    assert!(store.read().await.unwrap().is_none());
}
