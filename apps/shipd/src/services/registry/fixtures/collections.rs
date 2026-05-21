//! `collections.*` fixtures (state / list / detail / items).
//!
//! The schema + field shapes are grounded in the real `notes` builtin
//! collection and the registry's own `fields_records` / `fields_tuples`
//! builders, so the fixture can't drift from the live wire shape. Item rows
//! are empty (a fresh daemon has no rows) — the empty-state `collection_summary`
//! (item_count 0, no status enum on `notes`).

use serde_json::{json, Value};

use crate::services::registry::collections::{fields_records, fields_tuples, is_builtin};
use ship_platform::collections::builtin::notes_schema;

/// `collections.list` row for the `notes` collection in empty state.
fn list_row() -> Value {
    let s = notes_schema();
    json!({
        "id": s.id,
        "name": s.name,
        "field_count": s.fields.len(),
        "fields": fields_records(&s),
        "builtin": is_builtin(&s.id),
        "item_count": 0,
        "status_field": Value::Null,
        "status_values": [],
        "status_counts": {},
        "last_activity": Value::Null,
    })
}

pub fn examples() -> Vec<(&'static str, Value)> {
    let s = notes_schema();
    let detail_schema = json!({
        "id": s.id,
        "name": s.name,
        "fields": fields_records(&s),
        "builtin": is_builtin(&s.id),
    });
    let state_schema = json!({
        "id": s.id,
        "name": s.name,
        "fields": fields_tuples(&s),
    });
    let state_summary = json!({
        "id": s.id,
        "name": s.name,
        "field_count": s.fields.len(),
        "fields": fields_tuples(&s),
    });

    vec![
        (
            "collections.state",
            json!({
                "collections": [state_summary],
                "active_collection": s.id,
                "schema": state_schema,
                "items": [],
                "total_items": 0,
            }),
        ),
        ("collections.list", json!({ "items": [list_row()] })),
        (
            "collections.detail",
            json!({ "schema": detail_schema, "items": [], "total_items": 0 }),
        ),
        ("collections.items", json!({ "items": [], "total_items": 0 })),
    ]
}
