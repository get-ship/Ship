pub mod data_store;
pub mod kernel_log;
pub mod single_backend;
pub mod space_context;
pub mod step_executor;

// Heavy port impls are their own leaves so their deps stay off callers that
// don't need them — reached directly, NOT re-exported here:
// - the `notify` watcher is the `fs-watch` leaf (`fs_watch::{FsWatch, LocalFsWatch}`)
// - the `object_store`/aws S3 backend is the `blob-s3` leaf
//   (`blob_s3::{S3BlobStore, BlobBackendConfig}`)

// The content-addressed byte plane (`blobstore`) and the gated disk door
// (`fs-cap`) are their own leaves. Re-exported here so
// `ports::{BlobStore, FsBroker, ...}` keeps resolving for callers.
pub use blobstore::{
    AccountId, BlobError, BlobHash, BlobScope, BlobStore, ByteStream, LocalFsBlobStore,
};
pub use data_store::{
    ColumnKind, ColumnSpec, CountQuery, DataStore, DataStoreError, FilterOp, Predicate, PrimaryKey,
    Query, QueryResult, Row, Sort, TableSchema, is_safe_ident,
};
pub use fs_cap::{FsBroker, FsCap, FsError, FsMode, FsRoot, LocalFsBroker};
pub use kernel_log::{Cursor, KernelLog};
pub use single_backend::SingleBackendDataStore;
pub use space_context::{SpaceContext, SpaceId};
pub use step_executor::{StepExecutor, StepExecutorError, StepOutput};
