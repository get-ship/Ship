-- Per-turn token usage rollup.
--
-- Each row is one assistant turn within a session. Populated by
-- AnalyticsProjection on hook.turn_observed events. Source of truth
-- for tokens; cost is a derived view computed on read against the
-- current rate table (we don't store dollar amounts because they go
-- stale + many sessions run against subscription plans where marginal
-- per-token cost is 0).
--
-- Pairing with analytics_sessions: session_key is the same FK we use
-- elsewhere (actor_id when the harness doesn't surface a session_id).
CREATE TABLE IF NOT EXISTS analytics_turns (
  id                       TEXT PRIMARY KEY NOT NULL,
  session_key              TEXT NOT NULL,
  actor_id                 TEXT NOT NULL,
  turn_idx                 INTEGER,        -- ordinal within session when known
  model                    TEXT,           -- model used for THIS turn (may differ from session model on fallback)
  input_tokens             INTEGER NOT NULL DEFAULT 0,
  output_tokens            INTEGER NOT NULL DEFAULT 0,
  reasoning_tokens         INTEGER NOT NULL DEFAULT 0,
  cache_creation_tokens    INTEGER NOT NULL DEFAULT 0,
  cache_read_tokens        INTEGER NOT NULL DEFAULT 0,
  stop_reason              TEXT,           -- "end_turn" | "max_tokens" | "tool_use" | "stop" | harness-specific
  observed_at              TEXT NOT NULL   -- ISO-8601
);
CREATE INDEX IF NOT EXISTS idx_analytics_turns_session  ON analytics_turns(session_key);
CREATE INDEX IF NOT EXISTS idx_analytics_turns_observed ON analytics_turns(observed_at);
CREATE INDEX IF NOT EXISTS idx_analytics_turns_model    ON analytics_turns(model);
