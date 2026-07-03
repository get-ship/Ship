//! blob-s3 — the `object_store`-backed BlobStore backend (S3 / R2 / MinIO / in-memory).
//!
//! The heavy blob backend, sibling to the light `blobstore` contract + on-disk
//! backend. It lives in its own leaf so consumers that only need the byte plane
//! don't compile `object_store`/aws. Forward-looking: no production wiring yet
//! (Ship stores blobs on-disk today); ready for when R2/S3 is turned on.

pub mod s3_blob_store;
pub use s3_blob_store::{BlobBackendConfig, S3BlobStore};

#[cfg(test)]
mod tests_blob;
