//! Actor state projection — derived from actor.* events.
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
