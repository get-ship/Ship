//! Projection rebuild — replay each scope db's event log to reconstruct its
//! derived state.
//!
//! This is the recovery mechanism when projections drift from events. After the
//! Phase-4 storage partition every scope owns its own db file (`runtime.db`,
//! `account.db`, each mounted `spaces/<id>/space.db`), each carrying its own
//! `events` log AND its own projection tables. A rebuild therefore runs
//! per-scope: for every present scope db it loads that db's events, truncates
//! the projection tables in that same db, and replays the events through the
//! full projection bus. Projections only fire on event types they handle, so a
//! scope db materializes exactly the projections its events feed.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::db::envelope::{EVENT_COLS as COLS, envelope_from_row};
use crate::db::{block_on, ensure_events_db_at_path, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::projections::{ActorProjection, EventBus, SessionProjection};

/// Report returned by a projection rebuild. Aggregates across every scope db
/// that was rebuilt.
pub struct RebuildReport {
    pub events_replayed: usize,
    pub projections_rebuilt: Vec<String>,
    pub duration_ms: u64,
}

/// Construct the full projection bus: the L0 substrate defaults plus every
/// projection a higher layer registered via `install_extra_projections_factory`.
/// The same set is registered against every scope db; each db only materializes
/// the projections whose event types appear in its own log.
fn full_bus() -> EventBus {
    let mut bus = EventBus::new();
    bus.register(Box::new(SessionProjection::new()));
    bus.register(Box::new(ActorProjection::new()));
    for projection in crate::projections::extra_projections() {
        bus.register(projection);
    }
    bus
}

/// The scope db files that exist under `ship_dir` and should be rebuilt:
/// `runtime.db`, `account.db` (if present), and every mounted
/// `spaces/<id>/space.db`. `runtime.db` is always included — it is the
/// kernel's primary scope and is provisioned at boot — so a fresh install
/// still rebuilds (into an empty log).
fn scope_db_paths(ship_dir: &Path) -> Vec<PathBuf> {
    let mut paths = vec![crate::db::runtime_db_path(ship_dir)];

    let account = crate::db::account_db_path(ship_dir);
    if account.exists() {
        paths.push(account);
    }

    let spaces_dir = ship_dir.join(crate::db::SPACES_DIR);
    if let Ok(entries) = std::fs::read_dir(&spaces_dir) {
        for entry in entries.flatten() {
            if let Some(id) = entry.file_name().to_str() {
                let space_db = crate::db::space_db_path(ship_dir, id);
                if space_db.exists() {
                    paths.push(space_db);
                }
            }
        }
    }

    paths
}

/// Rebuild every projection in a single scope db from that db's own event log.
///
/// 1. Ensures the events schema exists at `db_path`.
/// 2. Loads all events from that db.
/// 3. Truncates the projection tables in that db.
/// 4. Replays events through the full projection bus.
///
/// Returns the per-db `(events_replayed, projections_rebuilt)`.
pub fn rebuild_scope_db(db_path: &Path) -> Result<(usize, Vec<String>)> {
    ensure_events_db_at_path(db_path)?;
    let mut conn = open_db_at(db_path)?;

    let rows = block_on(async {
        sqlx::query(&format!("SELECT {COLS} FROM events ORDER BY id ASC"))
            .fetch_all(&mut conn)
            .await
    })?;

    let events: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, true))
        .collect::<Result<_>>()?;

    let bus = full_bus();
    let report = bus.rebuild(&events, &mut conn)?;
    Ok((report.events_replayed, report.projections_rebuilt))
}

/// Rebuild all projections across every scope db under `ship_dir`.
///
/// Iterates `runtime.db`, `account.db`, and each mounted
/// `spaces/<id>/space.db`, rebuilding each from ITS own event log into ITS own
/// projection tables. The report aggregates the replayed-event count and the
/// distinct projection names touched.
pub fn rebuild_projections(ship_dir: &Path) -> Result<RebuildReport> {
    let start = Instant::now();

    let mut events_replayed = 0usize;
    let mut projections_rebuilt: Vec<String> = Vec::new();

    for db_path in scope_db_paths(ship_dir) {
        let (replayed, names) = rebuild_scope_db(&db_path)?;
        events_replayed += replayed;
        for name in names {
            if !projections_rebuilt.contains(&name) {
                projections_rebuilt.push(name);
            }
        }
    }

    Ok(RebuildReport {
        events_replayed,
        projections_rebuilt,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Rebuild projections using the default global dir.
pub fn rebuild_projections_default() -> Result<RebuildReport> {
    let ship_dir = crate::project::get_global_dir()?;
    rebuild_projections(&ship_dir)
}
