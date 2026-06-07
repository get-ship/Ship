//! Private blob storage backend — delegates to `ship_substrate::ports::LocalFsBlobStore`.
//!
//! `BlobBackend` is a narrow sync seam so the service's atomicity story
//! (emit event → GC on failure) is not entangled with async executor details.
//! The underlying `LocalFsBlobStore` does only synchronous I/O internally,
//! so we drive it with `block_in_place` + `Handle::block_on`.

use std::io;
use std::path::PathBuf;

use anyhow::Result;
use ship_substrate::ports::{BlobHash, BlobScope, BlobStore, LocalFsBlobStore, SpaceId};

use crate::types::ArtifactScope;

pub(crate) trait BlobBackend: Send + Sync {
    /// Store `bytes` and return the relative path (from the backend root)
    /// at which the blob is addressable. The path encodes the content hash
    /// so callers can recover it via `path.split('/').last()`.
    fn put(&self, scope: &ArtifactScope, bytes: &[u8]) -> Result<String>;
    /// Fetch bytes by the key stored in the last segment of the artifact's
    /// `path` column (the content hash for `SubstrateBlobBackend`).
    fn get(&self, scope: &ArtifactScope, key: &str) -> Result<Vec<u8>>;
    /// Remove the blob. Returns `true` if it existed, `false` if already absent.
    fn delete(&self, scope: &ArtifactScope, key: &str) -> Result<bool>;
}

/// Maps `ArtifactScope` to a `BlobScope` understood by the substrate port.
///
/// `Run` and `Project` use different `SpaceId` values so their blob directories
/// never overlap: a GC pass on one scope's path count will not delete the other
/// scope's copy of the same content-hash.
///
/// Layout:
/// - `Project { space_id }` → `spaces/<space_id>/blobs/<h[:2]>/<hash>`
/// - `Run { space_id, run_id }` → `spaces/<space_id>@<run_id>/blobs/<h[:2]>/<hash>`
fn to_blob_scope(scope: &ArtifactScope) -> BlobScope {
    match scope {
        ArtifactScope::Project { space_id } => BlobScope::Space(SpaceId(space_id.clone())),
        ArtifactScope::Run { space_id, run_id } => {
            BlobScope::Space(SpaceId(format!("{space_id}@{run_id}")))
        }
    }
}

/// Blob backend that delegates to `ship_substrate::ports::LocalFsBlobStore`.
/// Blobs live at `{root}/spaces/<space_id>/blobs/<hash[:2]>/<hash>`.
pub(crate) struct SubstrateBlobBackend {
    store: LocalFsBlobStore,
}

impl SubstrateBlobBackend {
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            store: LocalFsBlobStore::new(root.into()),
        }
    }

    /// Drive an async future synchronously. `LocalFsBlobStore`'s `put`/`get`
    /// do only sync I/O internally; `block_in_place` is safe here.
    fn block<F, T>(fut: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        })
    }
}

impl BlobBackend for SubstrateBlobBackend {
    fn put(&self, scope: &ArtifactScope, bytes: &[u8]) -> Result<String> {
        let blob_scope = to_blob_scope(scope);
        let hash = Self::block(self.store.put(&blob_scope, bytes))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        // Build rel_path: `<scope-prefix>/<hash[:2]>/<hash>`
        let h = hash.as_str();
        let rel = format!("{}/{}/{h}", blob_scope.prefix(), &h[..2]);
        Ok(rel)
    }

    fn get(&self, scope: &ArtifactScope, key: &str) -> Result<Vec<u8>> {
        let blob_scope = to_blob_scope(scope);
        let hash = BlobHash::parse(key).map_err(|e| anyhow::anyhow!("{e}"))?;
        Self::block(self.store.get(&blob_scope, &hash)).map_err(|e| {
            if matches!(e, ship_substrate::ports::BlobError::NotFound(_)) {
                anyhow::anyhow!(io::Error::new(io::ErrorKind::NotFound, e.to_string()))
            } else {
                anyhow::anyhow!("{e}")
            }
        })
    }

    fn delete(&self, scope: &ArtifactScope, key: &str) -> Result<bool> {
        let blob_scope = to_blob_scope(scope);
        let hash = match BlobHash::parse(key) {
            Ok(h) => h,
            // Key is not a valid hash (e.g. a ULID from old records) — treat as absent.
            Err(_) => return Ok(false),
        };
        let existed = Self::block(self.store.exists(&blob_scope, &hash))
            .unwrap_or(false);
        Self::block(self.store.delete(&blob_scope, &hash))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(existed)
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory backend for unit tests — no disk I/O.
    pub(crate) struct InMemoryBackend {
        blobs: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl InMemoryBackend {
        pub(crate) fn new() -> Self {
            Self { blobs: Mutex::new(HashMap::new()) }
        }
    }

    impl BlobBackend for InMemoryBackend {
        fn put(&self, _scope: &ArtifactScope, bytes: &[u8]) -> Result<String> {
            let hash = ship_types::sha256_hex(bytes);
            self.blobs.lock().unwrap().insert(hash.clone(), bytes.to_vec());
            Ok(format!("mem/{}/{hash}", &hash[..2]))
        }

        fn get(&self, _scope: &ArtifactScope, key: &str) -> Result<Vec<u8>> {
            self.blobs.lock().unwrap().get(key).cloned().ok_or_else(|| {
                anyhow::anyhow!(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("blob missing: {key}"),
                ))
            })
        }

        fn delete(&self, _scope: &ArtifactScope, key: &str) -> Result<bool> {
            Ok(self.blobs.lock().unwrap().remove(key).is_some())
        }
    }
}
