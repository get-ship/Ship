-- Add slug column to canonical_sessions table.
--
-- slug is the human-readable operational identifier carried on the
-- session.started payload. Required for the cancel path, sweep orphan
-- check, and pipeline compose-terminal lookup — see the
-- canonical-emitter-migration ADR for the full rationale.
--
-- NULL for sessions that pre-date this migration (Floki sessions and any
-- session.started that didn't carry a slug); backfill is not possible since
-- the original agent.session.started events are in a different table.

ALTER TABLE canonical_sessions ADD COLUMN slug TEXT;

CREATE INDEX IF NOT EXISTS idx_canonical_sessions_slug ON canonical_sessions(slug);
