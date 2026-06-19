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

-- The workspace primitive retired. Worktrees are a git feature, queried
-- on demand via `runtime::git`; the runtime stores no worktree state.

CREATE TABLE IF NOT EXISTS events (

  id               TEXT PRIMARY KEY NOT NULL,
  event_type       TEXT NOT NULL,
  entity_id        TEXT NOT NULL,
  actor            TEXT NOT NULL DEFAULT 'ship',
  payload_json     TEXT NOT NULL DEFAULT '{}',
  version          INTEGER NOT NULL DEFAULT 1,
  causation_id     TEXT,
  -- Partition tuple (D0): scope = device|account|space|system, scope_key = the
  -- id within that scope (Space→space_id, Account→account_id, Device/System→
  -- device_id or NULL). Replaces the loose space_id column; scope and its key
  -- always travel together. Pre-existing DBs are migrated by events_scope_reconcile.
  scope            TEXT NOT NULL DEFAULT 'device',
  scope_key        TEXT,
  session_id       TEXT,
  actor_id         TEXT,
  parent_actor_id  TEXT,
  elevated         INTEGER NOT NULL DEFAULT 0,
  job_id           TEXT,
  agent_id         TEXT,
  target_actor_id  TEXT,
  -- Author-identity + signature groundwork (Phase 2, additive). author_*
  -- carry the signing principal; signature is reserved (NULL until a later
  -- phase signs over EventEnvelope::signing_bytes). Pre-existing DBs get these
  -- added idempotently by events_author_reconcile.
  author_account_id TEXT,
  author_device_id  TEXT,
  signature         TEXT,
  created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_events_entity    ON events(entity_id, event_type);
CREATE INDEX IF NOT EXISTS idx_events_scope   ON events(scope, scope_key, created_at);
CREATE INDEX IF NOT EXISTS idx_events_session   ON events(session_id);
CREATE INDEX IF NOT EXISTS idx_events_actor     ON events(actor_id, elevated);
CREATE INDEX IF NOT EXISTS idx_events_job       ON events(job_id);

CREATE TRIGGER IF NOT EXISTS events_immutable
BEFORE UPDATE ON events
BEGIN
  SELECT RAISE(FAIL, 'events table is immutable -- use append only');
END;

-- Bookmarks: sync push/pull positions and per-projection rebuild cursors.
-- One row per named cursor; value is the last-applied event id (ULID).
-- Naming convention: `{purpose}:{scope}` — e.g. `push:platform:{space_id}`,
-- `projection:auth_sessions`. Names are opaque to the table.
CREATE TABLE IF NOT EXISTS cursors (
  name           TEXT PRIMARY KEY,
  last_event_id  TEXT NOT NULL,
  updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Machine-wide daemon settings (terminal.program, daemon.port, etc.).
-- One row per key. Replaces the `settings` namespace in kv.db; daemon-managed
-- values that need ACID alongside event writes belong here, not in KV.
CREATE TABLE IF NOT EXISTS daemon_settings (
  key         TEXT PRIMARY KEY NOT NULL,
  value_json  TEXT NOT NULL,
  updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Auth sessions — daemon-scoped bearer tokens issued to local clients
-- (CLI, MCP agents, Studio-local). Token lookup happens on every authed
-- request. Soft-delete via `revoked_at_unix` for audit; the projection
-- is materialized from auth.session.{created,revoked,expired} events.
CREATE TABLE IF NOT EXISTS auth_sessions (
  token            TEXT PRIMARY KEY NOT NULL,
  agent_id         TEXT NOT NULL,
  scopes_json      TEXT NOT NULL,
  space_id       TEXT,
  created_at_unix  INTEGER NOT NULL,
  expires_at_unix  INTEGER NOT NULL,
  revoked_at_unix  INTEGER,
  source           TEXT NOT NULL,
  updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_auth_sessions_active
  ON auth_sessions(token) WHERE revoked_at_unix IS NULL;

-- Daemon owner — at most one row, indicating which cloud user paired this
-- daemon. Materialized from daemon.owner.{bound,unbound} events.
CREATE TABLE IF NOT EXISTS daemon_owner (
  user_id        TEXT PRIMARY KEY NOT NULL,
  bound_at_unix  INTEGER NOT NULL,
  source         TEXT NOT NULL,
  updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Project registry — daemon's record of project paths it has seen.
-- Materialized from project.registered / project.seen / project.archived
-- events. One row per space_id (stable hash of canonical path).
CREATE TABLE IF NOT EXISTS projects (
  id            TEXT PRIMARY KEY NOT NULL,
  path          TEXT NOT NULL,
  name          TEXT NOT NULL,
  created_at    TEXT NOT NULL,
  last_seen_at  TEXT NOT NULL,
  archived_at   TEXT,
  updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
CREATE INDEX IF NOT EXISTS idx_projects_last_seen ON projects(last_seen_at DESC);

-- Project compile state — what was produced by the last `ship compile` for
-- each project. Materialized from `compile.completed` events; the latest
-- event's payload IS the truth. CLI `ship status` reads here; daemon reads
-- here when answering "which agent is active for project X".
CREATE TABLE IF NOT EXISTS project_compile_state (
  space_id          TEXT PRIMARY KEY NOT NULL,
  active_agent        TEXT,
  compiled_at         TEXT NOT NULL,
  ship_version        TEXT NOT NULL,
  harness            TEXT,
  plugins_installed_json TEXT NOT NULL DEFAULT '[]',
  updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

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
  space_id          TEXT,
  cwd                 TEXT,
  parent_actor_id     TEXT,
  restart_count       INTEGER NOT NULL DEFAULT 0,
  agent_id            TEXT,
  harness            TEXT,
  model               TEXT,
  transport           TEXT,
  persistence         TEXT,
  harness_session_id TEXT,
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
CREATE INDEX IF NOT EXISTS idx_actors_project    ON actors(space_id);
CREATE INDEX IF NOT EXISTS idx_actors_status     ON actors(status);
CREATE INDEX IF NOT EXISTS idx_actors_harness   ON actors(harness);
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
