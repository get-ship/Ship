//! Actor state projection — derived from actor.* events.
//!
//! Read API: `list_actors` and `get_actor` project the actors table for
//! the unified dashboard. Filterable by workspace, kind, status; ordered
//! by `last_event_at` desc so the most recently active actors surface
//! first.
//!
//! This projection maintains the `actors` table in per-workspace DBs as a read
//! model. It handles every actor event type and applies the state change.
//! On rebuild, the table is truncated and replayed from the event log.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::async_projection::AsyncProjection;
use super::registry::Projection;
use crate::db::block_on;
use crate::events::types::event_types;
use crate::events::EventEnvelope;

/// Projection that maintains the actors table from actor.* events.
#[derive(Default)]
pub struct ActorProjection;

impl ActorProjection {
    pub fn new() -> Self {
        Self
    }
}

const HANDLED: &[&str] = &[
    event_types::ACTOR_CREATED,
    event_types::ACTOR_WOKE,
    event_types::ACTOR_SLEPT,
    event_types::ACTOR_STOPPED,
    event_types::ACTOR_CRASHED,
    // Cross-domain: process.spawned carries the real pid the supervisor
    // captures AFTER the OS spawn returns. actor.created (emitted before
    // spawn from emit_actor_created_simple) doesn't have it. Apply the
    // pid from process.spawned to the matching actor row so live = pid_alive
    // works honestly instead of falling back to the activity_live timer.
    crate::events::process::event_types::PROCESS_SPAWNED,
];

impl Projection for ActorProjection {
    fn name(&self) -> &str {
        "actor_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        let entity = &event.entity_id;
        match event.event_type.as_str() {
            event_types::ACTOR_CREATED => apply_created(entity, &event.payload_json, conn),
            event_types::ACTOR_WOKE => apply_status(entity, "active", conn),
            event_types::ACTOR_SLEPT => apply_status(entity, "sleeping", conn),
            event_types::ACTOR_STOPPED => apply_stopped(entity, &event.payload_json, conn),
            event_types::ACTOR_CRASHED => apply_crashed(entity, &event.payload_json, conn),
            t if t == crate::events::process::event_types::PROCESS_SPAWNED => {
                apply_process_spawned(&event.payload_json, conn)
            }
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM actors").execute(conn).await?;
            Ok(())
        })
    }
}

impl AsyncProjection for ActorProjection {
    fn name(&self) -> &str {
        Projection::name(self)
    }
    fn event_types(&self) -> &[&str] {
        Projection::event_types(self)
    }
    fn apply(&self, event: &EventEnvelope, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::apply(self, event, conn)
    }
    fn truncate(&self, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::truncate(self, conn)
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct CreatedPayload {
    kind: String,
    environment_type: String,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    parent_actor_id: Option<String>,
    #[serde(default)]
    restart_count: u32,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    harness: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    transport: Option<String>,
    #[serde(default)]
    persistence: Option<String>,
    #[serde(default)]
    system_prompt_hash: Option<String>,
    #[serde(default)]
    pid: Option<u32>,
    #[serde(default)]
    capabilities: Vec<String>,
}

fn apply_created(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CreatedPayload = serde_json::from_str(payload_json)?;
    let capabilities_json = serde_json::to_string(&p.capabilities).unwrap_or_else(|_| "[]".into());
    block_on(async {
        // ON CONFLICT(id) DO UPDATE — NOT INSERT OR REPLACE — because a
        // second actor.created for the same id (e.g. fire-and-forget
        // emit racing a reconcile) must NOT trample fields written by
        // intervening events. Specifically: `pid` is set by
        // apply_process_spawned AFTER spawn returns; the actor.created
        // payload only carries pid:None. INSERT OR REPLACE wiped the
        // pid back to NULL on every duplicate emit, breaking actor.live
        // for dispatched sessions. Keep created_at + last_event_at from
        // the original row, refresh updated_at only.
        sqlx::query(
            "INSERT INTO actors \
             (id, kind, environment_type, status, workspace_id, space_id, cwd, \
              parent_actor_id, restart_count, agent_id, harness, model, transport, \
              persistence, system_prompt_hash, pid, capabilities_json, \
              created_at, updated_at, last_event_at) \
             VALUES (?, ?, ?, 'created', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, \
                     datetime('now'), datetime('now'), datetime('now')) \
             ON CONFLICT(id) DO UPDATE SET \
                 kind = excluded.kind, \
                 environment_type = excluded.environment_type, \
                 workspace_id = excluded.workspace_id, \
                 space_id = excluded.space_id, \
                 cwd = excluded.cwd, \
                 parent_actor_id = excluded.parent_actor_id, \
                 agent_id = excluded.agent_id, \
                 harness = excluded.harness, \
                 model = excluded.model, \
                 transport = excluded.transport, \
                 persistence = excluded.persistence, \
                 system_prompt_hash = excluded.system_prompt_hash, \
                 capabilities_json = excluded.capabilities_json, \
                 updated_at = datetime('now')",
        )
        .bind(entity_id)
        .bind(&p.kind)
        .bind(&p.environment_type)
        .bind(&p.workspace_id)
        .bind(&p.space_id)
        .bind(&p.cwd)
        .bind(&p.parent_actor_id)
        .bind(p.restart_count as i64)
        .bind(&p.agent_id)
        .bind(&p.harness)
        .bind(&p.model)
        .bind(&p.transport)
        .bind(&p.persistence)
        .bind(&p.system_prompt_hash)
        .bind(p.pid.map(|v| v as i64))
        .bind(&capabilities_json)
        .execute(conn)
        .await?;
        Ok(())
    })
}

/// Update an actor row's pid from a `process.spawned` event. The
/// supervisor emits this AFTER the OS spawn returns, so it carries the
/// real child pid (where `actor.created` couldn't — it fires earlier).
/// Keyed by the payload's `slug`, which matches the actor row id for
/// dispatched sessions.
///
/// Idempotent: if no actor row exists for the slug yet, the UPDATE is
/// a no-op. Pre-existing rows get pid filled in even if it was None.
fn apply_process_spawned(payload_json: &str, conn: &mut SqliteConnection) -> Result<()> {
    let payload: crate::events::process::ProcessSpawnedPayload =
        serde_json::from_str(payload_json)?;
    let pid = match payload.pid {
        Some(p) if p > 0 => p as i64,
        _ => return Ok(()),
    };
    block_on(async {
        // UPSERT: process.spawned can land BEFORE actor.created in the
        // projection bus (actor.created is fire-and-forget from a
        // std::thread spawn in routing.rs to avoid tokio deadlock; the
        // synchronous emit_process_spawned can race ahead). When the
        // row doesn't exist yet, INSERT a stub with the pid; the later
        // actor.created lands via ON CONFLICT DO UPDATE (apply_created)
        // and fills in the metadata without trampling pid.
        let _ = sqlx::query(
            "INSERT INTO actors (id, kind, pid, status, environment_type, capabilities_json) \
             VALUES (?, 'pipeline-step', ?, 'created', 'local', '[]') \
             ON CONFLICT(id) DO UPDATE SET pid = excluded.pid, updated_at = datetime('now')"
        )
            .bind(&payload.slug)
            .bind(pid)
            .execute(conn)
            .await?;
        Ok(())
    })
}

fn apply_status(entity_id: &str, status: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query("UPDATE actors SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(status)
            .bind(entity_id)
            .execute(conn)
            .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct StoppedPayload {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    exit_code: Option<i32>,
    #[serde(default)]
    duration_ms: Option<u64>,
}

fn apply_stopped(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: StoppedPayload = serde_json::from_str(payload_json).unwrap_or(StoppedPayload {
        reason: String::new(),
        exit_code: None,
        duration_ms: None,
    });
    block_on(async {
        sqlx::query(
            "UPDATE actors SET status = 'stopped', exit_reason = ?, exit_code = ?, \
             duration_ms = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&p.reason)
        .bind(p.exit_code)
        .bind(p.duration_ms.map(|v| v as i64))
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct CrashedPayload {
    #[allow(dead_code)]
    error: String,
    restart_count: u32,
}

fn apply_crashed(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CrashedPayload = serde_json::from_str(payload_json)?;
    block_on(async {
        sqlx::query(
            "UPDATE actors SET status = 'crashed', restart_count = ?, \
             updated_at = datetime('now') WHERE id = ?",
        )
        .bind(p.restart_count as i64)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

// ── Read API ─────────────────────────────────────────────────────────────────

/// Read-side row shape returned by `list_actors` / `get_actor`. JSON-friendly,
/// transport-agnostic — the HTTP layer can pass it through directly. All
/// columns mirror the `actors` table after migration 0006.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActorRow {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub workspace_id: Option<String>,
    pub space_id: Option<String>,
    pub cwd: Option<String>,
    pub agent_id: Option<String>,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub transport: Option<String>,
    pub persistence: Option<String>,
    pub harness_session_id: Option<String>,
    pub system_prompt_hash: Option<String>,
    pub pid: Option<i64>,
    pub capabilities: Vec<String>,
    pub parent_actor_id: Option<String>,
    pub restart_count: i64,
    pub exit_code: Option<i64>,
    pub exit_reason: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub last_event_at: Option<String>,
    /// Derived: process alive right now (kill -0 probe via /proc).
    /// `false` for dead pids and rows without a pid (services).
    pub live: bool,
}

/// Filter for `list_actors`. All `Option`s default to "no filter."
#[derive(Default, Debug, Clone)]
pub struct ListActorsFilter {
    pub workspace_id: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub limit: i64,
}

const SELECT_COLS: &str =
    "id, kind, status, workspace_id, space_id, cwd, agent_id, harness, model, \
     transport, persistence, harness_session_id, system_prompt_hash, \
     pid, capabilities_json, parent_actor_id, restart_count, exit_code, \
     exit_reason, duration_ms, created_at, updated_at, last_event_at";

fn row_to_actor(row: &sqlx::sqlite::SqliteRow) -> ActorRow {
    use sqlx::Row;
    let caps_json: String = row
        .try_get("capabilities_json")
        .unwrap_or_else(|_| "[]".into());
    let capabilities: Vec<String> = serde_json::from_str(&caps_json).unwrap_or_default();
    // SQLite + sqlx sometimes surfaces a NULL pid column as `Some(0)`
    // here (the binding side writes NULL, but the read path collapses).
    // pid 0 is the kernel swapper, never a real child — treat it as
    // "no pid recorded" so `activity_live` can take over for actors
    // whose pid hasn't been wired into the projection yet (Task #14).
    let pid: Option<i64> = row.try_get("pid").ok().filter(|&p| p > 0);
    let status: String = row.try_get("status").unwrap_or_default();
    let last_event_at: Option<String> = row.try_get("last_event_at").ok();
    // Liveness sources, in order of confidence:
    //   1. pid recorded → /proc probe (definitive for daemon-spawned actors)
    //   2. no pid → recent activity (within 90s) means the mailbox is
    //      probably still draining (used by MCP-bridge actors that don't
    //      advertise their pid via actor_spawn).
    let pid_live = pid.map(pid_alive).unwrap_or(false);
    let activity_live = pid.is_none()
        && last_event_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|t| (chrono::Utc::now() - t.with_timezone(&chrono::Utc)).num_seconds() < 90)
            .unwrap_or(false);
    let live = (pid_live || activity_live)
        && status != "stopped"
        && status != "crashed";
    ActorRow {
        id: row.try_get("id").unwrap_or_default(),
        kind: row.try_get("kind").unwrap_or_default(),
        status,
        workspace_id: row.try_get("workspace_id").ok(),
        space_id: row.try_get("space_id").ok(),
        cwd: row.try_get("cwd").ok(),
        agent_id: row.try_get("agent_id").ok(),
        harness: row.try_get("harness").ok(),
        model: row.try_get("model").ok(),
        transport: row.try_get("transport").ok(),
        persistence: row.try_get("persistence").ok(),
        harness_session_id: row.try_get("harness_session_id").ok(),
        system_prompt_hash: row.try_get("system_prompt_hash").ok(),
        pid,
        capabilities,
        parent_actor_id: row.try_get("parent_actor_id").ok(),
        restart_count: row.try_get("restart_count").unwrap_or(0),
        exit_code: row.try_get("exit_code").ok(),
        exit_reason: row.try_get("exit_reason").ok(),
        duration_ms: row.try_get("duration_ms").ok(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
        last_event_at: row.try_get("last_event_at").ok(),
        live,
    }
}

pub fn list_actors(filter: &ListActorsFilter) -> Result<Vec<ActorRow>> {
    let mut conn = crate::db::open_db()?;
    let limit = if filter.limit > 0 { filter.limit } else { 200 }.min(1000);
    let mut sql = format!(
        "SELECT {SELECT_COLS} FROM actors WHERE 1=1"
    );
    if filter.workspace_id.is_some() {
        sql.push_str(" AND workspace_id = ?");
    }
    if filter.kind.is_some() {
        sql.push_str(" AND kind = ?");
    }
    if filter.status.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY COALESCE(last_event_at, created_at) DESC LIMIT ?");

    block_on(async {
        let mut q = sqlx::query(&sql);
        if let Some(w) = &filter.workspace_id {
            q = q.bind(w);
        }
        if let Some(k) = &filter.kind {
            q = q.bind(k);
        }
        if let Some(s) = &filter.status {
            q = q.bind(s);
        }
        q = q.bind(limit);
        let rows = q.fetch_all(&mut conn).await?;
        Ok(rows.iter().map(row_to_actor).collect())
    })
}

pub fn get_actor(actor_id: &str) -> Result<Option<ActorRow>> {
    let mut conn = crate::db::open_db()?;
    let sql = format!("SELECT {SELECT_COLS} FROM actors WHERE id = ?");
    block_on(async {
        let row = sqlx::query(&sql).bind(actor_id).fetch_optional(&mut conn).await?;
        Ok(row.as_ref().map(row_to_actor))
    })
}

/// Check if a pid exists. Used to derive real liveness for the dashboard
/// since pipeline `actor.stopped` events don't fire today and chat actor
/// `created` status is sticky after process exit.
///
/// Linux: `/proc/<pid>` exists iff the process is alive.
/// Single-instance source of truth for the multi-window shell: the live
/// `window`-kind actor hosting `space_id`, if any. The launcher consults this
/// to focus an existing window instead of spawning a duplicate. Windows are
/// self-registering lifecycle actors (`ActorKind::Window`); `live` is the
/// supervisor's pid probe.
pub fn find_live_window_for_space(space_id: &str) -> Result<Option<ActorRow>> {
    // "window" == ActorKind::Window.as_str(); the projection treats kind as a
    // string throughout, so we filter on the string (no supervisor dep here).
    let windows = list_actors(&ListActorsFilter {
        kind: Some("window".to_string()),
        limit: 256,
        ..Default::default()
    })?;
    Ok(windows
        .into_iter()
        .find(|r| r.live && r.space_id.as_deref() == Some(space_id)))
}

pub fn pid_alive(pid: i64) -> bool {
    if pid <= 0 {
        return false;
    }
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// One row from the event log for a given actor. Strongly-typed wrapper
/// over the events table — avoids leaking sqlx types to HTTP handlers.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActorEventRow {
    pub id: String,
    pub event_type: String,
    pub entity_id: String,
    pub actor_id: Option<String>,
    pub target_actor_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub payload_json: String,
    pub created_at: String,
}

/// Read events for an actor since a given event id. Used by the
/// reconnect / replay path: client tracks the last id it saw, requests
/// `?from_id=<id>` on reconnect, kernel streams everything after.
///
/// Matches both events the actor emitted (`actor_id = id`) and events
/// directed at it (`target_actor_id = id`). Ordered by id ascending so
/// the replay arrives in causal order.
pub fn list_actor_events(
    actor_id: &str,
    from_id: Option<&str>,
    limit: i64,
) -> Result<Vec<ActorEventRow>> {
    let mut conn = crate::db::open_db()?;
    let limit = if limit > 0 { limit } else { 500 }.min(2000);
    let mut sql = String::from(
        "SELECT id, event_type, entity_id, actor_id, target_actor_id, \
         workspace_id, session_id, parent_actor_id, payload_json, created_at \
         FROM events \
         WHERE (actor_id = ? OR target_actor_id = ?)",
    );
    if from_id.is_some() {
        sql.push_str(" AND id > ?");
    }
    sql.push_str(" ORDER BY id ASC LIMIT ?");

    block_on(async {
        let mut q = sqlx::query(&sql).bind(actor_id).bind(actor_id);
        if let Some(f) = from_id {
            q = q.bind(f);
        }
        q = q.bind(limit);
        let rows = q.fetch_all(&mut conn).await?;
        Ok(rows
            .iter()
            .map(|r| {
                use sqlx::Row;
                ActorEventRow {
                    id: r.try_get("id").unwrap_or_default(),
                    event_type: r.try_get("event_type").unwrap_or_default(),
                    entity_id: r.try_get("entity_id").unwrap_or_default(),
                    actor_id: r.try_get("actor_id").ok(),
                    target_actor_id: r.try_get("target_actor_id").ok(),
                    workspace_id: r.try_get("workspace_id").ok(),
                    session_id: r.try_get("session_id").ok(),
                    parent_actor_id: r.try_get("parent_actor_id").ok(),
                    payload_json: r.try_get("payload_json").unwrap_or_else(|_| "{}".into()),
                    created_at: r.try_get("created_at").unwrap_or_default(),
                }
            })
            .collect())
    })
}

/// Look up `(status, workspace_id)` for routing decisions in the input
/// endpoint. Returns `None` if the actor isn't in the projection.
pub fn actor_status(actor_id: &str) -> Result<Option<(String, Option<String>)>> {
    let mut conn = crate::db::open_db()?;
    block_on(async {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT status, workspace_id FROM actors WHERE id = ?",
        )
        .bind(actor_id)
        .fetch_optional(&mut conn)
        .await?;
        Ok(row)
    })
}
