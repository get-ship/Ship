//! `ship-substrate` (L0) — substrate facade + heavy back-end machinery.
//!
//! Post-carve (2026-06-28), this crate is a back-compat facade over
//! [`ship_substrate_core`] (the shared collection/reactive slice, used by both
//! shipd AND the lightweight L1 consumers `ship-collection`/`ship-reactive`/
//! `ship-artifacts`) plus the heavy back-end machinery that lives only on the
//! daemon side:
//!
//! - [`ports::fs_watch`] + [`ports::local_fs_watch`] (notify)
//! - [`services`] — `spawn_service` background-actor primitive (mesh + access deleted as dead)
//! - [`supervisor`] — daemon supervisor runtime
//! - [`isolation`] — sandbox decision layer
//! - [`persistence::journal_store`] — durable-execution journal store
//! - [`events::pipeline`] / [`events::payload_registry`] — these moved INTO
//!   `ship-substrate-core` (they depend on `ship-pipelines` types only, which
//!   is itself a clean crate — see [`ship_substrate_core::events::pipeline`]
//!   and [`ship_substrate_core::events::payload_registry`]). Re-exported here
//!   so `ship_substrate::events::pipeline` and `ship_substrate::events::payload_registry`
//!   keep resolving unchanged.
//!
//! L0 registers only runtime mechanics (actor.*, process.*, executor.*,
//! daemon.*, kernel.actor.*, mesh.*, supervisor.shutting_down,
//! settings.changed, sync.*†). Domain families (collection.*, artifact.*,
//! studio.*, skill.*, auth.*, session.*, op.*, dispatch.*) register in
//! their owning L1 crates. †sync.* is queued to move to ship-cloud.

// ── Heavy back-end modules that physically live in the facade ───────────────
pub mod isolation;
pub mod services;
pub mod supervisor;
// ── Ports facade ───────────────────────────────────────────────────────────
//
// `blob_store`/`data_store`/`fs_broker`/`kernel_log`/`local_fs_{broker,blob_store}`/
// `single_backend`/`space_context`/`step_executor` live in `ship-substrate-core`;
// `fs_watch`/`local_fs_watch` live here (notify dep). The object_store-backed
// `s3_blob_store` + the blob conformance suite are the `blob-s3` leaf.
pub mod ports;

// ── Events facade ───────────────────────────────────────────────────────────
//
// Everything L0-event-related (types / validator / kernel_router / global_kernel_router
// / kernel_lifecycle / filter / payload_schema / payload_registry / pipeline /
// daemon / executor / process) lives in `ship-substrate-core` — they depend
// on `ship-pipelines` types only, which is itself a clean crate (no
// object_store / notify / portable-pty), so the carve invariant holds without
// per-facade shims.
pub mod events {
    pub use ship_substrate_core::events::*;
}

// ── Persistence facade ─────────────────────────────────────────────────────
//
// `actor_store`/`event_store`/`partitioned_sink` live in core;
// `journal_store` (durable-execution) lives here.
pub mod persistence {
    pub use ship_substrate_core::persistence::*;

    pub mod journal_store;
    pub use journal_store::SqliteJournalStore;
}

// ── Other core modules (re-exported whole) ──────────────────────────────────
//
// Everything below is owned by `ship-substrate-core`; the facade exposes
// each at the original `ship_substrate::<path>` for back-compat.
pub use ship_substrate_core::{
    config_doc, data, db, env, errors, fs_util, home, path_layout,
    project, projections, query, runtime_reads, spawn_bounded,
    task_health, testutil,
};

// ── Public re-exports at the crate root for back-compat ─────────────────────
pub use ship_substrate_core::{
    gen_nanoid, gen_slug_suffix, gen_ulid, DaemonFile, Db, EventEnvelope,
    KernelError, Scope, ShipHome, Vector, WorkspaceError,
    list_events_since, query_events_since, read_events,
    read_recent_events,
    is_safe_ident, AccountId, BlobError, BlobHash, BlobScope, BlobStore, ByteStream,
    ColumnKind, ColumnSpec, CountQuery, Cursor, DataStore, DataStoreError, FilterOp,
    FsBroker, FsCap, FsError, FsMode, FsRoot, KernelLog, LocalFsBlobStore, LocalFsBroker,
    Predicate, PrimaryKey, Query, QueryResult, Row, SingleBackendDataStore, Sort,
    SpaceContext, SpaceId, StepExecutor, StepExecutorError, StepOutput, TableSchema,
    ActorStore, EventStore, SqliteEventStore, PartitionedSink, EventFilter,
    init_kernel_router, kernel_router, new_isolated_for_test, KernelRouter,
    PayloadSchemaValidator, ElevatedValidator, EventValidator, KERNEL_NAMESPACES,
    NamespaceValidator, PLATFORM_ONLY_NAMESPACES, RESERVED_NAMESPACES,
    ReservedNamespaceValidator, ValidationError, is_kernel_event,
    // types::event_types re-export
};
