//! blobstore — the content-addressed byte plane (L0 primitive).
//!
//! A blob is identified solely by the SHA-256 of its bytes, so `put` is
//! idempotent. Blobs are partitioned by [`BlobScope`] (space / account /
//! device). This crate owns the [`BlobStore`] contract and the on-disk
//! [`LocalFsBlobStore`] backend; the `object_store`-backed S3/R2/MinIO backend
//! lives in the heavy `ship-substrate` facade (it carries the aws dep).

pub mod blob_store;
pub mod local_fs_blob_store;

pub use blob_store::{AccountId, BlobError, BlobHash, BlobScope, BlobStore, ByteStream};
pub use local_fs_blob_store::LocalFsBlobStore;
