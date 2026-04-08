-- Drop correlation_id column and index (unused, causation_id is sufficient).
-- SQLite doesn't support DROP COLUMN before 3.35.0, so recreate the table.

DROP INDEX IF EXISTS idx_events_corr;

ALTER TABLE events DROP COLUMN correlation_id;
