-- Ship platform schema — single-file, pre-launch.
--
-- Project convention while we're pre-launch: all schema changes go here,
-- and the database is wiped + recreated rather than evolved through an
-- incremental migration chain. Keeps the surface honest and the storage
-- layer free of "drop table that we never should have created" debt.

-- ═══════════════════════════════════════════════════════════════════
-- Platform layer — portable runtime
-- ═══════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS kv_state (
  namespace  TEXT NOT NULL,
  key        TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);

-- Workspace projection (from `workspace.*` events). Strictly process-
-- attachment + identity today: the columns the projection writes are
-- the same five rows that get queried (branch, id, path,
-- tmux_session_name, created_at). Earlier iterations of this table
-- carried selections (active_agent / active_preset), authored
-- providers/mcp/skills caches, and a compile-state cache; all were
-- write-only — projection updated them, nothing read them — and were
-- removed. Settings live in `kv_state`; compile state recomputes on
-- demand.
CREATE TABLE IF NOT EXISTS workspace (
  branch             TEXT PRIMARY KEY,
  id                 TEXT,
  path               TEXT,
  tmux_session_name  TEXT,
  created_at         TEXT,
  updated_at         TEXT
);

CREATE TABLE IF NOT EXISTS workspace_session (
  id                          TEXT PRIMARY KEY,
  workspace_id                TEXT NOT NULL,
  workspace_branch            TEXT NOT NULL,
  status                      TEXT NOT NULL DEFAULT 'active',
  started_at                  TEXT NOT NULL,
  ended_at                    TEXT,
  agent_id                    TEXT,
  primary_provider            TEXT,
  goal                        TEXT,
  summary                     TEXT,
  updated_workspace_ids_json  TEXT NOT NULL DEFAULT '[]',
  tool_call_count             INTEGER NOT NULL DEFAULT 0,
  drained_at                  TEXT,
  last_activity_at            TEXT,
  created_at                  TEXT NOT NULL,
  updated_at                  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS workspace_session_workspace_idx
  ON workspace_session(workspace_id, started_at DESC);
CREATE INDEX IF NOT EXISTS workspace_session_status_idx
  ON workspace_session(status, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_ws_session_drain
  ON workspace_session(status, workspace_id, agent_id);

CREATE TABLE IF NOT EXISTS workspace_session_record (
  id                         TEXT PRIMARY KEY,
  session_id                 TEXT NOT NULL UNIQUE,
  workspace_id               TEXT NOT NULL,
  workspace_branch           TEXT NOT NULL,
  summary                    TEXT,
  updated_workspace_ids_json TEXT NOT NULL DEFAULT '[]',
  duration_secs              INTEGER,
  provider                   TEXT,
  model                      TEXT,
  agent_id                   TEXT,
  files_changed              INTEGER,
  gate_result                TEXT,
  lines_added                INTEGER,
  lines_removed              INTEGER,
  tool_calls                 INTEGER,
  exit_reason                TEXT,
  created_at                 TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS workspace_session_record_workspace_idx
  ON workspace_session_record(workspace_id, created_at DESC);

CREATE TABLE IF NOT EXISTS events (
  id               TEXT PRIMARY KEY NOT NULL,
  event_type       TEXT NOT NULL,
  entity_id        TEXT NOT NULL,
  actor            TEXT NOT NULL DEFAULT 'ship',
  payload_json     TEXT NOT NULL DEFAULT '{}',
  version          INTEGER NOT NULL DEFAULT 1,
  causation_id     TEXT,
  workspace_id     TEXT,
  session_id       TEXT,
  actor_id         TEXT,
  parent_actor_id  TEXT,
  elevated         INTEGER NOT NULL DEFAULT 0,
  job_id           TEXT,
  agent_id         TEXT,
  target_actor_id  TEXT,
  created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_events_entity    ON events(entity_id, event_type);
CREATE INDEX IF NOT EXISTS idx_events_workspace ON events(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_events_session   ON events(session_id);
CREATE INDEX IF NOT EXISTS idx_events_actor     ON events(actor_id, elevated);
CREATE INDEX IF NOT EXISTS idx_events_job       ON events(job_id);

CREATE TRIGGER IF NOT EXISTS events_immutable
BEFORE UPDATE ON events
BEGIN
  SELECT RAISE(FAIL, 'events table is immutable -- use append only');
END;

-- Actors — one row per running or terminated kernel actor (chat, pipeline,
-- service, shell). The dashboard renders every running thing from this
-- table; recency comes from `last_event_at` which is bumped by triggers
-- on every event correlated to the actor (either emitted by it via
-- `actor_id` or directed at it via `target_actor_id`).
CREATE TABLE IF NOT EXISTS actors (
  id                  TEXT PRIMARY KEY NOT NULL,
  kind                TEXT NOT NULL,
  environment_type    TEXT NOT NULL DEFAULT 'local',
  status              TEXT NOT NULL DEFAULT 'created',
  workspace_id        TEXT,
  project_id          TEXT,
  cwd                 TEXT,
  parent_actor_id     TEXT,
  restart_count       INTEGER NOT NULL DEFAULT 0,
  agent_id            TEXT,
  provider            TEXT,
  model               TEXT,
  transport           TEXT,
  persistence         TEXT,
  provider_session_id TEXT,
  system_prompt_hash  TEXT,
  pid                 INTEGER,
  capabilities_json   TEXT NOT NULL DEFAULT '[]',
  exit_code           INTEGER,
  exit_reason         TEXT,
  duration_ms         INTEGER,
  last_event_at       TEXT,
  created_at          TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_actors_workspace  ON actors(workspace_id);
CREATE INDEX IF NOT EXISTS idx_actors_project    ON actors(project_id);
CREATE INDEX IF NOT EXISTS idx_actors_status     ON actors(status);
CREATE INDEX IF NOT EXISTS idx_actors_provider   ON actors(provider);
CREATE INDEX IF NOT EXISTS idx_actors_agent_id   ON actors(agent_id);
CREATE INDEX IF NOT EXISTS idx_actors_last_event ON actors(last_event_at);

CREATE TRIGGER IF NOT EXISTS bump_actor_last_event_on_actor_id
AFTER INSERT ON events
WHEN NEW.actor_id IS NOT NULL
BEGIN
  UPDATE actors SET last_event_at = NEW.created_at WHERE id = NEW.actor_id;
END;

CREATE TRIGGER IF NOT EXISTS bump_actor_last_event_on_target
AFTER INSERT ON events
WHEN NEW.target_actor_id IS NOT NULL
BEGIN
  UPDATE actors SET last_event_at = NEW.created_at WHERE id = NEW.target_actor_id;
END;

-- Job state lives entirely in the event log. `runtime::projections::job::
-- load_jobs()` folds `job.*` / `step.*` events into JobRecords on demand;
-- there is no `jobs` projection table.
