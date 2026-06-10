-- durable_journals: persisted JournalStore backing table (P1.0b).
--
-- Each row holds ONE durable run's journal: the full ordered op log serialized
-- as JSON. A journal is append-only from the engine's perspective (a later save
-- is always a superset of an earlier one), so a full UPSERT is correct and
-- simplest — no partial-row merges.
--
-- Placement: durable-execution is Runtime-scoped (agent turns and pipeline runs
-- are per-machine execution state), so the table lives in runtime.db.
-- CREATE TABLE IF NOT EXISTS means the DDL is applied in every scoped db at
-- migration time; only runtime.db receives writes from SqliteJournalStore.
--
-- The SqliteJournalStore implementation also issues its own
-- CREATE TABLE IF NOT EXISTS on construction, so it is usable against
-- any SQLite file (fresh temp dbs in tests, production runtime.db) without
-- requiring the full migration chain to have run first.

CREATE TABLE IF NOT EXISTS durable_journals (
  run_id       TEXT PRIMARY KEY,
  journal_json TEXT NOT NULL,
  updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
