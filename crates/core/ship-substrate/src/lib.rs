//! `ship-substrate` (L0) — the substrate engine. Truth-and-storage plus the
//! kernel/event plumbing every layer above builds on.
//!
//! This is one crate, not a `-core`/facade pair. The heavy machinery that used
//! to bloat it is now a set of leaves consumed directly where needed — the
//! `blob-s3` (object_store) backend, the `fs-watch` (notify) watcher, the
//! `isolation-port` contract, and durable-execution journal storage (the `journal-sqlite` leaf in
//! ship-exec). What remains here is the coupled engine: db + events + ports +
//! persistence + projections.
//!
//! Public modules:
//! - `ports` — `DataStore` / `KernelLog` / `SpaceContext` plus the blob trait
//!   (`BlobStore` + `LocalFsBlobStore`) and fs capability layer (`FsBroker` +
//!   `LocalFsBroker`). The `S3` backend + notify watcher are the `blob-s3` /
//!   `fs-watch` leaves.
//! - `events` — event envelope, kernel router, validator, filter, payload
//!   schema, payload_registry, pipeline.
//! - `db` — sqlite engine + per-scope event-log openers (`runtime.db` /
//!   `account.db` / `spaces/<id>/space.db`).
//! - `persistence` — `EventStore`, `SqliteEventStore`, `PartitionedSink`. The
//! - `projections` — the `EventBus` + the L0 default projection set, plus the
//!   [`ProjectionFactory`] type composition roots use to pass domain
//!   projections into store / sink / kernel construction.
//! - `data` — `DataService` (sqlx-backed `DataStore` impl).
//! - `home`, `project`, `env`, `path_layout`, `errors`, `query`, `config_doc`,
//!   `runtime_reads`, `spawn_bounded` — substrate-side
//!   utilities, all std-only.

pub use config_doc;
pub mod data;
pub mod db;
pub mod ports;
pub use ship_paths::env;
pub mod errors;
pub mod events;
pub use fs_util;
pub mod home;
pub mod persistence;
pub mod project;
pub mod projections;
pub mod query;
pub mod services;
pub mod text;
pub use path_layout;
pub mod runtime_reads;
pub mod snapshot;
pub use home::{DaemonFile, Db, ShipHome};

pub mod testutil;

pub use ports::{
    AccountId, BlobError, BlobHash, BlobScope, BlobStore, ByteStream, ColumnKind, ColumnSpec,
    CountQuery, Cursor, DataStore, DataStoreError, FilterOp, FsBroker, FsCap, FsError, FsMode,
    FsRoot, KernelLog, LocalFsBlobStore, LocalFsBroker, Predicate, PrimaryKey, Query, QueryResult,
    Row, SingleBackendDataStore, Sort, SpaceContext, SpaceId, StepExecutor, StepExecutorError,
    StepOutput, TableSchema, is_safe_ident,
};
// `S3BlobStore` + `BlobBackendConfig` (object_store/aws) are the `blob-s3` leaf —
// reach them at `blob_s3::{S3BlobStore, BlobBackendConfig}`, not through the engine.

pub use errors::KernelError;
pub use events::{
    EventEnvelope, list_events_since, query_events_since, read_events, read_recent_events,
};

// Persistence + event-router aliases surfaced at the crate root for callers
// who historically imported them via `ship_substrate::Foo` rather than
// `ship_substrate::events::Foo`.
pub use events::{
    ElevatedValidator, EventFilter, EventValidator, KERNEL_NAMESPACES, KernelRouter,
    NamespaceValidator, PLATFORM_ONLY_NAMESPACES, PayloadSchemaValidator, RESERVED_NAMESPACES,
    ReservedNamespaceValidator, SharedKernelRouter, ValidationError, init_kernel_router,
    init_kernel_router_with, is_kernel_event, new_isolated_for_test, new_isolated_for_test_with,
};
pub use persistence::{EventStore, PartitionedSink, SqliteEventStore};
pub use projections::{ProjectionFactory, no_extra_projections};
pub use services::{HealthStatus, ServiceHandle, ServiceHandler, run_service, spawn_service};
// Space-snapshot surface — the substrate-level backup primitive. Used by
// ship-space's wire layer to satisfy the `SpaceService.Snapshot` RPC, and
// by tests + restoration tools for the inverse path.
pub use snapshot::{
    RestoreResult, SnapshotManifest, SnapshotResult, restore_space, snapshot_space,
};
/// Generate a monotonic ULID string. Thread-safe via thread-local generator.
pub fn gen_ulid() -> String {
    use std::cell::RefCell;
    use std::time::SystemTime;
    thread_local! {
        static GEN: RefCell<ulid::Generator> = const { RefCell::new(ulid::Generator::new()) };
    }
    GEN.with(|g| {
        let now = SystemTime::now();
        let mut generator = g.borrow_mut();
        // Monotonic overflow within a single ms — reset the generator so the
        // counter starts fresh at the new timestamp and try again.
        generator
            .generate_from_datetime(now)
            .unwrap_or_else(|_| {
                *generator = ulid::Generator::new();
                ulid::Ulid::from_datetime(now)
            })
            .to_string()
    })
}

/// Generate a short, collision-free lowercase slug suffix (8 chars).
///
/// Drawn from the ULID's RANDOM tail, never its timestamp head. A
/// ULID's first 10 chars encode the millisecond timestamp, so any
/// prefix slice (`gen_ulid()[..8]`) is pure time — rapid dispatches in
/// the same workspace land in the same ms bucket and collide. The last
/// 8 chars are the random component, so suffixes are unique even for
/// back-to-back generation. Use this for `<workspace>-<suffix>` slugs.
pub fn gen_slug_suffix() -> String {
    let ulid = gen_ulid();
    ulid[ulid.len() - 8..].to_lowercase()
}

/// Generate a nanoid using Ship's 56-character alphabet (no ambiguous chars).
/// Same alphabet as `stack_model::gen_nanoid` — substrate keeps a local copy
/// so L0 does not depend on stack-model (stack inversion: stack may depend on
/// product crates that depend on substrate).
pub fn gen_nanoid() -> String {
    let alphabet: [char; 56] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
        'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z',
    ];
    nanoid::format(nanoid::rngs::default, &alphabet, 8)
}

#[cfg(test)]
mod slug_suffix_tests {
    use super::gen_slug_suffix;
    use std::collections::HashSet;

    #[test]
    fn rapid_suffixes_are_unique() {
        // Regression: slugs were built from gen_ulid()[..8] (the
        // millisecond-timestamp head), so back-to-back same-ms dispatches
        // collided, sharing a worktree + tmux session. Drawing from the
        // random tail must make tight loops collision-free.
        let mut seen = HashSet::new();
        for _ in 0..1000 {
            let s = gen_slug_suffix();
            assert_eq!(s.len(), 8, "suffix must be 8 chars: {s}");
            assert_eq!(s, s.to_lowercase(), "suffix must be lowercase: {s}");
            assert!(
                seen.insert(s.clone()),
                "duplicate suffix in tight loop: {s}"
            );
        }
    }
}
