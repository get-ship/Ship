//! Pure projection-read helpers for runtime observability endpoints.
//!
//! All functions accept an explicit `ship_dir` path and return plain data — no
//! kernel or daemon-state access. Designed to be called from thin HTTP handlers
//! in `shipd` and from `EventsService.Poll` / `RuntimeService` wires without
//! any daemon-specific coupling.

use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::events::EventEnvelope;
use crate::projections::process::ProcessRecord;

// ─── Event reads ──────────────────────────────────────────────────────────────

/// Cursor-walk event poll. Reads the most recent events from the Runtime
/// partition (`runtime.db`), filters by `since_id` cursor, namespace prefix,
/// and slug, and returns `{ "events": [...] }`.
///
/// Shared by the REST handler and the `EventsService.Poll` ConnectRPC wire.
pub fn poll_events_core(
    ship_dir: &Path,
    since_id: Option<&str>,
    slug: Option<&str>,
    namespace: Option<&str>,
    limit: usize,
) -> Result<Value, String> {
    let limit = limit.min(2000);
    let events = crate::db::events::list_recent_events_at(ship_dir, 5_000)
        .map_err(|e| e.to_string())?;

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
/// the global db path. Mirrors `load_pipeline_runs_at` for jobs.
pub fn list_processes_at(
    ship_dir: &Path,
    slug: Option<&str>,
) -> Result<Vec<ProcessRecord>> {
    let events = crate::db::events::list_recent_events_at(ship_dir, 50_000)?;
    let mut records = crate::projections::process::project(&events);
    if let Some(s) = slug {
        records.retain(|r| r.slug == s);
    }
    Ok(records)
}

/// Filter recent events by event_type set, against an explicit `ship_dir`.
pub fn load_recent_events_by_type(
    ship_dir: &Path,
    types: &[&str],
    limit: usize,
) -> Result<Vec<EventEnvelope>> {
    let events = crate::db::events::list_recent_events_at(ship_dir, limit)?;
    Ok(events
        .into_iter()
        .filter(|e| types.contains(&e.event_type.as_str()))
        .collect())
}

// ─── Drift helpers ────────────────────────────────────────────────────────────

/// `true` if the most recent `supervisor.drift_detected` for `slug` is newer
/// than any subsequent `supervisor.coherence_restored` — i.e. the drift has
/// not yet been resolved.
pub fn drift_active_for_slug(slug: &str, events: &[EventEnvelope]) -> bool {
    let mut latest_drift: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut latest_restored: Option<chrono::DateTime<chrono::Utc>> = None;
    for e in events {
        if e.entity_id != slug {
            continue;
        }
        match e.event_type.as_str() {
            "supervisor.drift_detected" => {
                latest_drift =
                    Some(latest_drift.map_or(e.created_at, |d| d.max(e.created_at)));
            }
            "supervisor.coherence_restored" => {
                latest_restored =
                    Some(latest_restored.map_or(e.created_at, |d| d.max(e.created_at)));
            }
            _ => {}
        }
    }
    match (latest_drift, latest_restored) {
        (Some(d), Some(r)) => d > r,
        (Some(_), None) => true,
        _ => false,
    }
}

// ─── String helpers ───────────────────────────────────────────────────────────

/// Truncate `s` to at most `max` chars, appending `…` when cut.
pub fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        chars[..max.saturating_sub(1)].iter().collect::<String>() + "…"
    }
}

// ─── Cross-project session drain ──────────────────────────────────────────────

/// Active sessions across all spaces, each row labeled with `space_id` and
/// the space's display name. Used by the `ListSessionsAcrossProjects` endpoint.
pub fn list_active_sessions_with_projects() -> Result<Vec<serde_json::Value>, String> {
    let sessions = crate::db::session_drain::list_active_sessions()
        .map_err(|e| e.to_string())?;

    let by_id: std::collections::HashMap<String, crate::db::projects::SpaceRecord> =
        crate::db::projects::list_active()
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

    Ok(sessions
        .into_iter()
        .map(|s| {
            let project_name = s
                .space_id
                .as_deref()
                .and_then(|id| by_id.get(id))
                .map(|p| p.name.clone());
            serde_json::json!({
                "id": s.id,
                "workspace_id": s.workspace_id,
                "branch": s.branch,
                "space_id": s.space_id,
                "project_name": project_name,
                "status": s.status,
                "agent_id": s.agent_id,
                "started_at": s.started_at,
                "last_activity_at": s.last_activity_at,
                "goal": s.goal,
            })
        })
        .collect())
}
