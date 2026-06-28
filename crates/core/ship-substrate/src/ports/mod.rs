//! `ports` facade — collection/reactive slice + heavy back-end impls.
//!
//! Most port types live in [`ship_substrate_core::ports`] (re-exported here
//! via `super`). The heavy impls `fs_watch`, `local_fs_watch`, and
//! `s3_blob_store` (which pull in `notify` / `object_store`+aws respectively)
//! live in this crate.

pub use ship_substrate_core::ports::*;

pub mod fs_watch;
pub mod local_fs_watch;
pub mod s3_blob_store;

pub use fs_watch::{ChangeKind, FsDelta, FsWatch, WatchOpts};
pub use local_fs_watch::LocalFsWatch;
pub use s3_blob_store::{BlobBackendConfig, S3BlobStore};

// Conformance test for both blob backends; stays here because the S3 impl
// is in this crate.
#[cfg(test)]
mod tests_blob;
