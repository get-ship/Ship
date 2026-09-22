//! Pure projection-read helpers for runtime observability endpoints.
//!
//! All functions accept an explicit `ship_dir` path and return plain data — no
//! kernel or daemon-state access. Designed to be called from ConnectRPC
//! service handlers in `shipd` and from `EventsService.Poll` / `RuntimeService`
//! wires without any daemon-specific coupling.

use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::projections::process::ProcessRecord;

// ─── Event reads ──────────────────────────────────────────────────────────────

/// Cursor-walk event poll. Reads the most recent events from the Runtime
/// partition (`runtime.db`), filters by `since_id` cursor, namespace prefix,
/// and slug, and returns `{ "events": [...] }`.
///
/// Called by the `EventsService.Poll` ConnectRPC handler.
pub fn poll_events_core(
    ship_dir: &Path,
    since_id: Option<&str>,
    slug: Option<&str>,
    namespace: Option<&str>,
    limit: usize,
) -> Result<Value, String> {
    let limit = limit.min(2000);
    let events =
        crate::db::events::list_recent_events_at(ship_dir, 5_000).map_err(|e| e.to_string())?;

    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| match since_id {
            Some(c) => e.id.as_str() > c,
            None => true,
        })
        .filter(|e| match namespace {
            Some(prefix) => e.event_type.starts_with(&format!("{prefix}.")),
            None => true,
        })
        .filter(|e| match slug {
            Some(s) => {
                if e.entity_id == s {
                    return true;
                }
                serde_json::from_str::<Value>(&e.payload_json)
                    .ok()
                    .and_then(|v| v["slug"].as_str().map(str::to_string))
                    .map(|p| p == s)
                    .unwrap_or(false)
            }
            None => true,
        })
        .take(limit)
        .collect();

    Ok(serde_json::json!({ "events": filtered }))
}

// ─── Process reads ────────────────────────────────────────────────────────────

/// Project process events from an explicit `ship_dir` without going through
/// the global db path.
pub fn list_processes_at(ship_dir: &Path, slug: Option<&str>) -> Result<Vec<ProcessRecord>> {
    let events = crate::db::events::list_recent_events_at(ship_dir, 50_000)?;
    let mut records = crate::projections::process::project(&events);
    if let Some(s) = slug {
        records.retain(|r| r.slug == s);
    }
    Ok(records)
}
