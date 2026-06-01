//! BlobStore — the content-addressed byte plane (L0 port).
//!
//! A blob is identified solely by the SHA-256 of its bytes, so `put` is
//! idempotent: writing the same bytes twice yields the same [`BlobHash`] and
//! the second write is a no-op. Blobs are partitioned by [`BlobScope`] so a
//! space, an account, or the device each address an isolated namespace.
//!
//! This module defines the contract only. Concrete backends live in
//! [`super::local_fs_blob_store`] (on-disk) and [`super::s3_blob_store`]
//! (`object_store`-backed: S3/R2/MinIO/in-memory).

use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;

use super::space_context::SpaceId;

/// Opaque per-account identifier (mirrors [`SpaceId`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

impl AccountId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Content address of a blob: the lowercase hex SHA-256 of its bytes.
///
/// The ONLY way to mint a hash from bytes is [`BlobHash::of`], which delegates
/// to [`ship_types::sha256_hex`]. [`BlobHash::parse`] validates an existing hex
/// string without re-hashing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlobHash(String);

impl BlobHash {
    /// Mint a content address by hashing `bytes`. The single hashing ctor.
    pub fn of(bytes: &[u8]) -> Self {
        Self(ship_types::sha256_hex(bytes))
    }

    /// Validate an existing 64-char lowercase-hex digest without re-hashing.
    pub fn parse(s: impl Into<String>) -> Result<Self, BlobError> {
        let s = s.into();
        let valid = s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if valid {
            Ok(Self(s))
        } else {
            Err(BlobError::InvalidHash(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BlobHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Namespace a blob lives in. Matches the substrate scope model — there is no
/// Run/Project/User scope at the byte plane.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlobScope {
    Space(SpaceId),
    Account(AccountId),
    Device,
}

impl BlobScope {
    /// Storage prefix for this scope (no trailing slash). Shared by every
    /// backend so on-disk and object-store layouts agree on partitioning.
    pub fn prefix(&self) -> String {
        match self {
            BlobScope::Space(id) => format!("spaces/{id}/blobs"),
            BlobScope::Account(id) => format!("account/{id}/blobs"),
            BlobScope::Device => "device/blobs".to_string(),
        }
    }
}

/// Errors surfaced by a [`BlobStore`].
#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("blob not found: {0}")]
    NotFound(String),
    #[error("invalid blob hash: {0}")]
    InvalidHash(String),
    #[error("integrity check failed: expected {expected}, got {actual}")]
    Integrity { expected: String, actual: String },
    #[error("blob io error: {0}")]
    Io(String),
}

/// A boxed byte stream yielded by [`BlobStore::get_stream`].
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, BlobError>> + Send>>;

/// Content-addressed byte store. `put` is idempotent on identical bytes.
#[async_trait]
pub trait BlobStore: Send + Sync {
    /// Store `bytes`, returning their content address. Idempotent: a second
    /// put of the same bytes is a no-op and returns the same hash.
    async fn put(&self, scope: &BlobScope, bytes: &[u8]) -> Result<BlobHash, BlobError>;

    /// Fetch a blob and re-verify `sha256(bytes) == hash` before returning.
    async fn get(&self, scope: &BlobScope, hash: &BlobHash) -> Result<Vec<u8>, BlobError>;

    /// Whether a blob exists in `scope`.
    async fn exists(&self, scope: &BlobScope, hash: &BlobHash) -> Result<bool, BlobError>;

    /// Remove a blob. Tolerant of a missing blob (returns `Ok`).
    async fn delete(&self, scope: &BlobScope, hash: &BlobHash) -> Result<(), BlobError>;

    /// Stream bytes in, buffering and hashing them, then store. Default impl
    /// collects the stream and delegates to [`BlobStore::put`].
    async fn put_stream(
        &self,
        scope: &BlobScope,
        mut stream: ByteStream,
    ) -> Result<BlobHash, BlobError> {
        use futures_util::StreamExt;
        let mut buf = Vec::new();
        while let Some(chunk) = stream.next().await {
            buf.extend_from_slice(&chunk?);
        }
        self.put(scope, &buf).await
    }

    /// Stream a blob's bytes out. Default impl fetches via [`BlobStore::get`]
    /// (which integrity-checks) and yields a single chunk.
    async fn get_stream(
        &self,
        scope: &BlobScope,
        hash: &BlobHash,
    ) -> Result<ByteStream, BlobError> {
        let bytes = self.get(scope, hash).await?;
        Ok(Box::pin(futures_util::stream::once(async move {
            Ok(Bytes::from(bytes))
        })))
    }
}
