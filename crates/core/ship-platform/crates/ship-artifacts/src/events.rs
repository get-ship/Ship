// -- Artifact events -----------------------------------------------------------
//
// Artifact METADATA is event-sourced (ADR D6/D7): the write path is emit-only
// and `crate::metadata_projection::ArtifactProjection` (L1, keystone-A
// de-bucket 2026-06-10) materializes the
// `collection_artifacts` table from these events. BLOBS stay on the
// content-addressed blob plane (not event-sourced). These payloads are
// projection-complete: every metadata column the read model needs rides the
// event so the projection never has to read prior state.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactCreated {
    pub id: String,
    pub scope: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    /// Relative blob path — the projection materializes it so reads can locate
    /// bytes without consulting the blob backend.
    pub path: String,
    /// Row creation timestamp (RFC3339) — the projection writes it verbatim.
    pub created_at: String,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub owner_actor: Option<String>,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub item_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactUpdated {
    pub id: String,
    pub scope: String,
    /// Map of changed metadata columns → new values. The projection applies a
    /// partial column UPDATE of exactly these keys (`name`/`owner_actor`/
    /// `expires_at`), leaving untouched columns alone.
    pub changed: serde_json::Value,
}

/// `artifact.moved` — a promote: insert the new metadata row (`new`) and delete
/// the old row (`old_id`) in one projection step. Both rows live in the same
/// space.db (promote changes the scope KIND, not the scope space), so this is a
/// single same-transaction insert+delete.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactMoved {
    /// Old metadata row id to delete.
    pub old_id: String,
    pub from_scope: String,
    pub to_scope: String,
    /// The full new metadata row to insert — projection-complete.
    pub new: ArtifactCreated,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactDeleted {
    pub id: String,
    pub scope: String,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
}

/// Every artifact.* event name in one slice — the domain-owned closed list
/// (keystone A), mirroring `ALL_CANONICAL_*` / `ALL_COLLECTION`. Used by the
/// full-linkage vocabulary guard in shipd `tests/event_vocabulary.rs`.
pub const ALL_ARTIFACT: &[&str] = &[
    "artifact.created",
    "artifact.deleted",
    "artifact.moved",
    "artifact.updated",
];
