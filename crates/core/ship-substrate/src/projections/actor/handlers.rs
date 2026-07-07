//! Per-event handlers — apply each actor.* event to the `actors` row.

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::db::block_on;

/// Deserialize an actor-event payload, treating a SHAPE mismatch as a
/// fail-closed SKIP rather than an error.
///
/// Actor projections run inside the event-append transaction (S7): a handler
/// `Err` rolls the whole write back. A payload this projection cannot interpret
/// means only "no actor row can be derived from this event" — the durable event
/// must still persist, so we skip the row write and return `None`. Genuine
/// DB-write failures (constraint, I/O) are NOT caught here — they stay hard
/// errors and correctly roll the transaction back.
fn parse_or_skip<P: serde::de::DeserializeOwned>(event_type: &str, payload_json: &str) -> Option<P> {
    match serde_json::from_str::<P>(payload_json) {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::debug!(
                "[projection:actor_state] skipping {}: payload not interpretable as {}: {}",
                event_type,
                std::any::type_name::<P>(),
                e,
            );
            None
        }
    }
}

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

pub(super) fn apply_created(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let Some(p) = parse_or_skip::<CreatedPayload>("actor.created", payload_json) else {
        return Ok(());
    };
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
pub(super) fn apply_process_spawned(payload_json: &str, conn: &mut SqliteConnection) -> Result<()> {
    let Some(payload) =
        parse_or_skip::<crate::events::process::ProcessSpawnedPayload>("process.spawned", payload_json)
    else {
        return Ok(());
    };
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
        sqlx::query(
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

pub(super) fn apply_status(entity_id: &str, status: &str, conn: &mut SqliteConnection) -> Result<()> {
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

pub(super) fn apply_stopped(
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

/// Crash payload. Both fields default so the projection applies whether the
/// crash was emitted by the supervisor (`{error, restart_count}`) or by the
/// kernel's lifecycle path (`{actor_id, reason}` — see `handle_actor_crash`).
#[derive(serde::Deserialize, Default)]
struct CrashedPayload {
    #[serde(default)]
    restart_count: u32,
}

pub(super) fn apply_crashed(
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
