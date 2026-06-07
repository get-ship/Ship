//! Projection trait and event bus.

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::events::EventEnvelope;

/// A projection maintains a derived read model from events.
///
/// Projections are registered with the [`EventBus`]. On each event append,
/// the bus calls [`apply`] on every projection whose [`event_types`] match.
/// The same [`apply`] path is used during replay — one code path for live
/// and rebuild.
pub trait Projection: Send + Sync {
    /// Human-readable name for logging and selective rebuild.
    fn name(&self) -> &str;

    /// Event types this projection handles.
    fn event_types(&self) -> &[&str];

    /// Apply a single event to the read model.
    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()>;

    /// Drop all derived state so replay can rebuild from scratch.
    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()>;
}

/// Per-projection cursor name in the `cursors` table. The format is fixed at
/// `projection:<name>`; consumers should not write to this prefix directly.
fn cursor_name(projection_name: &str) -> String {
    format!("projection:{projection_name}")
}

/// Dispatches events to registered projections.
pub struct EventBus {
    projections: Vec<Box<dyn Projection>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            projections: Vec::new(),
        }
    }

    pub fn register(&mut self, projection: Box<dyn Projection>) {
        self.projections.push(projection);
    }

    /// Dispatch an event to all matching projections.
    ///
    /// Called synchronously after every event append, on the SAME connection
    /// (and inside the SAME transaction) as the event-row insert. A handler
    /// error is propagated as `Err` to the caller — it does NOT get swallowed.
    /// `SqliteEventStore::append` rolls the transaction back on that `Err`, so a
    /// projection that rejects an event leaves no orphaned event row. The first
    /// failing handler short-circuits dispatch; later projections do not run for
    /// this event (the whole write is about to be rolled back anyway).
    ///
    /// On a successful `apply`, the projection's cursor in the `cursors` table
    /// is bumped to this event's id so an incremental rebuild can resume from
    /// here. The cursor write is bookkeeping, not part of the projection
    /// contract, so a cursor-write failure is logged rather than failing the
    /// write.
    pub fn dispatch(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        for proj in &self.projections {
            if !proj.event_types().contains(&event.event_type.as_str()) {
                continue;
            }
            proj.apply(event, conn).map_err(|e| {
                e.context(format!(
                    "projection '{}' failed to apply '{}'",
                    proj.name(),
                    event.event_type,
                ))
            })?;
            if let Err(e) = bump_cursor_with_conn(conn, proj.name(), &event.id) {
                tracing::warn!(
                    "[projection:{}] cursor write failed for {}: {}",
                    proj.name(),
                    event.id,
                    e,
                );
            }
        }
        Ok(())
    }

    /// Replay all events through registered projections.
    ///
    /// Truncates each projection first (and clears its cursor), then applies
    /// events in order. This is the correctness guarantee: projections are
    /// always rebuildable.
    ///
    /// A handler that errors on replay aborts the rebuild with `Err` — a
    /// drifted projection that cannot apply a valid event must fail loud, not
    /// silently report a partial rebuild as success.
    pub fn rebuild(
        &self,
        events: &[EventEnvelope],
        conn: &mut SqliteConnection,
    ) -> Result<RebuildReport> {
        let mut report = RebuildReport {
            events_replayed: 0,
            projections_rebuilt: Vec::new(),
        };

        for proj in &self.projections {
            proj.truncate(conn)?;
            // Clear the cursor so the next live event re-establishes it.
            crate::db::cursors::delete(&cursor_name(proj.name()))?;
            report.projections_rebuilt.push(proj.name().to_string());
        }

        for event in events {
            self.dispatch(event, conn)?;
            report.events_replayed += 1;
        }

        Ok(report)
    }

    /// List registered projection names.
    pub fn projection_names(&self) -> Vec<&str> {
        self.projections.iter().map(|p| p.name()).collect()
    }

    /// Read the cursor for a registered projection, if any.
    pub fn cursor_for(&self, projection_name: &str) -> Result<Option<String>> {
        crate::db::cursors::get(&cursor_name(projection_name))
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RebuildReport {
    pub events_replayed: usize,
    pub projections_rebuilt: Vec<String>,
}

/// Bump the cursor for a projection inside an existing connection. The
/// `cursors` table lives in platform.db so this is a same-DB write.
fn bump_cursor_with_conn(
    conn: &mut SqliteConnection,
    projection_name: &str,
    event_id: &str,
) -> Result<()> {
    let name = cursor_name(projection_name);
    crate::db::block_on(async {
        sqlx::query(
            "INSERT INTO cursors (name, last_event_id, updated_at)
             VALUES (?, ?, datetime('now'))
             ON CONFLICT(name) DO UPDATE SET
               last_event_id = excluded.last_event_id,
               updated_at    = excluded.updated_at",
        )
        .bind(&name)
        .bind(event_id)
        .execute(conn)
        .await
    })?;
    Ok(())
}
