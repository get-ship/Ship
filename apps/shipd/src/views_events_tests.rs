//! Tests for the events view context + template rendering.

use super::*;
use crate::views::events_state::{EntityGroup, EventFilter, EventRow, group_by_entity};
use crate::views::templates;

fn row(id: &str, kind: &str, entity: &str, actor: &str) -> EventRow {
    EventRow {
        id: id.into(),
        event_type: kind.into(),
        entity_id: entity.into(),
        actor: actor.into(),
        workspace_id: String::new(),
        session_id: String::new(),
        timestamp: "2024-01-01T00:00:00Z".into(),
        payload_pretty: "{}".into(),
    }
}

fn sample_ctx(rows: Vec<EventRow>, filter: EventFilter) -> EventsCtx {
    let total = rows.len();
    let groups = group_by_entity(rows.clone());
    EventsCtx {
        filter,
        types: vec!["job.started".into(), "studio.message".into()],
        actors: vec!["studio".into()],
        groups,
        rows,
        total,
    }
}

#[test]
fn shell_renders_filter_chips_and_row_list() {
    let rows = vec![
        row("1", "job.started", "j1", "studio"),
        row("2", "studio.message", "m1", "studio"),
    ];
    let ctx = sample_ctx(rows, EventFilter::default());
    let html = templates::render("events/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("id=\"events-root\""));
    assert!(html.contains("job.started"));
    assert!(html.contains("studio.message"));
    assert!(html.contains("id=\"events-content\""));
}

#[test]
fn content_partial_renders_groups() {
    let rows: Vec<EventRow> = (0..12)
        .map(|i| row(&format!("{i}"), "job.started", "jobA", "studio"))
        .collect();
    let ctx = sample_ctx(rows, EventFilter::default());
    let html = templates::render("events/content.html.j2", &ctx).expect("render");
    assert!(html.contains("jobA"));
    // Groups >10 default collapsed via <details> without the `open` attr.
    assert!(!html.contains("<details open"), "large groups should render closed: {html}");
}

#[test]
fn empty_state_when_no_events() {
    let ctx = sample_ctx(Vec::new(), EventFilter::default());
    let html = templates::render("events/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-empty-state"), "empty state present: {html}");
}

#[test]
fn active_filter_renders_selected_chip() {
    let rows = vec![row("1", "job.started", "j1", "studio")];
    let ctx = sample_ctx(
        rows,
        EventFilter {
            event_type: Some("job.".into()),
            ..Default::default()
        },
    );
    let html = templates::render("events/shell.html.j2", &ctx).expect("render");
    assert!(
        html.contains("selected") && html.contains("job."),
        "selected chip missing: {html}"
    );
}

#[test]
fn group_by_entity_preserves_order() {
    let rows = vec![
        row("1", "k", "a", "x"),
        row("2", "k", "b", "x"),
        row("3", "k", "a", "x"),
    ];
    let groups: Vec<EntityGroup> = group_by_entity(rows);
    assert_eq!(groups[0].entity_id, "a");
    assert_eq!(groups[0].count, 2);
    assert_eq!(groups[1].entity_id, "b");
    assert_eq!(groups[1].count, 1);
}
