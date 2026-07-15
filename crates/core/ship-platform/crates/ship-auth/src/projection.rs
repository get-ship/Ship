//! Auth projections — `auth_sessions` and `daemon_owner` tables.
//!
//! Materialized from auth.session.* and daemon.owner.* events. Token lookup
//! on every authed request reads `auth_sessions WHERE token = ? AND
//! revoked_at_unix IS NULL`. The owner row is at most one (single-row
//! projection).

use anyhow::Result;
use sqlx::{Row, SqliteConnection};

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
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
    pub space_id: Option<String>,
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
             (token, agent_id, space_id, created_at_unix, \
              expires_at_unix, revoked_at_unix, source, updated_at) \
             VALUES (?, ?, ?, ?, ?, NULL, ?, datetime('now')) \
             ON CONFLICT(token) DO UPDATE SET \
               agent_id        = excluded.agent_id, \
               space_id      = excluded.space_id, \
               created_at_unix = excluded.created_at_unix, \
               expires_at_unix = excluded.expires_at_unix, \
               revoked_at_unix = NULL, \
               source          = excluded.source, \
               updated_at      = datetime('now')",
        )
        .bind(&event.entity_id)
        .bind(&payload.agent_id)
        .bind(&payload.space_id)
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
    let mut conn = open_account_conn()?;
    let now = chrono::Utc::now().timestamp();
    let row = block_on(async {
        sqlx::query(
            "SELECT token, agent_id, space_id, created_at_unix, \
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
    let mut conn = open_account_conn()?;
    let now = chrono::Utc::now().timestamp();
    let rows = block_on(async {
        sqlx::query(
            "SELECT token, agent_id, space_id, created_at_unix, \
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
        space_id: row.try_get(2).ok(),
        created_at_unix: row.get(3),
        expires_at_unix: row.get(4),
        revoked_at_unix: row.try_get(5).ok(),
        source: row.get(6),
    }
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

/// Read the current daemon owner, if any.
pub fn current_owner() -> Result<Option<DaemonOwnerRow>> {
    let mut conn = open_account_conn()?;
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

/// Open the per-account `account.db` for the auth/owner projections (both
/// `auth.session.*` and `daemon.owner.*` are Account-scoped).
fn open_account_conn() -> Result<SqliteConnection> {
    let ship_dir = ship_substrate::project::get_global_dir()?;
    ship_substrate::db::open_account_db(&ship_dir)
}
