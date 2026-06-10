//! Schema-evolution round-trips for the artifact.* payloads.
//!
//! Moved with the payload structs from ship-types/tests/payload_schema_evolution.rs
//! (keystone-A de-bucket, 2026-06-10). Same doctrine: the JSON literals are the
//! FROZEN v1 wire shapes — a struct change that breaks them is a wire break.

use ship_artifacts::events::{ArtifactCreated, ArtifactDeleted, ArtifactMoved, ArtifactUpdated};

fn parse<'de, T: serde::Deserialize<'de>>(json: &'de str, label: &str) -> T {
    serde_json::from_str(json).unwrap_or_else(|e| panic!("{label} v1 shape must parse: {e}"))
}

#[test]
fn artifact_created_v1_round_trips() {
    let json = r#"{
        "id": "a-1",
        "scope": "run",
        "name": "out.txt",
        "mime": "text/plain",
        "size": 10,
        "path": "runs/r-1/blobs/a-1",
        "created_at": "2026-01-01T00:00:00Z",
        "content_hash": "deadbeef",
        "owner_actor": "lead",
        "project_id": "p-1",
        "run_id": "r-1"
    }"#;
    let _: ArtifactCreated = parse(json, "ArtifactCreated");
}

#[test]
fn artifact_updated_v1_round_trips() {
    let json = r#"{"id": "a-1", "scope": "run", "changed": {"size": 20}}"#;
    let _: ArtifactUpdated = parse(json, "ArtifactUpdated");
}

#[test]
fn artifact_moved_v1_round_trips() {
    // `artifact.moved` (promote) carries the old id to delete + the full new
    // metadata row to insert (projection-complete, ADR D6/D7).
    let _: ArtifactMoved = parse(
        r#"{
            "old_id": "a-1",
            "from_scope": "run",
            "to_scope": "project",
            "new": {
                "id": "a-2",
                "scope": "project",
                "name": "out.txt",
                "mime": "text/plain",
                "size": 10,
                "path": "blobs/a-2",
                "created_at": "2026-01-01T00:00:00Z"
            }
        }"#,
        "ArtifactMoved",
    );
}

#[test]
fn artifact_deleted_v1_round_trips() {
    let _: ArtifactDeleted = parse(r#"{"id": "a-1", "scope": "run"}"#, "ArtifactDeleted");
}

