//! ship-substrate — L0 kernel infrastructure.
//!
//! Contains the event bus, persistence, DB, actor model, projections,
//! supervisor, sessions, mesh, home/project path resolution, and
//! related low-level substrate. No knowledge of agents, skills, collections,
//! config schemas, or any L1 primitives.

pub mod access;
pub mod data;
pub mod db;
pub mod env;
pub mod errors;
pub mod events;
pub mod fs_util;
pub mod spawn_bounded;
pub mod task_health;
pub mod git;
pub mod home;
pub mod isolation;
pub mod log;
pub mod persistence;
pub mod project;
pub mod projections;
pub mod services;
pub mod supervisor;
pub mod trash;

pub use home::{DaemonFile, Db, Scope, ShipHome, Vector};

pub use events::{
    EventEnvelope, list_events_since, query_events_since, read_events, read_recent_events,
};
pub use errors::{KernelError, WorkspaceError};
pub use db::types::SessionDb;
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

pub fn gen_nanoid() -> String {
    let alphabet: [char; 56] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
        'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z',
    ];
    nanoid::format(nanoid::rngs::default, &alphabet, 8)
}
