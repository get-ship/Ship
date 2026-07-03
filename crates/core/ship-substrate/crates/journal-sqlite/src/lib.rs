//! journal-sqlite — the sqlx/SQLite-backed durable-execution `JournalStore`.
//!
//! The heavy (sqlx) journal backend for durable execution. Its own leaf so sqlx
//! compiles only for the crates that persist journals (shipd), and off the
//! facade's other consumers.
//!
//! NOTE (one-sqlx follow-up): `open` currently connects to `runtime.db` with its
//! OWN `SqlitePool` and manages the `durable_journals` table directly — a second
//! sqlx path into runtime.db alongside the storage engine. Routing this through
//! the storage engine is a separate, deliberate refactor.

pub mod journal_store;
pub use journal_store::SqliteJournalStore;
