//! Auth projections — `auth_sessions` and `daemon_owner` tables.
//!
//! Materialized from auth.session.* and daemon.owner.* events. Token lookup
//! on every authed request reads `auth_sessions WHERE token = ? AND
//! revoked_at_unix IS NULL`. The owner row is at most one (single-row
//! projection).

use anyhow::Result;
use sqlx::{Row, SqliteConnection};

use crate::db::block_on;
use crate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{
    AUTH_SESSION_CREATED, AUTH_SESSION_EXPIRED, AUTH_SESSION_REVOKED, DAEMON_OWNER_BOUND,
    DAEMON_OWNER_UNBOUND,
};
use ship_types::payload::{
    AuthSessionCreated, AuthSessionExpired, AuthSessionRevoked, DaemonOwnerBound,
    DaemonOwnerUnbound,
};

// ── Sessions ────────────────────────────────────────────────────────────────

/// Read-model row for `auth_sessions`. Caller-facing copy of the projection.
#[derive(Debug, Clone)]
pub struct AuthSessionRow {
    pub token: String,
    pub agent_id: String,
    pub scopes_json: String,
    pub project_id: Option<String>,
    pub created_at_unix: i64,
    pub expires_at_unix: i64,
    pub revoked_at_unix: Option<i64>,
    pub source: String,
}

pub struct AuthSessionsProjection;

impl Projection for AuthSessionsProjection {
    fn name(&self) -> &str {
        "auth_sessions"
    }

    fn event_types(&self) -> &[&str] {
        &[
            AUTH_SESSION_CREATED,
            AUTH_SESSION_REVOKED,
            AUTH_SESSION_EXPIRED,
        ]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            AUTH_SESSION_CREATED => apply_session_created(event, conn),
            AUTH_SESSION_REVOKED => apply_session_terminal(
                &event.entity_id,
                "revoked",
                &event.payload_json,
                conn,
            ),
            AUTH_SESSION_EXPIRED => apply_session_terminal(
                &event.entity_id,
                "expired",
                &event.payload_json,
                conn,
            ),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM auth_sessions").execute(conn).await
        })?;
        Ok(())
    }
}

fn apply_session_created(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: AuthSessionCreated = serde_json::from_str(&event.payload_json)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO auth_sessions \
             (token, agent_id, scopes_json, project_id, created_at_unix, \
              expires_at_unix, revoked_at_unix, source, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, NULL, ?, datetime('now')) \
             ON CONFLICT(token) DO UPDATE SET \
               agent_id        = excluded.agent_id, \
               scopes_json     = excluded.scopes_json, \
               project_id      = excluded.project_id, \
               created_at_unix = excluded.created_at_unix, \
               expires_at_unix = excluded.expires_at_unix, \
               revoked_at_unix = NULL, \
               source          = excluded.source, \
               updated_at      = datetime('now')",
        )
        .bind(&event.entity_id)
        .bind(&payload.agent_id)
        .bind(&payload.scopes_json)
        .bind(&payload.project_id)
        .bind(payload.created_at_unix)
        .bind(payload.expires_at_unix)
        .bind(&payload.source)
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn apply_session_terminal(
    token: &str,
    kind: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    // Sanity-deserialize so a corrupted payload doesn't silently no-op.
    if kind == "revoked" {
        let _: AuthSessionRevoked = serde_json::from_str(payload_json)?;
    } else {
        let _: AuthSessionExpired = serde_json::from_str(payload_json).unwrap_or(AuthSessionExpired {});
    }
    let now = chrono::Utc::now().timestamp();
    block_on(async {
        sqlx::query(
            "UPDATE auth_sessions SET revoked_at_unix = ?, updated_at = datetime('now') \
             WHERE token = ?",
        )
        .bind(now)
        .bind(token)
        .execute(conn)
        .await
    })?;
    Ok(())
}

/// Look up an active (not revoked, not expired) session by token. Used on
/// every authenticated request — single-PK lookup against `auth_sessions`.
pub fn active_session(token: &str) -> Result<Option<AuthSessionRow>> {
    crate::db::ensure_db()?;
    let mut conn = crate::db::open_db()?;
    let now = chrono::Utc::now().timestamp();
    let row = block_on(async {
        sqlx::query(
            "SELECT token, agent_id, scopes_json, project_id, created_at_unix, \
                    expires_at_unix, revoked_at_unix, source \
             FROM auth_sessions \
             WHERE token = ? AND revoked_at_unix IS NULL AND expires_at_unix > ?",
        )
        .bind(token)
        .bind(now)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(session_row))
}

/// List all live (non-revoked, non-expired) sessions — for hydrate at startup.
pub fn list_active_sessions() -> Result<Vec<AuthSessionRow>> {
    crate::db::ensure_db()?;
    let mut conn = crate::db::open_db()?;
    let now = chrono::Utc::now().timestamp();
    let rows = block_on(async {
        sqlx::query(
            "SELECT token, agent_id, scopes_json, project_id, created_at_unix, \
                    expires_at_unix, revoked_at_unix, source \
             FROM auth_sessions \
             WHERE revoked_at_unix IS NULL AND expires_at_unix > ?",
        )
        .bind(now)
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.into_iter().map(session_row).collect())
}

fn session_row(row: sqlx::sqlite::SqliteRow) -> AuthSessionRow {
    AuthSessionRow {
        token: row.get(0),
        agent_id: row.get(1),
        scopes_json: row.get(2),
        project_id: row.try_get(3).ok(),
        created_at_unix: row.get(4),
        expires_at_unix: row.get(5),
        revoked_at_unix: row.try_get(6).ok(),
        source: row.get(7),
    }
}

/// One-shot drain of the legacy `auth.sessions` KV namespace into events.
/// For each persisted row, emit `auth.session.created` so the projection
/// materializes equivalent state. Idempotent: re-running on a drained
/// install is a no-op (KV namespace empty after drain).
pub fn drain_legacy_sessions_kv() -> Result<usize> {
    use crate::db::{block_on, ensure_kv_db, open_kv_db};
    use crate::events::EventEnvelope;
    use crate::persistence::event_store::{EventStore, SqliteEventStore};
    use ship_types::payload::AuthSessionCreated;

    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let rows = block_on(async {
        sqlx::query("SELECT key, value_json FROM kv_state WHERE namespace = 'auth.sessions'")
            .fetch_all(&mut conn)
            .await
    })?;
    if rows.is_empty() {
        return Ok(0);
    }

    #[derive(serde::Deserialize)]
    struct LegacyPersistedSession {
        token: String,
        agent_id: String,
        scopes: serde_json::Value,
        workspace_id: Option<String>,
        created_at_unix: i64,
        expires_at_unix: i64,
    }

    let store = SqliteEventStore::new()?;
    let mut migrated = 0usize;
    for row in &rows {
        let json_str: String = row.get(1);
        let Ok(legacy) = serde_json::from_str::<LegacyPersistedSession>(&json_str) else {
            continue;
        };
        let scopes_json = serde_json::to_string(&legacy.scopes)
            .unwrap_or_else(|_| "[]".to_string());
        let payload = AuthSessionCreated {
            agent_id: legacy.agent_id,
            scopes_json,
            project_id: legacy.workspace_id,
            created_at_unix: legacy.created_at_unix,
            expires_at_unix: legacy.expires_at_unix,
            source: "legacy_kv".to_string(),
        };
        let env = EventEnvelope::new(AUTH_SESSION_CREATED, &legacy.token, &payload)?;
        let _ = store.append(&env);
        migrated += 1;
    }
    block_on(async {
        sqlx::query("DELETE FROM kv_state WHERE namespace = 'auth.sessions'")
            .execute(&mut conn)
            .await
    })?;
    Ok(migrated)
}

// ── Owner ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DaemonOwnerRow {
    pub user_id: String,
    pub bound_at_unix: i64,
    pub source: String,
}

pub struct DaemonOwnerProjection;

impl Projection for DaemonOwnerProjection {
    fn name(&self) -> &str {
        "daemon_owner"
    }

    fn event_types(&self) -> &[&str] {
        &[DAEMON_OWNER_BOUND, DAEMON_OWNER_UNBOUND]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            DAEMON_OWNER_BOUND => apply_owner_bound(event, conn),
            DAEMON_OWNER_UNBOUND => apply_owner_unbound(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM daemon_owner").execute(conn).await
        })?;
        Ok(())
    }
}

fn apply_owner_bound(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: DaemonOwnerBound = serde_json::from_str(&event.payload_json)?;
    block_on(async {
        // Single-row table: clear any prior binding, then insert the new one.
        sqlx::query("DELETE FROM daemon_owner").execute(&mut *conn).await?;
        sqlx::query(
            "INSERT INTO daemon_owner (user_id, bound_at_unix, source, updated_at) \
             VALUES (?, ?, ?, datetime('now'))",
        )
        .bind(&event.entity_id)
        .bind(payload.bound_at_unix)
        .bind(&payload.source)
        .execute(&mut *conn)
        .await
    })?;
    Ok(())
}

fn apply_owner_unbound(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let _: DaemonOwnerUnbound = serde_json::from_str(&event.payload_json).unwrap_or(DaemonOwnerUnbound {});
    block_on(async {
        sqlx::query("DELETE FROM daemon_owner WHERE user_id = ?")
            .bind(&event.entity_id)
            .execute(conn)
            .await
    })?;
    Ok(())
}

/// One-shot drain of the legacy `auth.daemon_owner` KV namespace into a
/// `daemon.owner.bound` event. Idempotent.
pub fn drain_legacy_owner_kv() -> Result<usize> {
    use crate::db::{block_on, ensure_kv_db, open_kv_db};
    use crate::events::EventEnvelope;
    use crate::persistence::event_store::{EventStore, SqliteEventStore};
    use ship_types::payload::DaemonOwnerBound;

    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT key, value_json FROM kv_state WHERE namespace = 'auth.daemon_owner'",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    if rows.is_empty() {
        return Ok(0);
    }

    let store = SqliteEventStore::new()?;
    let mut migrated = 0usize;
    for row in &rows {
        let json_str: String = row.get(1);
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) else {
            continue;
        };
        let Some(user_id) = value.as_str() else {
            continue;
        };
        let payload = DaemonOwnerBound {
            source: "legacy_kv".to_string(),
            bound_at_unix: chrono::Utc::now().timestamp(),
        };
        let env = EventEnvelope::new(DAEMON_OWNER_BOUND, user_id, &payload)?;
        let _ = store.append(&env);
        migrated += 1;
    }
    block_on(async {
        sqlx::query("DELETE FROM kv_state WHERE namespace = 'auth.daemon_owner'")
            .execute(&mut conn)
            .await
    })?;
    Ok(migrated)
}

/// Read the current daemon owner, if any.
pub fn current_owner() -> Result<Option<DaemonOwnerRow>> {
    crate::db::ensure_db()?;
    let mut conn = crate::db::open_db()?;
    let row = block_on(async {
        sqlx::query("SELECT user_id, bound_at_unix, source FROM daemon_owner LIMIT 1")
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.map(|r| DaemonOwnerRow {
        user_id: r.get(0),
        bound_at_unix: r.get(1),
        source: r.get(2),
    }))
}
