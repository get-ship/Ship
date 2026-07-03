//! journal-sqlite — the sqlx/SQLite-backed durable-execution `JournalStore`.
//!
//! The journal backend for durable execution. Its own leaf because it pulls
//! `durable-execution` (→ ship-js) — which must NOT reach the substrate engine —
//! so it can't fold into `ship-substrate`'s `persistence` module.
//!
//! It does route through the engine for storage, though: a single WAL-mode
//! `SqliteConnection` (no private pool, matching the engine's single-connection
//! model) and the `durable_journals` table from the ONE shared DDL in
//! `ship_substrate::db::scope_schema`.

pub mod journal_store;
pub use journal_store::SqliteJournalStore;
