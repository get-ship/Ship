//! Row conversion helpers for the collection service.

use ship_substrate::ports::Row;

use crate::types::ItemRecord;

pub(crate) fn emit_ctx(space_id: &str) -> ship_substrate::events::validator::EmitContext {
    // `collection.*` events are Space-scoped and the kernel fail-closes a
    // space-unbound caller (`ScopeViolation::NoSpaceForSpaceEvent`). Bind the
    // service's `SpaceContext` space so the daemon-internal mint carries the
    // right scope key.
    ship_substrate::events::validator::EmitContext::daemon_internal(Some(
        ship_types::SpaceId::new(space_id),
    ))
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub(crate) fn new_ulid() -> String {
    ship_substrate::gen_ulid()
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
