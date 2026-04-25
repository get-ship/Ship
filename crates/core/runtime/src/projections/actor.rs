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
    parent_actor_id: Option<String>,
    #[serde(default)]
    restart_count: u32,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    provider: Option<String>,
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
        sqlx::query(
            "INSERT OR REPLACE INTO actors \
             (id, kind, environment_type, status, workspace_id, parent_actor_id, \
              restart_count, agent_id, provider, model, transport, persistence, \
              system_prompt_hash, pid, capabilities_json, \
              created_at, updated_at, last_event_at) \
             VALUES (?, ?, ?, 'created', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, \
                     datetime('now'), datetime('now'), datetime('now'))",
        )
        .bind(entity_id)
        .bind(&p.kind)
        .bind(&p.environment_type)
        .bind(&p.workspace_id)
        .bind(&p.parent_actor_id)
        .bind(p.restart_count as i64)
        .bind(&p.agent_id)
        .bind(&p.provider)
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
    pub agent_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub transport: Option<String>,
    pub persistence: Option<String>,
    pub provider_session_id: Option<String>,
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
    "id, kind, status, workspace_id, agent_id, provider, model, \
     transport, persistence, provider_session_id, system_prompt_hash, \
     pid, capabilities_json, parent_actor_id, restart_count, exit_code, \
     exit_reason, duration_ms, created_at, updated_at, last_event_at";

fn row_to_actor(row: &sqlx::sqlite::SqliteRow) -> ActorRow {
    use sqlx::Row;
    let caps_json: String = row
        .try_get("capabilities_json")
        .unwrap_or_else(|_| "[]".into());
    let capabilities: Vec<String> = serde_json::from_str(&caps_json).unwrap_or_default();
    ActorRow {
        id: row.try_get("id").unwrap_or_default(),
        kind: row.try_get("kind").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        workspace_id: row.try_get("workspace_id").ok(),
        agent_id: row.try_get("agent_id").ok(),
        provider: row.try_get("provider").ok(),
        model: row.try_get("model").ok(),
        transport: row.try_get("transport").ok(),
        persistence: row.try_get("persistence").ok(),
        provider_session_id: row.try_get("provider_session_id").ok(),
        system_prompt_hash: row.try_get("system_prompt_hash").ok(),
        pid: row.try_get("pid").ok(),
        capabilities,
        parent_actor_id: row.try_get("parent_actor_id").ok(),
        restart_count: row.try_get("restart_count").unwrap_or(0),
        exit_code: row.try_get("exit_code").ok(),
        exit_reason: row.try_get("exit_reason").ok(),
        duration_ms: row.try_get("duration_ms").ok(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
        last_event_at: row.try_get("last_event_at").ok(),
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
