//! `project_compile_state` projection — the latest `compile.completed` event
//! for each space_id materializes into a single row. CLI `ship status`
//! and the daemon read here when answering "what was last compiled for
//! project X" without parsing JSON payloads on every read.

use anyhow::Result;
use sqlx::{SqliteConnection, Row};

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::COMPILE_COMPLETED;
use ship_types::payload::CompileCompleted;

/// Read model row materialized from `compile.completed` events.
#[derive(Debug, Clone)]
pub struct ProjectCompileState {
    pub space_id: String,
    pub active_agent: Option<String>,
    pub compiled_at: String,
    pub ship_version: String,
    pub harness: Option<String>,
    pub plugins_installed: Vec<String>,
}

pub struct CompileStateProjection;

impl Projection for CompileStateProjection {
    fn name(&self) -> &str {
        "project_compile_state"
    }

    fn event_types(&self) -> &[&str] {
        &[COMPILE_COMPLETED]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        if event.event_type != COMPILE_COMPLETED {
            return Ok(());
        }
        let payload: CompileCompleted = serde_json::from_str(&event.payload_json)?;
        let plugins_json = serde_json::to_string(&payload.plugins_installed)?;
        let space_id = event.entity_id.clone();
        block_on(async {
            sqlx::query(
                "INSERT INTO project_compile_state \
                 (space_id, active_agent, compiled_at, ship_version, harness, plugins_installed_json, updated_at) \
                 VALUES (?, ?, ?, ?, ?, ?, datetime('now')) \
                 ON CONFLICT(space_id) DO UPDATE SET \
                   active_agent = excluded.active_agent, \
                   compiled_at = excluded.compiled_at, \
                   ship_version = excluded.ship_version, \
                   harness = excluded.harness, \
                   plugins_installed_json = excluded.plugins_installed_json, \
                   updated_at = datetime('now')",
            )
            .bind(&space_id)
            .bind(&payload.active_agent)
            .bind(&payload.compiled_at)
            .bind(&payload.ship_version)
            .bind(&payload.harness)
            .bind(&plugins_json)
            .execute(conn)
            .await
        })?;
        Ok(())
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        ship_substrate::db::heal::truncate_or_heal(conn, "project_compile_state")
    }
}

/// Read the materialized compile state for a project. None if there's never
/// been a `compile.completed` event for that project.
pub fn load(space_id: &str) -> Result<Option<ProjectCompileState>> {
    // compile.* is Space-scoped → project_compile_state lives in the bounded
    // spaces/<space_id>/space.db; the `space_id` arg IS the scope key.
    let mut conn = ship_substrate::db::open_space_db(
        &ship_substrate::project::get_global_dir()?,
        space_id,
    )?;
    let row = block_on(async {
        sqlx::query(
            "SELECT space_id, active_agent, compiled_at, ship_version, harness, plugins_installed_json \
             FROM project_compile_state WHERE space_id = ?",
        )
        .bind(space_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(|r| {
        let plugins_json: String = r.get(5);
        let plugins_installed: Vec<String> =
            serde_json::from_str(&plugins_json).unwrap_or_default();
        ProjectCompileState {
            space_id: r.get(0),
            active_agent: r.try_get(1).ok(),
            compiled_at: r.get(2),
            ship_version: r.get(3),
            harness: r.try_get(4).ok(),
            plugins_installed,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::projections::EventBus;
    use ship_types::envelope::EventEnvelope;

    /// Initialize the auto-isolated project. compile.* is Space-scoped, so the
    /// projection row lands in the bound space's db — tests dispatch against
    /// `space_conn(id)`, the same db `load(id)` reads.
    fn setup() {
        let global = ship_substrate::project::get_global_dir().unwrap();
        let base = global.parent().unwrap().to_path_buf();
        crate::testutil::init_project(base).unwrap();
    }

    /// Open the per-space db for `space_id` under the auto-isolated global dir.
    fn space_conn(space_id: &str) -> sqlx::SqliteConnection {
        ship_substrate::db::open_space_db(
            &ship_substrate::project::get_global_dir().unwrap(),
            space_id,
        )
        .unwrap()
    }

    fn bus() -> EventBus {
        let mut b = EventBus::new();
        b.register(Box::new(CompileStateProjection));
        b
    }

    fn compile_event(space_id: &str, agent: &str, time: &str) -> EventEnvelope {
        let payload = CompileCompleted {
            active_agent: Some(agent.to_string()),
            compiled_at: time.to_string(),
            ship_version: "0.1.0".to_string(),
            harness: Some("claude".to_string()),
            plugins_installed: vec!["plug-a".to_string()],
        };
        EventEnvelope::new(COMPILE_COMPLETED, space_id, &payload).unwrap()
    }

    #[test]
    fn missing_project_returns_none() {
        setup();
        // Use a space_id that won't collide with prior test runs in the
        // shared global DB.
        let id = format!("proj-missing-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        assert!(load(&id).unwrap().is_none());
    }

    #[test]
    fn compile_completed_event_materializes_row() {
        setup();
        let id = format!("proj-mat-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let mut conn = space_conn(&id);
        let event = compile_event(&id, "planner", "2026-05-04T00:00:00Z");
        bus().dispatch(&event, &mut conn).unwrap();
        let row = load(&id).unwrap().expect("row should exist");
        assert_eq!(row.space_id, id);
        assert_eq!(row.active_agent.as_deref(), Some("planner"));
        assert_eq!(row.ship_version, "0.1.0");
        assert_eq!(row.harness.as_deref(), Some("claude"));
        assert_eq!(row.plugins_installed, vec!["plug-a".to_string()]);
    }

    #[test]
    fn second_event_overwrites_row() {
        setup();
        let id = format!("proj-over-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let mut conn = space_conn(&id);
        bus().dispatch(&compile_event(&id, "a", "2026-05-04T00:00:00Z"), &mut conn).unwrap();
        bus().dispatch(&compile_event(&id, "b", "2026-05-04T01:00:00Z"), &mut conn).unwrap();
        let row = load(&id).unwrap().unwrap();
        assert_eq!(row.active_agent.as_deref(), Some("b"));
        assert_eq!(row.compiled_at, "2026-05-04T01:00:00Z");
    }
}
