//! `collections.*` fixtures (state / list / detail / items).
//!
//! The schema + field shapes are grounded in the real `notes` builtin
//! collection and the registry's own `fields_records` / `fields_tuples`
//! builders, so the fixture can't drift from the live wire shape. Item rows
//! match `store::Item` (`id`/`collection`/`data`/`created_by`/timestamps) with
//! `data` conforming to the `notes` schema fields (title/content/tags/category)
//! — a populated table so the designer can lay out rows, not just empty state.

use serde_json::{json, Value};

use crate::services::registry::collections::{fields_records, fields_tuples, is_builtin};
use ship_collection::builtin::notes_schema;

const ITEM_COUNT: usize = 2;

/// Two `notes` rows shaped like `store::Item`; `data` conforms to the notes
/// schema. Grounded, not invented — the shape is the real wire item.
fn item_rows() -> Vec<Value> {
    vec![
        json!({
            "id": "note_01",
            "collection": "notes",
            "data": {
                "title": "Design system audit",
                "content": "Fixed `--border-strong` (was invalid color-mix) and flattened the light surface ladder.",
                "tags": ["design", "tokens"],
                "category": "design",
            },
            "created_at": "2026-01-01T09:00:00Z",
            "updated_at": "2026-01-01T09:30:00Z",
        }),
        json!({
            "id": "note_02",
            "collection": "notes",
            "data": {
                "title": "Sidecar isolation spike",
                "content": "Daemon stays the hub; sidecar relays over remoc. Mobile via cloud relay.",
                "tags": ["infra"],
                "category": "engineering",
            },
            "created_at": "2026-01-02T14:00:00Z",
            "updated_at": "2026-01-02T14:10:00Z",
        }),
    ]
}

/// `collections.list` row for the `notes` collection (populated).
fn list_row() -> Value {
    let s = notes_schema();
    json!({
        "id": s.id,
        "name": s.title,
        "field_count": s.fields.len(),
        "fields": fields_records(&s),
        "builtin": is_builtin(&s.id),
        "item_count": ITEM_COUNT,
        "status_field": Value::Null,
        "status_values": [],
        "status_counts": {},
        "last_activity": "2026-01-02T14:10:00Z",
    })
}

pub fn examples() -> Vec<(&'static str, Value)> {
    let s = notes_schema();
    let detail_schema = json!({
        "id": s.id,
        "name": s.title,
        "fields": fields_records(&s),
        "builtin": is_builtin(&s.id),
    });
    let state_schema = json!({
        "id": s.id,
        "name": s.title,
        "fields": fields_tuples(&s),
    });
    let state_summary = json!({
        "id": s.id,
        "name": s.title,
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
                "items": item_rows(),
                "total_items": ITEM_COUNT,
            }),
        ),
        ("collections.list", json!({ "items": [list_row()] })),
        (
            "collections.detail",
            json!({ "schema": detail_schema, "items": item_rows(), "total_items": ITEM_COUNT }),
        ),
        ("collections.items", json!({ "items": item_rows(), "total_items": ITEM_COUNT })),
    ]
}
