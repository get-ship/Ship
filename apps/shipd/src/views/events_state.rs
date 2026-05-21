//! Data helpers for the `events` daemon-rendered view.
//!
//! Reads the recent-event log from `<ship_dir>/platform.db` and converts
//! envelopes into template-friendly `EventRow` values. Filtering is pure
//! in-memory so tests can exercise it without touching the DB.

use serde::Serialize;
use ship_substrate::events::EventEnvelope;
use std::collections::BTreeSet;
use std::path::Path;

/// Default cap on rows returned to the view. Enough to populate the
/// stream for typical sessions without blowing up minijinja render time.
pub const DEFAULT_LIMIT: usize = 200;

#[derive(Clone, Debug, Default, Serialize)]
pub struct EventFilter {
    pub event_type: Option<String>,
    pub actor: Option<String>,
    pub entity: Option<String>,
    pub q: Option<String>,
}

impl EventFilter {
    pub fn is_empty(&self) -> bool {
        self.event_type.as_deref().unwrap_or("").is_empty()
            && self.actor.as_deref().unwrap_or("").is_empty()
            && self.entity.as_deref().unwrap_or("").is_empty()
            && self.q.as_deref().unwrap_or("").is_empty()
    }

    pub fn matches(&self, e: &EventEnvelope) -> bool {
        if let Some(t) = self.event_type.as_deref().filter(|s| !s.is_empty())
            && !e.event_type.starts_with(t)
        {
            return false;
        }
        if let Some(a) = self.actor.as_deref().filter(|s| !s.is_empty())
            && e.actor != a
        {
            return false;
        }
        if let Some(en) = self.entity.as_deref().filter(|s| !s.is_empty())
            && e.entity_id != en
        {
            return false;
        }
        if let Some(q) = self.q.as_deref().filter(|s| !s.is_empty()) {
            let needle = q.to_lowercase();
            let hay = format!(
                "{} {} {} {}",
                e.event_type, e.entity_id, e.actor, e.payload_json
            )
            .to_lowercase();
            if !hay.contains(&needle) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EventRow {
    pub id: String,
    pub event_type: String,
    pub entity_id: String,
    pub actor: String,
    pub workspace_id: String,
    pub session_id: String,
    pub timestamp: String,
    pub payload_pretty: String,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EntityGroup {
    pub entity_id: String,
    pub count: usize,
    /// Default collapsed when more than 10 events cluster under one entity.
    pub collapsed: bool,
    pub rows: Vec<EventRow>,
}

fn to_row(e: &EventEnvelope) -> EventRow {
    let payload_pretty = serde_json::from_str::<serde_json::Value>(&e.payload_json)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| e.payload_json.clone());
    EventRow {
        id: e.id.clone(),
        event_type: e.event_type.clone(),
        entity_id: e.entity_id.clone(),
        actor: e.actor.clone(),
        workspace_id: e.project_id.clone().unwrap_or_default(),
        session_id: e.session_id.clone().unwrap_or_default(),
        timestamp: e.created_at.to_rfc3339(),
        payload_pretty,
    }
}

/// Read recent events, most recent first, applying `filter`.
pub fn list_events(ship_dir: &Path, filter: &EventFilter, limit: usize) -> Vec<EventRow> {
    let envs = ship_substrate::db::events::list_recent_events_at(ship_dir, limit.max(1))
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "list_recent_events_at failed");
            Vec::new()
        });
    envs.into_iter()
        .rev()
        .filter(|e| filter.matches(e))
        .map(|e| to_row(&e))
        .collect()
}

/// Distinct (event_type, actor) values across `rows` — drives filter chips.
pub fn distinct_types(rows: &[EventRow]) -> Vec<String> {
    rows.iter()
        .map(|r| r.event_type.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn distinct_actors(rows: &[EventRow]) -> Vec<String> {
    rows.iter()
        .map(|r| r.actor.clone())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Group `rows` by `entity_id`, preserving within-group order.
pub fn group_by_entity(rows: Vec<EventRow>) -> Vec<EntityGroup> {
    let mut groups: Vec<EntityGroup> = Vec::new();
    for row in rows {
        if let Some(g) = groups.iter_mut().find(|g| g.entity_id == row.entity_id) {
            g.rows.push(row);
            g.count += 1;
        } else {
            groups.push(EntityGroup {
                entity_id: row.entity_id.clone(),
                count: 1,
                collapsed: false,
                rows: vec![row],
            });
        }
    }
    for g in &mut groups {
        g.collapsed = g.count > 10;
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ship_substrate::events::EventEnvelope;

    fn env(id: &str, kind: &str, entity: &str, actor: &str) -> EventEnvelope {
        EventEnvelope {
            id: id.into(),
            event_type: kind.into(),
            entity_id: entity.into(),
            actor: actor.into(),
            payload_json: "{}".into(),
            version: 1,
            causation_id: None,
            project_id: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn filter_event_type_is_prefix() {
        let f = EventFilter { event_type: Some("pipeline.".into()), ..Default::default() };
        assert!(f.matches(&env("1", "pipeline.run.started", "j1", "a")));
        assert!(!f.matches(&env("2", "studio.message", "x", "a")));
    }

    #[test]
    fn group_marks_collapsed_above_ten() {
        let rows: Vec<EventRow> = (0..12)
            .map(|i| to_row(&env(&format!("{i}"), "k", "E", "a")))
            .collect();
        let groups = group_by_entity(rows);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].count, 12);
        assert!(groups[0].collapsed);
    }

    #[test]
    fn distinct_types_sorted() {
        let rows = vec![
            to_row(&env("1", "b", "x", "a")),
            to_row(&env("2", "a", "x", "a")),
            to_row(&env("3", "b", "x", "a")),
        ];
        assert_eq!(distinct_types(&rows), vec!["a".to_string(), "b".to_string()]);
    }
}
