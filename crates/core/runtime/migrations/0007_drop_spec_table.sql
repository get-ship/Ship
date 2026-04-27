-- Migration 0007: drop the dedicated `spec` table.
--
-- Specs are now first-party items in the `specs` collection (see
-- runtime::collections::builtin::specs_schema). The dedicated table
-- added in 0002 is no longer read or written; rows still in it are
-- migrated to the collection on first startup by
-- `runtime::collections::migration::migrate_spec_table_to_collection`
-- (idempotent; gated by a completion flag in data.db).
--
-- Indexes listed explicitly so SQLite cleans up dependent metadata
-- even on databases where the CREATE TABLE statement was rewritten
-- after the fact.

DROP INDEX IF EXISTS spec_status_idx;
DROP TABLE IF EXISTS spec;
