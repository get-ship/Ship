//! `object_store`-backed [`BlobStore`] — one impl over S3/R2/MinIO plus the
//! local-filesystem and in-memory backends (per the substrate north-star, the
//! single object-store trait is the recommended byte-plane abstraction).
//!
//! Keys are `<scope-prefix>/<hash>`. `put` is head-then-put so a re-put of the
//! same content address is a no-op. `get` re-verifies the SHA-256.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use object_store::path::Path as ObjPath;
use object_store::{ObjectStore, ObjectStoreExt, PutPayload};

use blobstore::{BlobError, BlobHash, BlobScope, BlobStore};

/// Which concrete `object_store` backend to construct.
#[derive(Debug, Clone)]
pub enum BlobBackendConfig {
    /// Local filesystem rooted at `root` (via `object_store::local`).
    Local { root: PathBuf },
    /// An S3-compatible endpoint (AWS S3, Cloudflare R2, MinIO, …).
    S3 {
        bucket: String,
        region: Option<String>,
        endpoint: Option<String>,
        access_key_id: Option<String>,
        secret_access_key: Option<String>,
        /// Allow plain HTTP (e.g. a local MinIO without TLS).
        allow_http: bool,
    },
}

impl BlobBackendConfig {
    /// Construct a live [`S3BlobStore`] from this config.
    pub fn build(&self) -> Result<S3BlobStore, BlobError> {
        let store: Arc<dyn ObjectStore> = match self {
            BlobBackendConfig::Local { root } => {
                std::fs::create_dir_all(root).map_err(|e| BlobError::Io(e.to_string()))?;
                let fs = object_store::local::LocalFileSystem::new_with_prefix(root)
                    .map_err(|e| BlobError::Io(e.to_string()))?;
                Arc::new(fs)
            }
            BlobBackendConfig::S3 {
                bucket,
                region,
                endpoint,
                access_key_id,
                secret_access_key,
                allow_http,
            } => {
                let mut builder = object_store::aws::AmazonS3Builder::new()
                    .with_bucket_name(bucket)
                    .with_allow_http(*allow_http);
                if let Some(r) = region {
                    builder = builder.with_region(r);
                }
                if let Some(e) = endpoint {
                    builder = builder.with_endpoint(e);
                }
                if let Some(k) = access_key_id {
                    builder = builder.with_access_key_id(k);
                }
                if let Some(s) = secret_access_key {
                    builder = builder.with_secret_access_key(s);
                }
                Arc::new(builder.build().map_err(|e| BlobError::Io(e.to_string()))?)
            }
        };
        Ok(S3BlobStore::new(store))
    }
}

/// A [`BlobStore`] backed by any [`ObjectStore`].
pub struct S3BlobStore {
    store: Arc<dyn ObjectStore>,
}

impl S3BlobStore {
    pub fn new(store: Arc<dyn ObjectStore>) -> Self {
        Self { store }
    }

    fn key(scope: &BlobScope, hash: &BlobHash) -> ObjPath {
        ObjPath::from(format!("{}/{}", scope.prefix(), hash))
    }
}

fn is_not_found(err: &object_store::Error) -> bool {
    matches!(err, object_store::Error::NotFound { .. })
}

#[async_trait]
impl BlobStore for S3BlobStore {
    async fn put(&self, scope: &BlobScope, bytes: &[u8]) -> Result<BlobHash, BlobError> {
        let hash = BlobHash::of(bytes);
        let key = Self::key(scope, &hash);
        match self.store.head(&key).await {
            Ok(_) => return Ok(hash), // idempotent: already present
            Err(e) if is_not_found(&e) => {}
            Err(e) => return Err(BlobError::Io(e.to_string())),
        }
        let payload = PutPayload::from(Bytes::copy_from_slice(bytes));
        self.store
            .put(&key, payload)
            .await
            .map_err(|e| BlobError::Io(e.to_string()))?;
        Ok(hash)
    }

    async fn get(&self, scope: &BlobScope, hash: &BlobHash) -> Result<Vec<u8>, BlobError> {
        let key = Self::key(scope, hash);
        let result = match self.store.get(&key).await {
            Ok(r) => r,
            Err(e) if is_not_found(&e) => return Err(BlobError::NotFound(hash.to_string())),
            Err(e) => return Err(BlobError::Io(e.to_string())),
        };
        let bytes = result
            .bytes()
            .await
            .map_err(|e| BlobError::Io(e.to_string()))?;
        let actual = BlobHash::of(&bytes);
        if &actual != hash {
            return Err(BlobError::Integrity {
                expected: hash.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(bytes.to_vec())
    }

    async fn exists(&self, scope: &BlobScope, hash: &BlobHash) -> Result<bool, BlobError> {
        match self.store.head(&Self::key(scope, hash)).await {
            Ok(_) => Ok(true),
            Err(e) if is_not_found(&e) => Ok(false),
            Err(e) => Err(BlobError::Io(e.to_string())),
        }
    }

    async fn delete(&self, scope: &BlobScope, hash: &BlobHash) -> Result<(), BlobError> {
        match self.store.delete(&Self::key(scope, hash)).await {
            Ok(()) => Ok(()),
            Err(e) if is_not_found(&e) => Ok(()),
            Err(e) => Err(BlobError::Io(e.to_string())),
        }
    }
}
