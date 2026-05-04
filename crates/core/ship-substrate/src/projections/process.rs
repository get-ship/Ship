//! Process lifecycle projection. Derived from process.* events.
//!
//! Answers: for slug X, which processes spawned, when, with what
//! signals, and how did they exit?
//!
//! A `ProcessRecord` is keyed by `(slug, pid)`. Multiple processes per
//! slug are possible across pipeline steps, restarts, or crashes.
//! Records with `pid: None` use `(slug, spawned_at)` as the synthetic key.

use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::events::process::event_types;
use crate::events::EventEnvelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    Running,
    Exited,
    Killed,
    Orphaned,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SignalEntry {
    pub signum: String,
    pub reason: String,
    pub actor: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProcessRecord {
    pub pid: Option<u32>,
    pub slug: String,
    pub job_id: Option<String>,
    pub step_id: Option<String>,
    pub cmd: String,
    pub executor_kind: String,
    pub spawned_at: DateTime<Utc>,
    pub exited_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub exit_signal: Option<String>,
    pub signals_sent: Vec<SignalEntry>,
    pub status: ProcessStatus,
}

/// Synthetic key for process records. Processes with a PID are keyed
/// by `(slug, pid)`. Those without (tmux-detached) use `(slug, spawned_at)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RecordKey {
    Pid(String, u32),
    Synthetic(String, DateTime<Utc>),
}

fn key_for(slug: &str, pid: Option<u32>, spawned_at: DateTime<Utc>) -> RecordKey {
    match pid {
        Some(p) => RecordKey::Pid(slug.to_string(), p),
        None => RecordKey::Synthetic(slug.to_string(), spawned_at),
    }
}

/// Apply a single event to an existing `ProcessRecord`.
pub fn apply(record: &mut ProcessRecord, event: &EventEnvelope) {
    let ts = event.created_at;
    match event.event_type.as_str() {
        event_types::PROCESS_EXITED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.exit_code = p["exit_code"].as_i64().map(|c| c as i32);
                record.exit_signal = p["signal"].as_str().map(str::to_string);
                record.exited_at = Some(ts);
                record.status = ProcessStatus::Exited;
            }
        }
        event_types::PROCESS_SIGNAL_SENT => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.signals_sent.push(SignalEntry {
                    signum: p["signum"].as_str().unwrap_or("").to_string(),
                    reason: p["reason"].as_str().unwrap_or("").to_string(),
                    actor: p["actor"].as_str().unwrap_or("").to_string(),
                    at: ts,
                });
            }
        }
        event_types::PROCESS_KILLED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.exit_signal = p["signal"].as_str().map(str::to_string);
                record.exited_at = Some(ts);
                record.status = ProcessStatus::Killed;
                record.signals_sent.push(SignalEntry {
                    signum: p["signal"].as_str().unwrap_or("").to_string(),
                    reason: p["reason"].as_str().unwrap_or("").to_string(),
                    actor: p["actor"].as_str().unwrap_or("").to_string(),
                    at: ts,
                });
            }
        }
        event_types::PROCESS_ORPHANED => {
            record.status = ProcessStatus::Orphaned;
        }
        _ => {}
    }
}

/// Fold a slice of events into a list of `ProcessRecord`.
pub fn project(events: &[EventEnvelope]) -> Vec<ProcessRecord> {
    let mut map: HashMap<RecordKey, ProcessRecord> = HashMap::new();

    for event in events {
        if !event.event_type.starts_with("process.") {
            continue;
        }

        if event.event_type == event_types::PROCESS_SPAWNED {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                let slug = p["slug"].as_str().unwrap_or("").to_string();
                let pid = p["pid"].as_u64().map(|v| v as u32);
                let key = key_for(&slug, pid, event.created_at);
                let record = ProcessRecord {
                    pid,
                    slug,
                    job_id: p["job_id"].as_str().map(str::to_string),
                    step_id: p["step_id"].as_str().map(str::to_string),
                    cmd: p["cmd"].as_str().unwrap_or("").to_string(),
                    executor_kind: p["executor_kind"].as_str().unwrap_or("").to_string(),
                    spawned_at: event.created_at,
                    exited_at: None,
                    exit_code: None,
                    exit_signal: None,
                    signals_sent: vec![],
                    status: ProcessStatus::Running,
                };
                map.insert(key, record);
            }
            continue;
        }

        // All other process events match by slug + pid.
        if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
            let slug = p["slug"].as_str().unwrap_or("").to_string();
            let pid = p["pid"].as_u64().map(|v| v as u32);
            // Find the matching record — prefer (slug, pid) key.
            let matching_key = if let Some(pid_val) = pid {
                let pid_key = RecordKey::Pid(slug.clone(), pid_val);
                if map.contains_key(&pid_key) {
                    Some(pid_key)
                } else {
                    // Fall back to most recent synthetic key for this slug.
                    map.keys()
                        .filter(|k| matches!(k, RecordKey::Synthetic(s, _) if s == &slug))
                        .max_by_key(|k| match k {
                            RecordKey::Synthetic(_, t) => *t,
                            _ => DateTime::<Utc>::MIN_UTC,
                        })
                        .cloned()
                }
            } else {
                map.keys()
                    .filter(|k| match k {
                        RecordKey::Pid(s, _) | RecordKey::Synthetic(s, _) => s == &slug,
                    })
                    .max_by_key(|k| match k {
                        RecordKey::Synthetic(_, t) => *t,
                        RecordKey::Pid(_, _) => DateTime::<Utc>::MIN_UTC,
                    })
                    .cloned()
            };

            if let Some(key) = matching_key {
                if let Some(record) = map.get_mut(&key) {
                    apply(record, event);
                }
            }
        }
    }

    let mut records: Vec<_> = map.into_values().collect();
    records.sort_by_key(|r| r.spawned_at);
    records
}

/// Load all process events from the platform DB and return projected records.
pub fn load_processes(slug: Option<&str>) -> Result<Vec<ProcessRecord>> {
    use crate::db::{db_path, ensure_db, open_db_at};

    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    load_processes_from_conn(&mut conn, slug)
}

fn load_processes_from_conn(
    conn: &mut sqlx::SqliteConnection,
    slug: Option<&str>,
) -> Result<Vec<ProcessRecord>> {
    use crate::db::block_on;
    use crate::db::envelope::envelope_from_row;

    const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
        causation_id, project_id, session_id, \
        actor_id, parent_actor_id, elevated, created_at";

    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events WHERE event_type LIKE 'process.%' ORDER BY created_at ASC"
        ))
        .fetch_all(&mut *conn)
        .await
    })?;

    let events: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, false))
        .collect::<Result<Vec<_>>>()?;

    let mut records = project(&events);

    if let Some(s) = slug {
        records.retain(|r| r.slug == s);
    }

    Ok(records)
}

#[cfg(test)]
#[path = "tests_process.rs"]
mod tests;
