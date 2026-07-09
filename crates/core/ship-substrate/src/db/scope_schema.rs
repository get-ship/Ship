//! Per-scope event-db schema — the single source of truth for what tables each
//! scope's db carries.
//!
//! ## Why this exists
//! Historically one `migrations/` dir was applied identically to every event db
//! (`runtime.db`, `account.db`, every `spaces/<id>/space.db`). So each db carried
//! the full ~18-table kitchen sink — most of it projection tables sitting in the
//! wrong db, permanently empty (The Storage Blueprint headline). This module
//! replaces that with an explicit per-scope split.
//!
//! ## The classification rule
//! All projections register into EVERY store's bus (`default_bus`); a projection's
//! `apply` only fires in the db where a matching event *lands*. An event lands in
//! its `primary_scope`'s db (via `PartitionedSink::route`). So:
//!
//!   **a projection table belongs in the scope(s) whose events its projection
//!   consumes** — NOT "where it happened to sit before".
//!
//! Worked from the event registry (`register_event!`):
//! - Runtime: `actors` (actor.*), `analytics_*` (op.tool_use_*, session.turn.ended,
//!   actor.*), `spaces` (space.opened/space.seen), `pipeline_runs`/
//! - Account: `auth_sessions` (auth.session.*), `daemon_owner` (daemon.owner.*),
//!   `provisioned_grants` (agent.grants.provisioned).
//! - Space: `project_compile_state` (compile.completed).
//!
//! ONE exception to "scope = event primary_scope": `canonical_sessions` aggregates
//! BOTH Space `session.*` and Runtime `session.turn.*` events, so its projection
//! bypasses the bus's per-scope conn and opens its OWN runtime.db connection for
//! every write (and its reader reads runtime.db). It is therefore classified by
//! where `apply()` WRITES — Runtime — not by its (mixed) event scopes.
//!
//! Self-provisioning projections are deliberately ABSENT here: `collection_<id>`
//! and the artifact metadata table are created on demand inside their own `apply`
//! (`CREATE TABLE IF NOT EXISTS`), so they materialize in whichever space.db their
//! Space-scoped events land in. (S5.2 will move the static tables above to the
//! same self-provisioning pattern; until then they are declared here.)
//!
//! `kv_state` is intentionally absent: it is the L2+ KV plane and lives in
//! `kv.db`, not an event db.
//!
//! ## Mechanism
//! The DDL is **idempotent** (`CREATE … IF NOT EXISTS`) and applied directly on
//! every open, with no `_sqlx_migrations` version log — so there is nothing to
//! drift, and a fresh db lands scope-correct in one pass. The DDL is transcribed
//! verbatim from the final assembled schema (post-migration shape), so projection
//! writers see exactly the columns they wrote against before the squash.
//!
//! Upholds inv-db-scope-partition (a scope's db carries only its scope's tables)
//! and inv-db-per-scope-only.

use crate::events::EventScope;

/// Universal base — applied to EVERY event db regardless of scope: the immutable
/// `events` log (envelope + partition tuple + author provenance), its indexes,
/// the immutability trigger, and the per-scope projection `cursors` table.
pub const BASE_DDL: &[&str] = &[
    // The append-only event log. `payload_json` carries the domain data; `scope`
    // + `scope_key` are the partition tuple (D0); `author_*`/`signature` are the
    // Phase-2 author-identity groundwork (signature reserved, NULL until signing).
    r#"CREATE TABLE IF NOT EXISTS events (
  id               TEXT PRIMARY KEY NOT NULL,
  event_type       TEXT NOT NULL,
  entity_id        TEXT NOT NULL,
  actor            TEXT NOT NULL DEFAULT 'ship',
  payload_json     TEXT NOT NULL DEFAULT '{}',
  version          INTEGER NOT NULL DEFAULT 1,
  causation_id     TEXT,
  scope            TEXT NOT NULL DEFAULT 'device',
  scope_key        TEXT,
  session_id       TEXT,
  actor_id         TEXT,
  parent_actor_id  TEXT,
  elevated         INTEGER NOT NULL DEFAULT 0,
  job_id           TEXT,
  agent_id         TEXT,
  target_actor_id  TEXT,
  author_account_id TEXT,
  author_device_id  TEXT,
  signature         TEXT,
  created_at       TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
    "CREATE INDEX IF NOT EXISTS idx_events_actor   ON events(actor_id, elevated)",
    "CREATE INDEX IF NOT EXISTS idx_events_entity  ON events(entity_id, event_type)",
    "CREATE INDEX IF NOT EXISTS idx_events_job     ON events(job_id)",
    "CREATE INDEX IF NOT EXISTS idx_events_scope   ON events(scope, scope_key, created_at)",
    "CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id)",
    // The events table is append-only; UPDATE is rejected (inv-db-immutable-events).
    r#"CREATE TRIGGER IF NOT EXISTS events_immutable
BEFORE UPDATE ON events
BEGIN
  SELECT RAISE(FAIL, 'events table is immutable -- use append only');
END"#,
    // Projection read-cursors. Present in every per-scope db so each scope's
    // projections can track replay position independently.
    r#"CREATE TABLE IF NOT EXISTS cursors (
  name           TEXT PRIMARY KEY,
  last_event_id  TEXT NOT NULL,
  updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
];

/// The `durable_journals` table DDL — the single source. Part of `RUNTIME_DDL`
/// (materialized in `runtime.db`) AND ensured directly by the `journal-sqlite`
/// leaf on fresh test dbs, so the schema is defined once.
pub const DURABLE_JOURNALS_DDL: &str = r#"CREATE TABLE IF NOT EXISTS durable_journals (
  run_id       TEXT PRIMARY KEY,
  journal_json TEXT NOT NULL,
  updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
)"#;

/// Runtime projections — applied ONLY to `runtime.db`. Per-machine execution
/// state: actors, analytics, pipelines, the project registry; plus the
/// direct-write `daemon_settings` and the pipeline `durable_journals` store.
pub const RUNTIME_DDL: &[&str] = &[
    // ── actors (actor.* events) ─────────────────────────────────────────────────
    r#"CREATE TABLE IF NOT EXISTS actors (
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
)"#,
    "CREATE INDEX IF NOT EXISTS idx_actors_agent_id   ON actors(agent_id)",
    "CREATE INDEX IF NOT EXISTS idx_actors_harness    ON actors(harness)",
    "CREATE INDEX IF NOT EXISTS idx_actors_last_event ON actors(last_event_at)",
    "CREATE INDEX IF NOT EXISTS idx_actors_project    ON actors(space_id)",
    "CREATE INDEX IF NOT EXISTS idx_actors_status     ON actors(status)",
    "CREATE INDEX IF NOT EXISTS idx_actors_workspace  ON actors(workspace_id)",
    // The two actor activity-bump triggers fire on event append; they UPDATE
    // `actors`, so they belong wherever actor.* events land (runtime.db) and
    // nowhere else (account/space dbs have no `actors` table).
    r#"CREATE TRIGGER IF NOT EXISTS bump_actor_last_event_on_actor_id
AFTER INSERT ON events
WHEN NEW.actor_id IS NOT NULL
BEGIN
  UPDATE actors SET last_event_at = NEW.created_at WHERE id = NEW.actor_id;
END"#,
    r#"CREATE TRIGGER IF NOT EXISTS bump_actor_last_event_on_target
AFTER INSERT ON events
WHEN NEW.target_actor_id IS NOT NULL
BEGIN
  UPDATE actors SET last_event_at = NEW.created_at WHERE id = NEW.target_actor_id;
END"#,
    // ── analytics (op.tool_use_*, session.turn.ended, actor.*) ───────────────────
    r#"CREATE TABLE IF NOT EXISTS analytics_sessions (
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
)"#,
    "CREATE INDEX IF NOT EXISTS idx_analytics_sessions_actor   ON analytics_sessions(actor_id)",
    "CREATE INDEX IF NOT EXISTS idx_analytics_sessions_harness ON analytics_sessions(harness)",
    "CREATE INDEX IF NOT EXISTS idx_analytics_sessions_started ON analytics_sessions(started_at)",
    r#"CREATE TABLE IF NOT EXISTS analytics_tool_uses (
  id           TEXT PRIMARY KEY NOT NULL,
  session_key  TEXT NOT NULL,
  actor_id     TEXT NOT NULL,
  tool         TEXT NOT NULL,
  status       TEXT,
  started_at   TEXT NOT NULL,
  ended_at     TEXT,
  duration_ms  INTEGER,
  error        TEXT
)"#,
    "CREATE INDEX IF NOT EXISTS idx_analytics_tool_uses_session ON analytics_tool_uses(session_key)",
    "CREATE INDEX IF NOT EXISTS idx_analytics_tool_uses_started ON analytics_tool_uses(started_at)",
    "CREATE INDEX IF NOT EXISTS idx_analytics_tool_uses_tool    ON analytics_tool_uses(tool)",
    r#"CREATE TABLE IF NOT EXISTS analytics_turns (
  id                       TEXT PRIMARY KEY NOT NULL,
  session_key              TEXT NOT NULL,
  actor_id                 TEXT NOT NULL,
  turn_idx                 INTEGER,
  model                    TEXT,
  input_tokens             INTEGER NOT NULL DEFAULT 0,
  output_tokens            INTEGER NOT NULL DEFAULT 0,
  reasoning_tokens         INTEGER NOT NULL DEFAULT 0,
  cache_creation_tokens    INTEGER NOT NULL DEFAULT 0,
  cache_read_tokens        INTEGER NOT NULL DEFAULT 0,
  stop_reason              TEXT,
  observed_at              TEXT NOT NULL,
  harness_reported_cost_usd REAL
)"#,
    "CREATE INDEX IF NOT EXISTS idx_analytics_turns_model    ON analytics_turns(model)",
    "CREATE INDEX IF NOT EXISTS idx_analytics_turns_observed ON analytics_turns(observed_at)",
    "CREATE INDEX IF NOT EXISTS idx_analytics_turns_session  ON analytics_turns(session_key)",
    // ── daemon_settings (direct-write, runtime.db) ───────────────────────────────
    r#"CREATE TABLE IF NOT EXISTS daemon_settings (
  key         TEXT PRIMARY KEY NOT NULL,
  value_json  TEXT NOT NULL,
  updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
    // ── durable_journals (pipeline journal store) ────────────────────────────────
    // DDL lives in `DURABLE_JOURNALS_DDL` (below) so the `journal-sqlite` leaf can
    // ensure the same table on fresh test dbs from ONE source.
    DURABLE_JOURNALS_DDL,
    // ── pipeline_runs / pipeline_step_runs (pipeline.* events) ───────────────────
    // `pipeline_runs` is a quoted identifier and its indexes keep their legacy
    // `idx_jobs_*` names (renamed-table lineage); preserved verbatim.
    r#"CREATE TABLE IF NOT EXISTS "pipeline_runs" (
  pipeline_run_id              TEXT PRIMARY KEY,
  slug                TEXT NOT NULL,
  agent               TEXT NOT NULL,
  branch              TEXT NOT NULL,
  spec_path           TEXT NOT NULL,
  source_spec_path    TEXT,
  depends_on_json     TEXT,
  status              TEXT NOT NULL,
  worktree            TEXT,
  blocker             TEXT,
  error               TEXT,
  model               TEXT,
  harness            TEXT,
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL,
  harness_meta_json  TEXT,
  pipeline_json       TEXT,
  steps_json          TEXT,
  current_phase       INTEGER,
  completed_steps_json TEXT NOT NULL DEFAULT '[]',
  current_step        TEXT,
  vars_json           TEXT,
  step_retries_json   TEXT NOT NULL DEFAULT '{}',
  headless            INTEGER NOT NULL DEFAULT 0,
  baseline_sha        TEXT
)"#,
    r#"CREATE INDEX IF NOT EXISTS idx_jobs_slug ON "pipeline_runs"(slug)"#,
    r#"CREATE INDEX IF NOT EXISTS idx_jobs_status ON "pipeline_runs"(status)"#,
    r#"CREATE TABLE IF NOT EXISTS pipeline_step_runs (
  run_id          TEXT NOT NULL,
  step_id         TEXT NOT NULL,
  step_type       TEXT NOT NULL DEFAULT 'agent',
  status          TEXT NOT NULL DEFAULT 'pending',
  started_at      TEXT,
  completed_at    TEXT,
  depends_on_json TEXT,
  attempt         INTEGER NOT NULL DEFAULT 1,
  session_id      TEXT,
  note            TEXT,
  updated_at      TEXT NOT NULL,
  PRIMARY KEY (run_id, step_id)
)"#,
    "CREATE INDEX IF NOT EXISTS idx_pipeline_step_runs_run     ON pipeline_step_runs(run_id)",
    "CREATE INDEX IF NOT EXISTS idx_pipeline_step_runs_session ON pipeline_step_runs(session_id)",
    "CREATE INDEX IF NOT EXISTS idx_pipeline_step_runs_status  ON pipeline_step_runs(status)",
    // ── spaces (space.opened / space.seen) ─────────────────────────────────
    // The runtime-scoped `spaces` projection — the device-local record of opened
    // spaces, folded from the `space.*` events ship-space emits. Identity
    // (space_id) lives here, minted ONCE in ship-space::identity from a
    // nanoid, never derived from the path. `path` is the location on this
    // device (the projection's existing columns are the LaunchServices
    // analogy); it can change with a future `space.moved` event without
    // changing identity.
    r#"CREATE TABLE IF NOT EXISTS spaces (
  space_id      TEXT PRIMARY KEY NOT NULL,
  path          TEXT NOT NULL,
  name          TEXT NOT NULL,
  opened_at     TEXT NOT NULL,
  last_seen_at  TEXT NOT NULL,
  updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
    "CREATE INDEX IF NOT EXISTS idx_spaces_last_seen ON spaces(last_seen_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_spaces_path      ON spaces(path)",
    // ── canonical_sessions ──────────────────────────────────────────────────────
    // The agent-session ledger. Special case: it aggregates BOTH Space-scoped
    // session.* AND Runtime-scoped session.turn.* events, so CanonicalSessionProjection
    // deliberately bypasses the per-scope bus conn (which may be a space.db) and
    // opens its OWN runtime.db connection for every write. Its reader reads
    // runtime.db. So the table is Runtime — classified by where apply() WRITES,
    // not by the (mixed) scope of its events.
    r#"CREATE TABLE IF NOT EXISTS canonical_sessions (
  session_id   TEXT PRIMARY KEY NOT NULL,
  space_id     TEXT NOT NULL,
  agent_id     TEXT NOT NULL,
  harness      TEXT NOT NULL,
  model        TEXT,
  status       TEXT NOT NULL,
  input        TEXT,
  parent       TEXT,
  turn_count   INTEGER NOT NULL DEFAULT 0,
  started_at   TEXT NOT NULL,
  ended_at     TEXT,
  outcome      TEXT,
  reason       TEXT,
  slug TEXT
)"#,
    "CREATE INDEX IF NOT EXISTS idx_canonical_sessions_agent  ON canonical_sessions(agent_id)",
    "CREATE INDEX IF NOT EXISTS idx_canonical_sessions_slug   ON canonical_sessions(slug)",
    "CREATE INDEX IF NOT EXISTS idx_canonical_sessions_space  ON canonical_sessions(space_id)",
    "CREATE INDEX IF NOT EXISTS idx_canonical_sessions_status ON canonical_sessions(status)",
];

/// Account projections — applied ONLY to `account.db`. The portable identity
/// plane (cloud-synced): auth sessions, daemon ownership binding, and the
/// provisioned restricted-domain grants.
pub const ACCOUNT_DDL: &[&str] = &[
    // auth_sessions (auth.session.created/revoked/expired)
    r#"CREATE TABLE IF NOT EXISTS auth_sessions (
  token            TEXT PRIMARY KEY NOT NULL,
  agent_id         TEXT NOT NULL,
  scopes_json      TEXT NOT NULL,
  space_id       TEXT,
  created_at_unix  INTEGER NOT NULL,
  expires_at_unix  INTEGER NOT NULL,
  revoked_at_unix  INTEGER,
  source           TEXT NOT NULL,
  updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
    "CREATE INDEX IF NOT EXISTS idx_auth_sessions_active ON auth_sessions(token) WHERE revoked_at_unix IS NULL",
    // daemon_owner (daemon.owner.bound/unbound)
    r#"CREATE TABLE IF NOT EXISTS daemon_owner (
  user_id        TEXT PRIMARY KEY NOT NULL,
  bound_at_unix  INTEGER NOT NULL,
  source         TEXT NOT NULL,
  updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
    // provisioned_grants (agent.grants.provisioned)
    r#"CREATE TABLE IF NOT EXISTS provisioned_grants (
  agent_id     TEXT PRIMARY KEY NOT NULL,
  grants_json  TEXT NOT NULL,
  source       TEXT NOT NULL,
  approved_by  TEXT,
  updated_at   TEXT NOT NULL
)"#,
];

/// Space projections — applied to every `spaces/<id>/space.db`. The per-space
/// compile state. (canonical_sessions is NOT here: its projection bypasses the
/// per-scope bus conn and writes to runtime.db — see RUNTIME_DDL. Collections +
/// artifact metadata are NOT here either — their projections self-provision.)
pub const SPACE_DDL: &[&str] = &[
    // project_compile_state (compile.completed)
    r#"CREATE TABLE IF NOT EXISTS project_compile_state (
  space_id          TEXT PRIMARY KEY NOT NULL,
  active_agent        TEXT,
  compiled_at         TEXT NOT NULL,
  ship_version        TEXT NOT NULL,
  harness            TEXT,
  plugins_installed_json TEXT NOT NULL DEFAULT '[]',
  updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
)"#,
];

/// The ordered, idempotent DDL for a given event scope: the universal base, plus
/// exactly that scope's projection tables. Order is dependency-safe (tables
/// precede their indexes/triggers; `actors` precedes the `bump_actor_*`
/// triggers).
pub fn scope_schema(scope: EventScope) -> Vec<&'static str> {
    let mut ddl: Vec<&'static str> = BASE_DDL.to_vec();
    let scoped = match scope {
        EventScope::Runtime => RUNTIME_DDL,
        EventScope::Account => ACCOUNT_DDL,
        EventScope::Space => SPACE_DDL,
    };
    ddl.extend_from_slice(scoped);
    ddl
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, Row, SqliteConnection};

    /// Apply a scope's DDL to a fresh in-memory db and return the set of user
    /// table names (excludes sqlite internal + index/trigger objects).
    async fn tables_for(scope: EventScope) -> Vec<String> {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        for stmt in scope_schema(scope) {
            sqlx::query(stmt)
                .execute(&mut conn)
                .await
                .unwrap_or_else(|e| panic!("DDL failed: {e}\n--- stmt ---\n{stmt}"));
        }
        let rows = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .fetch_all(&mut conn)
        .await
        .unwrap();
        rows.into_iter().map(|r| r.get::<String, _>("name")).collect()
    }

    fn base() -> Vec<String> {
        vec!["cursors".to_string(), "events".to_string()]
    }

    #[tokio::test]
    async fn runtime_carries_runtime_projections_only() {
        let tables = tables_for(EventScope::Runtime).await;
        for expected in [
            "events",
            "cursors",
            "actors",
            "analytics_sessions",
            "analytics_tool_uses",
            "analytics_turns",
            "daemon_settings",
            "durable_journals",
            "pipeline_runs",
            "pipeline_step_runs",
            "spaces",
            "canonical_sessions",
        ] {
            assert!(tables.contains(&expected.to_string()), "runtime missing {expected}; got {tables:?}");
        }
        // Account/Space tables must NOT leak into runtime.db.
        for forbidden in ["auth_sessions", "daemon_owner", "provisioned_grants", "project_compile_state", "kv_state"] {
            assert!(!tables.contains(&forbidden.to_string()), "{forbidden} must not be in runtime.db");
        }
    }

    #[tokio::test]
    async fn account_carries_account_projections_only() {
        let mut tables = tables_for(EventScope::Account).await;
        tables.sort();
        let mut want = base();
        want.extend(["auth_sessions", "daemon_owner", "provisioned_grants"].map(String::from));
        want.sort();
        assert_eq!(tables, want, "account.db = base + auth_sessions/daemon_owner/provisioned_grants only");
    }

    #[tokio::test]
    async fn space_carries_space_projections_only() {
        let mut tables = tables_for(EventScope::Space).await;
        tables.sort();
        let mut want = base();
        want.extend(["project_compile_state"].map(String::from));
        want.sort();
        assert_eq!(tables, want, "space.db = base + project_compile_state only");
    }

    #[tokio::test]
    async fn no_table_is_in_two_scopes() {
        // Each projection table belongs to exactly one scope (base excepted) —
        // catches a copy-paste leak across the per-scope DDL sets.
        let runtime: std::collections::HashSet<_> = tables_for(EventScope::Runtime).await.into_iter().collect();
        let account: std::collections::HashSet<_> = tables_for(EventScope::Account).await.into_iter().collect();
        let space: std::collections::HashSet<_> = tables_for(EventScope::Space).await.into_iter().collect();
        let base: std::collections::HashSet<_> = base().into_iter().collect();
        let r = &runtime - &base;
        let a = &account - &base;
        let s = &space - &base;
        assert!(r.is_disjoint(&a), "runtime/account overlap: {:?}", &r & &a);
        assert!(r.is_disjoint(&s), "runtime/space overlap: {:?}", &r & &s);
        assert!(a.is_disjoint(&s), "account/space overlap: {:?}", &a & &s);
    }

    #[tokio::test]
    async fn schema_is_idempotent_for_every_scope() {
        // Applying each scope's schema twice must succeed (every statement is
        // CREATE … IF NOT EXISTS) — the mechanism re-applies on every open.
        for scope in [EventScope::Runtime, EventScope::Account, EventScope::Space] {
            let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
            for _pass in 0..2 {
                for stmt in scope_schema(scope) {
                    sqlx::query(stmt)
                        .execute(&mut conn)
                        .await
                        .unwrap_or_else(|e| panic!("re-apply failed for {scope:?}: {e}\n{stmt}"));
                }
            }
        }
    }
}
