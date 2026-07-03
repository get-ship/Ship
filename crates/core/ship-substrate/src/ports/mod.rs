//! `ports` facade — collection/reactive slice + heavy back-end impls.
//!
//! Most port types live in [`ship_substrate_core::ports`] (re-exported here
//! via `super`). The heavy impls `fs_watch`, `local_fs_watch`, and
//! `s3_blob_store` (which pull in `notify` / `object_store`+aws respectively)
//! live in this crate.

pub use ship_substrate_core::ports::*;

pub mod fs_watch;
pub mod local_fs_watch;

pub use fs_watch::{ChangeKind, FsDelta, FsWatch, WatchOpts};
pub use local_fs_watch::LocalFsWatch;

// The object_store-backed S3/R2 blob backend lives in the `blob-s3` leaf
// (`blob_s3::{S3BlobStore, BlobBackendConfig}`) so `object_store`/aws stays off
// every facade consumer. The blob-backend conformance suite moved with it.
