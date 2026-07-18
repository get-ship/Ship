//! Provisioned-grants projection — the `provisioned_grants` table.
//!
//! Materialized from `agent.grants.provisioned` events. One row per agent
//! (latest write wins). The token-mint path reads this baseline via
//! [`provisioned_grants_for`] instead of deriving grants from the role string;
//! the gate then enforces against the minted session's grants.
//!
//! Moved from `ship_substrate::projections::grants` (keystone-A Phase A bundle 2,
//! 2026-06-10): the AUTH domain owns its projection at L1.

use anyhow::Result;
use sqlx::{Row, SqliteConnection};

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::AGENT_GRANTS_PROVISIONED;
use ship_types::payload::AgentGrantsProvisioned;

/// Read-model row for `provisioned_grants`.
#[derive(Debug, Clone)]
pub struct ProvisionedGrantsRow {
    pub agent_id: String,
    pub grants: Vec<String>,
    pub source: String,
    pub approved_by: Option<String>,
}

pub struct ProvisionedGrantsProjection;

impl Projection for ProvisionedGrantsProjection {
    fn name(&self) -> &str {
        "provisioned_grants"
    }

    fn event_types(&self) -> &[&str] {
        &[AGENT_GRANTS_PROVISIONED]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            AGENT_GRANTS_PROVISIONED => apply_provisioned(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        ship_substrate::db::heal::truncate_or_heal(conn, "provisioned_grants")
    }
}

fn apply_provisioned(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: AgentGrantsProvisioned = serde_json::from_str(&event.payload_json)?;
    let grants_json = serde_json::to_string(&payload.grants)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO provisioned_grants \
             (agent_id, grants_json, source, approved_by, updated_at) \
             VALUES (?, ?, ?, ?, datetime('now')) \
             ON CONFLICT(agent_id) DO UPDATE SET \
               grants_json = excluded.grants_json, \
               source      = excluded.source, \
               approved_by = excluded.approved_by, \
               updated_at  = datetime('now')",
        )
        .bind(&event.entity_id)
        .bind(&grants_json)
        .bind(&payload.source)
        .bind(&payload.approved_by)
        .execute(conn)
        .await
    })?;
    Ok(())
}

/// Read an agent's provisioned grant baseline, if one has been recorded.
/// `None` means the agent has no baseline yet — the caller falls back to the
/// role-derived grants. Used on the token-mint path.
pub fn provisioned_grants_for(agent_id: &str) -> Result<Option<Vec<String>>> {
    // agent.grants.provisioned is Account-scoped → provisioned_grants in account.db.
    let mut conn = ship_substrate::db::open_account_db(
        &ship_substrate::project::get_global_dir()?,
    )?;
    let row = block_on(async {
        sqlx::query("SELECT grants_json FROM provisioned_grants WHERE agent_id = ?")
            .bind(agent_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    match row {
        Some(r) => {
            let grants_json: String = r.get(0);
            let grants: Vec<String> = serde_json::from_str(&grants_json)?;
            Ok(Some(grants))
        }
        None => Ok(None),
    }
}

/// Test/`_at` variant of [`provisioned_grants_for`] against a specific home dir.
pub fn provisioned_grants_for_at(
    home: &std::path::Path,
    agent_id: &str,
) -> Result<Option<Vec<String>>> {
    let mut conn = ship_substrate::db::open_account_db(home)?;
    let row = block_on(async {
        sqlx::query("SELECT grants_json FROM provisioned_grants WHERE agent_id = ?")
            .bind(agent_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    match row {
        Some(r) => {
            let grants_json: String = r.get(0);
            Ok(Some(serde_json::from_str(&grants_json)?))
        }
        None => Ok(None),
    }
}

/// Read the full provisioned row (for audit / display surfaces).
pub fn provisioned_row(agent_id: &str) -> Result<Option<ProvisionedGrantsRow>> {
    let mut conn = ship_substrate::db::open_account_db(
        &ship_substrate::project::get_global_dir()?,
    )?;
    let row = block_on(async {
        sqlx::query(
            "SELECT agent_id, grants_json, source, approved_by \
             FROM provisioned_grants WHERE agent_id = ?",
        )
        .bind(agent_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    match row {
        Some(r) => {
            let grants_json: String = r.get(1);
            Ok(Some(ProvisionedGrantsRow {
                agent_id: r.get(0),
                grants: serde_json::from_str(&grants_json)?,
                source: r.get(2),
                approved_by: r.try_get(3).ok(),
            }))
        }
        None => Ok(None),
    }
}
