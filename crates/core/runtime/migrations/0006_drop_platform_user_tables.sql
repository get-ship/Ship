-- Migration 0006: drop legacy user-content tables from platform.db.
--
-- User content (notes, ADRs, specs) now lives in data.db collections.
-- The `collections::migration::migrate_platform_to_collections` orchestrator
-- copies rows from these source tables into the matching collections before
-- this migration runs (see apps/shipd/src/ship_runtime.rs: DataService::open
-- is invoked before runtime::db::ensure_db_at).
--
-- `branch_context` has no destination — it was a transient branch→entity
-- link table. Workspace delete no longer writes to it. Safe to drop outright.
--
-- Indexes listed explicitly so SQLite cleans up dependent metadata even on
-- databases where the CREATE TABLE statements were removed after the fact.

DROP INDEX IF EXISTS target_kind_idx;
DROP INDEX IF EXISTS target_phase_status_idx;
DROP INDEX IF EXISTS capability_target_idx;
DROP INDEX IF EXISTS capability_milestone_idx;
DROP INDEX IF EXISTS capability_phase_idx;
DROP INDEX IF EXISTS capability_assignment_idx;
DROP INDEX IF EXISTS note_branch_idx;
DROP INDEX IF EXISTS idx_adr_status;

DROP TABLE IF EXISTS adr;
DROP TABLE IF EXISTS note;
DROP TABLE IF EXISTS capability;
DROP TABLE IF EXISTS target;
DROP TABLE IF EXISTS branch_context;
