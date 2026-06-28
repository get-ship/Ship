-- Analytics projection tables — folds hook.* events into rollups the
-- dashboard queries. See crates/core/ship-substrate/src/projections/
-- analytics.rs for the apply logic and tests_analytics.rs for the
-- contract.
--
-- Pre-launch convention (per migrations/README.md) is single-file,
-- but the migration system enforces immutability on applied
-- migrations — and 0001 is already applied. New tables land here.

-- One row per harness session. Lifecycle is bracketed by
-- hook.session_started → hook.session_ended; the projection computes
-- duration_ms at end time. tool_use_count and tool_error_count are
-- incremented per tool hook so the dashboard avoids a join for the
-- common case.
CREATE TABLE IF NOT EXISTS analytics_sessions (
  session_key      TEXT PRIMARY KEY NOT NULL,
  actor_id         TEXT NOT NULL,
  harness          TEXT,
  model            TEXT,
  started_at       TEXT NOT NULL,
  ended_at         TEXT,
  duration_ms      INTEGER,
  tool_use_count   INTEGER NOT NULL DEFAULT 0,
  tool_error_count INTEGER NOT NULL DEFAULT 0,
  reason           TEXT
);
CREATE INDEX IF NOT EXISTS idx_analytics_sessions_actor   ON analytics_sessions(actor_id);
CREATE INDEX IF NOT EXISTS idx_analytics_sessions_harness ON analytics_sessions(harness);
CREATE INDEX IF NOT EXISTS idx_analytics_sessions_started ON analytics_sessions(started_at);

-- One row per tool call. Started event INSERTs an open row (no
-- ended_at); completed event finds the oldest open row for
-- (session_key, tool) and UPDATEs it with end + duration + status.
-- Open rows stay visible so the dashboard can show in-flight calls.
CREATE TABLE IF NOT EXISTS analytics_tool_uses (
  id           TEXT PRIMARY KEY NOT NULL,
  session_key  TEXT NOT NULL,
  actor_id     TEXT NOT NULL,
  tool         TEXT NOT NULL,
  status       TEXT,             -- "ok" | "error" | "unknown" | null while pending
  started_at   TEXT NOT NULL,
  ended_at     TEXT,
  duration_ms  INTEGER,
  error        TEXT
);
CREATE INDEX IF NOT EXISTS idx_analytics_tool_uses_session ON analytics_tool_uses(session_key);
CREATE INDEX IF NOT EXISTS idx_analytics_tool_uses_tool    ON analytics_tool_uses(tool);
CREATE INDEX IF NOT EXISTS idx_analytics_tool_uses_started ON analytics_tool_uses(started_at);
