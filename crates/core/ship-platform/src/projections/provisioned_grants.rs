//! Provisioned-grants projection — the `provisioned_grants` table.
//!
//! Materialized from `agent.grants.provisioned` events. One row per agent
//! (latest write wins). The token-mint path reads this baseline via
//! [`provisioned_grants_for`] instead of deriving grants from the role string;
//! the gate then enforces against the minted session's grants.
//!
//! `requires:`/`grants:` in skills are *intent*; this projection holds the
//! operator-approved authority. See
//! `.artifacts/capability-provisioning-and-approval.md`.

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
        block_on(async {
            sqlx::query("DELETE FROM provisioned_grants").execute(conn).await
        })?;
        Ok(())
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
    ship_substrate::db::ensure_db()?;
    let mut conn = ship_substrate::db::open_db()?;
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
    ship_substrate::db::ensure_db_at(home)?;
    let mut conn =
        ship_substrate::db::open_db_at(&ship_substrate::db::platform_db_path(home))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> (tempfile::TempDir, sqlx::SqliteConnection) {
        let tmp = tempfile::tempdir().unwrap();
        ship_substrate::db::ensure_db_at(tmp.path()).unwrap();
        let conn = ship_substrate::db::open_db_at(
            &ship_substrate::db::platform_db_path(tmp.path()),
        )
        .unwrap();
        (tmp, conn)
    }

    fn provisioned(agent_id: &str, grants: &[&str], source: &str) -> EventEnvelope {
        EventEnvelope::new(
            AGENT_GRANTS_PROVISIONED,
            agent_id,
            &serde_json::json!({
                "grants": grants,
                "source": source,
            }),
        )
        .unwrap()
    }

    #[test]
    fn absent_agent_reads_none() {
        let (tmp, _conn) = fresh();
        assert_eq!(provisioned_grants_for_at(tmp.path(), "ghost").unwrap(), None);
    }

    #[test]
    fn provisioned_baseline_round_trips() {
        let (tmp, mut conn) = fresh();
        ProvisionedGrantsProjection
            .apply(&provisioned("commander", &["ship-orchestration"], "install"), &mut conn)
            .unwrap();
        assert_eq!(
            provisioned_grants_for_at(tmp.path(), "commander").unwrap(),
            Some(vec!["ship-orchestration".to_string()])
        );
    }

    #[test]
    fn latest_write_wins() {
        let (tmp, mut conn) = fresh();
        let proj = ProvisionedGrantsProjection;
        proj.apply(&provisioned("a", &["ship-orchestration"], "install"), &mut conn)
            .unwrap();
        // Operator narrows the set — the later write replaces, not merges.
        proj.apply(&provisioned("a", &["ship-authoring"], "operator"), &mut conn)
            .unwrap();
        assert_eq!(
            provisioned_grants_for_at(tmp.path(), "a").unwrap(),
            Some(vec!["ship-authoring".to_string()])
        );
        let row = {
            let r = block_on(async {
                sqlx::query("SELECT source FROM provisioned_grants WHERE agent_id = ?")
                    .bind("a")
                    .fetch_one(&mut conn)
                    .await
            })
            .unwrap();
            r.get::<String, _>(0)
        };
        assert_eq!(row, "operator");
    }
}

/// Read the full provisioned row (for audit / display surfaces).
pub fn provisioned_row(agent_id: &str) -> Result<Option<ProvisionedGrantsRow>> {
    ship_substrate::db::ensure_db()?;
    let mut conn = ship_substrate::db::open_db()?;
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
