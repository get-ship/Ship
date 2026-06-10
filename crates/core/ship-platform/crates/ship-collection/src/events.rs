// -- Collection event payloads -----------------------------------------------
//
// These structs moved here from `ship_types::payload::collection` as part of
// the keystone-A de-bucket (2026-06-10): the COLLECTION domain now owns its
// events + projections at L1. The rationale is that payload structs belong
// with the domain that authors and interprets them; moving them out of the
// shared L0 `ship-types` crate follows the same pattern as the `session.*`
// family migration to `agent-def`.

use serde::{Deserialize, Serialize};

/// `collection.item_created` — projection-complete: carries the full validated
/// item so `CollectionItemProjection` can materialize the row from the event alone
/// (event-sourced, no read-back). `data` is the validated item object (schema
/// fields only); `created_at`/`updated_at` are the row timestamps.
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionItemCreated {
    pub collection_id: String,
    pub item_id: String,
    #[serde(default)]
    pub item_type: Option<String>,
    /// The full validated item data (schema fields → values). The projection
    /// writes exactly these columns into the collection table.
    pub data: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// `collection.item_updated` — projection-complete: carries the patch (the
/// fields to set) plus the new `updated_at`. The projection applies a partial
/// column update; untouched columns keep their values.
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionItemUpdated {
    pub collection_id: String,
    pub item_id: String,
    /// The patch — only the fields provided on update. The projection writes
    /// exactly these columns (plus `updated_at`); other columns are untouched.
    pub data: serde_json::Value,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionItemDeleted {
    pub collection_id: String,
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionCreated {
    pub collection_id: String,
    pub schema_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionSchemaChanged {
    pub collection_id: String,
    pub schema_json: String,
}

/// Every collection.* CRUD event name in one slice — the domain-owned closed
/// list (keystone A), mirroring agent-def's `ALL_CANONICAL_*` pattern. Used by
/// the full-linkage vocabulary guard in shipd `tests/event_vocabulary.rs`.
pub const ALL_COLLECTION: &[&str] = &[
    "collection.created",
    "collection.item_created",
    "collection.item_deleted",
    "collection.item_updated",
    "collection.schema.changed",
];
