pub mod data_store;
pub mod fs_broker;
pub mod kernel_log;
pub mod local_fs_broker;
pub mod single_backend;
pub mod space_context;
pub mod step_executor;

// Heavy impls live in the facade `ship-substrate` (carved out of substrate-core
// for the lighter L1 consumers):
// - `fs_watch` + `local_fs_watch` carry the `notify` dependency
// - `s3_blob_store` carries the `object_store` + aws dependency
// Re-exports of those types live in the facade at
// `ship_substrate::ports::{FsWatch, LocalFsWatch, S3BlobStore, BlobBackendConfig}`.

#[cfg(test)]
mod tests_fs_broker;

// The content-addressed byte plane is its own leaf (`blobstore`). Re-exported
// here so `ports::{BlobStore, BlobHash, ...}` keeps resolving for callers.
pub use blobstore::{
    AccountId, BlobError, BlobHash, BlobScope, BlobStore, ByteStream, LocalFsBlobStore,
};
pub use data_store::{
    is_safe_ident, ColumnKind, ColumnSpec, CountQuery, DataStore, DataStoreError, FilterOp,
    Predicate, PrimaryKey, Query, QueryResult, Row, Sort, TableSchema,
};
pub use fs_broker::{FsBroker, FsCap, FsError, FsMode, FsRoot};
pub use kernel_log::{Cursor, KernelLog};
pub use local_fs_broker::LocalFsBroker;
pub use single_backend::SingleBackendDataStore;
pub use space_context::{SpaceContext, SpaceId};
pub use step_executor::{StepExecutor, StepExecutorError, StepOutput};
