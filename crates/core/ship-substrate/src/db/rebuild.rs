//! Projection rebuild — replay all events to reconstruct derived state.
//!
//! This is the recovery mechanism when projections drift from events.
//! Loads all events from platform.db, truncates projection tables, and
//! replays events through the EventBus.

use anyhow::Result;
use std::path::Path;
use std::time::Instant;

use crate::db::envelope::envelope_from_row;
use crate::db::{block_on, ensure_db_at, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::projections::{ActorProjection, EventBus, SessionProjection};

/// Report returned by a projection rebuild.
pub struct RebuildReport {
    pub events_replayed: usize,
    pub projections_rebuilt: Vec<String>,
    pub duration_ms: u64,
}

/// Rebuild all projections from the event log.
///
/// 1. Loads all events from platform.db
/// 2. Truncates all projection tables
/// 3. Replays events through the EventBus
/// 4. Reports results
pub fn rebuild_projections(ship_dir: &Path) -> Result<RebuildReport> {
    let start = Instant::now();
    ensure_db_at(ship_dir)?;
    let db_path = crate::db::platform_db_path(ship_dir);
    let mut conn = open_db_at(&db_path)?;

    const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
        causation_id, space_id, session_id, \
        actor_id, parent_actor_id, elevated, created_at, target_actor_id";

    let rows = block_on(async {
        sqlx::query(&format!("SELECT {COLS} FROM events ORDER BY id ASC"))
            .fetch_all(&mut conn)
            .await
    })?;

    let events: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, true))
        .collect::<Result<_>>()?;

    let mut bus = EventBus::new();
    bus.register(Box::new(SessionProjection::new()));
    bus.register(Box::new(ActorProjection::new()));
    for projection in crate::projections::extra_projections() {
        bus.register(projection);
    }

    let report = bus.rebuild(&events, &mut conn)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(RebuildReport {
        events_replayed: report.events_replayed,
        projections_rebuilt: report.projections_rebuilt,
        duration_ms,
    })
}

/// Rebuild projections using the default global dir.
pub fn rebuild_projections_default() -> Result<RebuildReport> {
    let ship_dir = crate::project::get_global_dir()?;
    crate::db::ensure_db()?;
    rebuild_projections(&ship_dir)
}
