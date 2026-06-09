-- canonical_sessions: read-model for canonical session.*/session.turn.* events.
--
-- Folds the canonical agent-session family (session.started, session.completed,
-- session.failed, session.cancelled, session.suspended, session.turn.started,
-- session.turn.ended) into a queryable table for the sessions projection.
--
-- Table lives in runtime.db (the only db that receives both Space-scoped
-- session.* and Runtime-scoped session.turn.* events in the same physical file).
-- The CREATE TABLE IF NOT EXISTS means the table is created in every scoped DB
-- at migration time (same pattern as agent_sessions), but only runtime.db
-- receives writes from CanonicalSessionProjection::apply().
--
-- Status FSM: active -> awaiting_input (via turn.ended) -> completed/failed/
--             cancelled/suspended (terminals); suspended is non-terminal
--             (expected to resume).
--
-- See the canonical-session-events ADR / .artifacts/agents-architecture.md.

CREATE TABLE IF NOT EXISTS canonical_sessions (
  session_id   TEXT PRIMARY KEY NOT NULL,
  space_id     TEXT NOT NULL,
  agent_id     TEXT NOT NULL,
  harness      TEXT NOT NULL,
  model        TEXT,
  status       TEXT NOT NULL,          -- active|suspended|awaiting_input|completed|failed|cancelled
  input        TEXT,
  parent       TEXT,
  turn_count   INTEGER NOT NULL DEFAULT 0,
  started_at   TEXT NOT NULL,          -- ISO-8601
  ended_at     TEXT,                   -- ISO-8601; NULL until terminal
  outcome      TEXT,                   -- completed|failed|cancelled|suspended; NULL while active
  reason       TEXT                    -- failure reason or suspend checkpoint; NULL otherwise
);

CREATE INDEX IF NOT EXISTS idx_canonical_sessions_space   ON canonical_sessions(space_id);
CREATE INDEX IF NOT EXISTS idx_canonical_sessions_agent   ON canonical_sessions(agent_id);
CREATE INDEX IF NOT EXISTS idx_canonical_sessions_status  ON canonical_sessions(status);
