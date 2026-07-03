pub mod data_store;
pub mod kernel_log;
pub mod single_backend;
pub mod space_context;
pub mod step_executor;

// Heavy impls live in the facade `ship-substrate` (carved out of substrate-core
// for the lighter L1 consumers):
// - `fs_watch` + `local_fs_watch` carry the `notify` dependency
// - `s3_blob_store` carries the `object_store` + aws dependency
// Re-exports of those types live in the facade at
// `ship_substrate::ports::{FsWatch, LocalFsWatch, S3BlobStore, BlobBackendConfig}`.

// The content-addressed byte plane (`blobstore`) and the gated disk door
// (`fs-cap`) are their own leaves. Re-exported here so
// `ports::{BlobStore, FsBroker, ...}` keeps resolving for callers.
pub use blobstore::{
    AccountId, BlobError, BlobHash, BlobScope, BlobStore, ByteStream, LocalFsBlobStore,
};
pub use fs_cap::{FsBroker, FsCap, FsError, FsMode, FsRoot, LocalFsBroker};
pub use data_store::{
    is_safe_ident, ColumnKind, ColumnSpec, CountQuery, DataStore, DataStoreError, FilterOp,
    Predicate, PrimaryKey, Query, QueryResult, Row, Sort, TableSchema,
};
pub use kernel_log::{Cursor, KernelLog};
pub use single_backend::SingleBackendDataStore;
pub use space_context::{SpaceContext, SpaceId};
pub use step_executor::{StepExecutor, StepExecutorError, StepOutput};
