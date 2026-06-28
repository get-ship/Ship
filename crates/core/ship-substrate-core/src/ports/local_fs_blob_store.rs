//! On-disk [`BlobStore`] backend.
//!
//! Layout under `root`:
//! - `Space(id)`   → `spaces/<id>/blobs/<hash[:2]>/<hash>`
//! - `Account(id)` → `account/<id>/blobs/<hash[:2]>/<hash>`
//! - `Device`      → `device/blobs/<hash[:2]>/<hash>`
//!
//! The two-char fan-out keeps directory sizes bounded. Writes are atomic
//! (temp file + rename); puts are idempotent (skip if the blob already exists).

use std::path::PathBuf;

use async_trait::async_trait;

use super::blob_store::{BlobError, BlobHash, BlobScope, BlobStore};

/// A [`BlobStore`] rooted at a local directory.
pub struct LocalFsBlobStore {
    root: PathBuf,
}

impl LocalFsBlobStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn blob_path(&self, scope: &BlobScope, hash: &BlobHash) -> PathBuf {
        let h = hash.as_str();
        self.root
            .join(scope.prefix())
            .join(&h[..2])
            .join(h)
    }

    fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<(), BlobError> {
        crate::fs_util::write_atomic(path, bytes)
            .map_err(|e| BlobError::Io(e.to_string()))
    }
}

#[async_trait]
impl BlobStore for LocalFsBlobStore {
    async fn put(&self, scope: &BlobScope, bytes: &[u8]) -> Result<BlobHash, BlobError> {
        let hash = BlobHash::of(bytes);
        let path = self.blob_path(scope, &hash);
        if path.exists() {
            return Ok(hash);
        }
        Self::write_atomic(&path, bytes)?;
        Ok(hash)
    }

    async fn get(&self, scope: &BlobScope, hash: &BlobHash) -> Result<Vec<u8>, BlobError> {
        let path = self.blob_path(scope, hash);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(BlobError::NotFound(hash.to_string()));
            }
            Err(e) => return Err(BlobError::Io(e.to_string())),
        };
        let actual = BlobHash::of(&bytes);
        if &actual != hash {
            return Err(BlobError::Integrity {
                expected: hash.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(bytes)
    }

    async fn exists(&self, scope: &BlobScope, hash: &BlobHash) -> Result<bool, BlobError> {
        Ok(self.blob_path(scope, hash).exists())
    }

    async fn delete(&self, scope: &BlobScope, hash: &BlobHash) -> Result<(), BlobError> {
        match std::fs::remove_file(self.blob_path(scope, hash)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(BlobError::Io(e.to_string())),
        }
    }
}
