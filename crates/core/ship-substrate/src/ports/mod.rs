//! `ports` facade — collection/reactive slice + heavy back-end impls.
//!
//! Most port types live in [`ship_substrate_core::ports`] (re-exported here
//! via `super`). The heavy impls `fs_watch`, `local_fs_watch`, and
//! `s3_blob_store` (which pull in `notify` / `object_store`+aws respectively)
//! live in this crate.

pub use ship_substrate_core::ports::*;

// Heavy backends now live in their own leaves so their deps stay off the
// facade's ~27 consumers:
// - `fs-watch` (`FsWatch`/`LocalFsWatch`) carries `notify`
// - `blob-s3` (`S3BlobStore`/`BlobBackendConfig`) carries `object_store`/aws
// Consumers that watch or store to S3 depend on those leaves directly.
