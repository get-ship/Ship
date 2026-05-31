//! Row building and conversion helpers for the collection service.

use ship_substrate::ports::Row;

use crate::types::{ItemRecord, SchemaDoc};

pub(crate) fn emit_ctx() -> ship_substrate::events::validator::EmitContext {
    use ship_substrate::events::CallerKind;
    ship_substrate::events::validator::EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        space_id: None,
        session_id: None,
    }
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub(crate) fn new_ulid() -> String {
    ulid::Ulid::new().to_string()
}

pub(crate) fn build_row(
    id: &str,
    schema: &SchemaDoc,
    data: &serde_json::Map<String, serde_json::Value>,
    created_at: &str,
    updated_at: &str,
) -> Row {
    let mut row = Row::new(id);
    row = row.with("created_at", serde_json::Value::String(created_at.to_string()));
    row = row.with("updated_at", serde_json::Value::String(updated_at.to_string()));
    for (name, _field) in &schema.fields {
        if let Some(val) = data.get(name) {
            let stored = match val {
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    serde_json::Value::String(val.to_string())
                }
                other => other.clone(),
            };
            row = row.with(name, stored);
        }
    }
    row
}

pub(crate) fn row_to_item(row: Row, collection: &str) -> ItemRecord {
    let created_at = row
        .fields
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let updated_at = row
        .fields
        .get("updated_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let data: serde_json::Map<String, serde_json::Value> = row
        .fields
        .iter()
        .filter(|(k, _)| *k != "created_at" && *k != "updated_at")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    ItemRecord {
        id: row.id,
        collection: collection.to_string(),
        data,
        created_at,
        updated_at,
    }
}
