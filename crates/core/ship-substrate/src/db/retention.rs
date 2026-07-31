//! Retention sweep — delete event rows of `Retention::Windowed` types older
//! than their per-type TTL.
//!
//! The sweep is the ONLY place the Windowed tier differs from Durable:
//! append/route persists both identically (row insert + in-TX projection
//! fold); the sweep later trims Windowed rows past their horizon, and replay
//! (`db::rebuild`) folds whatever survives. Projections over Windowed types
//! are therefore bounded by the retention horizon by construction.
//!
//! Constraints the sweep upholds:
//! - Deletes by `(event_type, timestamp horizon)` ONLY. Durable/Ephemeral
//!   types are never named in a DELETE; projection tables are never touched
//!   (rebuild reconciles them from the surviving log).
//! - Crash-safe: each per-type DELETE is a single autocommit statement, so a
//!   killed sweep leaves a consistent, merely-unswept log — the next run
//!   (boot or periodic) finishes the job. No cross-statement state exists.
//! - Maintenance, not an event source: nothing is emitted through the
//!   kernel; the only trace is the returned report + tracing lines.
//!
//! Scheduling: run once at daemon boot (alongside the integrity check) and
//! then every [`SWEEP_PERIOD`]. The period must stay well under the smallest
//! registered TTL so the surviving window overshoots its horizon by at most
//! `SWEEP_PERIOD` (6h against the 7-day tier ≈ 3.6% slop).

use anyhow::{Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::path::Path;

use crate::db::{block_on, open_db_at};

/// Cadence of the periodic sweep after the boot-time run.
pub const SWEEP_PERIOD: std::time::Duration = std::time::Duration::from_secs(6 * 60 * 60);

/// One swept `(event_type, ttl_days)` pair with the rows it deleted across
/// every scope db.
#[derive(Debug)]
pub struct SweptType {
    pub event_type: &'static str,
    pub ttl_days: u16,
    pub rows_deleted: u64,
}

/// Aggregate result of one sweep pass.
#[derive(Debug, Default)]
pub struct SweepReport {
    pub swept: Vec<SweptType>,
    pub dbs_visited: usize,
}

impl SweepReport {
    pub fn rows_deleted(&self) -> u64 {
        self.swept.iter().map(|s| s.rows_deleted).sum()
    }
}

/// Every registered Windowed type with its TTL. Sourced from the linked
/// event registry, so the sweep's type list can never drift from the
/// registrations — a type is sweepable exactly when its `register_event!`
/// says `Retention::Windowed`.
pub fn windowed_types() -> Vec<(&'static str, u16)> {
    ship_types::event_registry::registry()
        .filter_map(|d| d.retention.ttl_days().map(|ttl| (d.name, ttl)))
        .collect()
}

/// The UTC horizon for a TTL: rows strictly older than this are swept.
/// Rendered in sqlite's canonical `YYYY-MM-DD HH:MM:SS` form so it compares
/// against `datetime(created_at)` regardless of whether the row stored
/// RFC3339 (the envelope insert path) or `datetime('now')` (the column
/// default).
fn horizon(now: DateTime<Utc>, ttl_days: u16) -> String {
    (now - ChronoDuration::days(i64::from(ttl_days)))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Sweep one scope db at an explicit `now`. Returns per-type deleted counts
/// (only types that deleted at least one row are reported).
pub fn sweep_scope_db_at(db_path: &Path, now: DateTime<Utc>) -> Result<Vec<SweptType>> {
    sweep_types_at(db_path, &windowed_types(), now)
}

/// The DELETE mechanism over an explicit `(event_type, ttl_days)` list.
/// Production always passes [`windowed_types`] (the registry is the sole
/// source of sweepable types); tests pass explicit lists so a binary that
/// links no Windowed registration can still exercise the mechanics.
///
/// Each DELETE names exactly one event type and one horizon — a Durable row
/// can only be deleted here if sqlite fails to match `event_type = ?`, i.e.
/// never. `datetime(created_at)` normalizes both stored timestamp forms; a
/// row whose `created_at` does not parse as a datetime compares NULL and is
/// left alone (fail-closed toward keeping data).
pub(crate) fn sweep_types_at(
    db_path: &Path,
    types: &[(&'static str, u16)],
    now: DateTime<Utc>,
) -> Result<Vec<SweptType>> {
    let mut conn = open_db_at(db_path)
        .with_context(|| format!("retention sweep: open {}", db_path.display()))?;

    let mut swept = Vec::new();
    for &(event_type, ttl_days) in types {
        let cutoff = horizon(now, ttl_days);
        let result = block_on(async {
            sqlx::query(
                "DELETE FROM events \
                 WHERE event_type = ? AND datetime(created_at) < datetime(?)",
            )
            .bind(event_type)
            .bind(&cutoff)
            .execute(&mut conn)
            .await
        })
        .with_context(|| {
            format!(
                "retention sweep: DELETE {event_type} rows older than {cutoff} in {}",
                db_path.display()
            )
        })?;
        let rows_deleted = result.rows_affected();
        if rows_deleted > 0 {
            swept.push(SweptType {
                event_type,
                ttl_days,
                rows_deleted,
            });
        }
    }
    Ok(swept)
}

/// Sweep every present scope db under `ship_dir` (`runtime.db`, `account.db`,
/// each mounted `spaces/<id>/space.db`) at the current wall clock.
pub fn sweep_windowed_events(ship_dir: &Path) -> Result<SweepReport> {
    sweep_windowed_events_at(ship_dir, Utc::now())
}

/// Sweep every present scope db under `ship_dir` at an explicit `now`
/// (seam for tests; production passes the wall clock).
pub fn sweep_windowed_events_at(ship_dir: &Path, now: DateTime<Utc>) -> Result<SweepReport> {
    let mut report = SweepReport::default();
    for db_path in crate::db::integrity::scope_db_paths(ship_dir) {
        report.dbs_visited += 1;
        for swept in sweep_scope_db_at(&db_path, now)? {
            tracing::info!(
                event_type = swept.event_type,
                ttl_days = swept.ttl_days,
                rows_deleted = swept.rows_deleted,
                db = %db_path.display(),
                "retention sweep trimmed windowed rows"
            );
            match report
                .swept
                .iter_mut()
                .find(|s| s.event_type == swept.event_type)
            {
                Some(existing) => existing.rows_deleted += swept.rows_deleted,
                None => report.swept.push(swept),
            }
        }
    }
    Ok(report)
}
