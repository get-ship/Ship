//! ship-substrate — L0 kernel infrastructure.
//!
//! Contains the event bus, persistence, DB, actor model, projections,
//! supervisor, sessions, mesh, home/project path resolution, and
//! related low-level substrate. No L1 domain logic (agent defs, skill bodies,
//! config schemas); it does carry type-only event payloads and generic
//! projections for the collection.*, artifact.*, agent.session.*, and
//! skill.* lifecycle families via ship-types, with no platform-layer
//! dependencies.

pub mod access;
pub mod config_doc;
pub mod data;
pub mod ports;
pub mod db;
pub mod diagnostics;
pub mod env;
pub mod errors;
pub mod events;
pub mod fs_util;
pub mod spawn_bounded;
pub mod task_health;
pub mod home;
pub mod isolation;
pub mod locate;
pub mod sui_compiler;
pub mod log;
pub mod persistence;
pub mod project;
pub mod projections;
pub mod security;
pub mod query;
pub mod services;
pub mod supervisor;
pub mod trash;
pub mod path_layout;
pub mod runtime_reads;

pub use home::{DaemonFile, Db, Scope, ShipHome, Vector};

pub use ports::{
    is_safe_ident, AccountId, BlobBackendConfig, BlobError, BlobHash, BlobScope, BlobStore,
    ByteStream, ColumnKind, ColumnSpec, CountQuery, Cursor, DataStore, DataStoreError, FilterOp,
    FsBroker, FsCap, FsError, FsMode, FsRoot, KernelLog, LocalFsBlobStore, LocalFsBroker,
    Predicate, PrimaryKey, Query, QueryResult, Row, S3BlobStore, SingleBackendDataStore, Sort,
    SpaceContext, SpaceId, StepExecutor, StepExecutorError, StepOutput, TableSchema,
};

pub use events::{
    EventEnvelope, list_events_since, query_events_since, read_events, read_recent_events,
};
pub use errors::{KernelError, WorkspaceError};
pub use log::{LogEntry, log_action, log_action_by, read_log, read_log_entries};
pub use trash::{TrashEntry, list_trash, purge_old_trash, restore_file, trash_file};

pub mod testutil;

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
            assert!(seen.insert(s.clone()), "duplicate suffix in tight loop: {s}");
        }
    }
}
