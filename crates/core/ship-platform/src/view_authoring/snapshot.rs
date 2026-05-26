//! `view_snapshots` collection schema + service.
//!
//! Snapshots persist as artifacts (blob = raw `.sui` source, metadata row in
//! the `view_snapshots` collection). The metadata extras (`view_name`,
//! `turn_id`, `agent_id`, `label`, `is_full`) live on the snapshot collection
//! row, not the artifact row, so the snapshot listing is a single collection
//! query rather than a join.
//!
//! v1: every snapshot is `is_full = true` (stores the entire source). Diff
//! storage is a future optimisation; the field is reserved so the schema
//! doesn't need a migration when it lands.

use anyhow::{Context, Result, anyhow};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use indexmap::IndexMap;

use crate::artifacts::{ArtifactScope, ArtifactService, sha256_hex};
use crate::collections::query::QueryRequest;
use crate::collections::schema::{CollectionSchema, FieldDef, FieldType};
use crate::collections::service as collection_service;
use crate::collections::store;
use ship_substrate::data::DataService;
use ship_substrate::events::KernelRouter;

use super::types::SnapshotMeta;

/// ID of the built-in `view_snapshots` collection.
pub const VIEW_SNAPSHOTS_COLLECTION_ID: &str = "view_snapshots";

fn def(field_type: FieldType, required: bool, filterable: bool) -> FieldDef {
    FieldDef {
        field_type,
        required,
        filterable,
        default: None,
        values: None,
        items: None,
        properties: None,
        required_keys: None,
    }
}

/// Schema mirrors the contract block in `view-authoring-contract.md`.
pub fn view_snapshots_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("view_name".into(), def(FieldType::String, true, true));
    fields.insert("turn_id".into(), def(FieldType::String, true, true));
    fields.insert("agent_id".into(), def(FieldType::String, true, true));
    fields.insert("label".into(), def(FieldType::String, false, false));
    fields.insert("content_hash".into(), def(FieldType::String, true, true));
    fields.insert("size_bytes".into(), def(FieldType::Number, true, false));
    fields.insert(
        "is_full".into(),
        FieldDef {
            default: Some(serde_json::Value::Bool(true)),
            ..def(FieldType::Boolean, true, false)
        },
    );
    // The artifact id holding the blob — looked up by `read_snapshot`.
    fields.insert("artifact_id".into(), def(FieldType::String, true, true));
    // Project that owns the snapshot, used to scope the underlying artifact.
    fields.insert("space_id".into(), def(FieldType::String, true, true));

    CollectionSchema {
        id: VIEW_SNAPSHOTS_COLLECTION_ID.to_string(),
        name: Some("View Snapshots".to_string()),
        fields,
        system: true,
    }
}

/// Ensure the `view_snapshots` collection table exists. Idempotent.
pub fn ensure_view_snapshots_collection(svc: &mut DataService) -> Result<()> {
    store::ensure_collection_table(svc, &view_snapshots_schema())?;
    Ok(())
}

/// Persist a snapshot of `source` for `view_name` and return its metadata.
///
/// Steps (in order):
/// 1. Stores the bytes as a project-scoped artifact.
/// 2. Inserts a `view_snapshots` row pointing at that artifact.
/// 3. Reads back the row's `created_at` to build the returned `SnapshotMeta`.
pub async fn snapshot_view(
    svc: &mut DataService,
    kernel: &Arc<Mutex<KernelRouter>>,
    artifacts: &ArtifactService,
    space_id: &str,
    view_name: &str,
    source: &str,
    turn_id: &str,
    agent_id: &str,
    label: Option<&str>,
) -> Result<SnapshotMeta> {
    ensure_view_snapshots_collection(svc)?;

    let bytes = source.as_bytes();
    let content_hash = sha256_hex(bytes);
    let scope = ArtifactScope::Project {
        space_id: space_id.to_string(),
    };
    let artifact = artifacts
        .store(
            svc,
            kernel,
            &format!("view_snapshot:{view_name}"),
            "text/x-sui",
            bytes,
            &scope,
            Some(VIEW_SNAPSHOTS_COLLECTION_ID),
            None,
            Some(agent_id),
            None,
        )
        .await
        .context("persisting snapshot artifact blob")?;

    let snapshot_id = ship_substrate::gen_ulid();
    let row = serde_json::json!({
        "view_name": view_name,
        "turn_id": turn_id,
        "agent_id": agent_id,
        "label": label,
        "content_hash": content_hash,
        "size_bytes": bytes.len() as u64,
        "is_full": true,
        "artifact_id": artifact.id,
        "space_id": space_id,
    });
    let schema = view_snapshots_schema();
    let item = collection_service::create_item(
        svc,
        kernel,
        &schema,
        &snapshot_id,
        &row,
        Some(agent_id),
    )
    .await
    .context("inserting view_snapshots metadata row")?;

    Ok(SnapshotMeta {
        snapshot_id: snapshot_id.clone(),
        view_name: view_name.to_string(),
        turn_id: turn_id.to_string(),
        agent_id: agent_id.to_string(),
        timestamp: item.created_at,
        label: label.map(str::to_string),
        content_hash,
    })
}

/// List snapshots for a view, newest first. Cursor is the snapshot_id of the
/// last item from the previous page; pass `None` for the first page.
pub fn list_snapshots(
    svc: &mut DataService,
    view_name: &str,
    limit: u32,
    cursor: Option<&str>,
) -> Result<(Vec<SnapshotMeta>, Option<String>)> {
    ensure_view_snapshots_collection(svc)?;
    let mut filter: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    filter.insert(
        "view_name".into(),
        serde_json::Value::String(view_name.to_string()),
    );
    if let Some(c) = cursor {
        // Cursor is the snapshot_id (ULID) of the last item on the prior
        // page. gen_ulid is monotonic, so id__lt cursor selects strictly
        // older rows — the next page when sorted descending.
        filter.insert(
            "id__lt".into(),
            serde_json::Value::String(c.to_string()),
        );
    }
    let mut sort: BTreeMap<String, i8> = BTreeMap::new();
    sort.insert("created_at".into(), -1);
    let req = QueryRequest {
        collection: VIEW_SNAPSHOTS_COLLECTION_ID.to_string(),
        filter,
        sort,
        limit,
        offset: 0,
    };
    let schema = view_snapshots_schema();
    let items = collection_service::query_items(svc, &schema, &req)?;
    let mut out = Vec::with_capacity(items.len());
    for item in &items {
        let d = &item.data;
        out.push(SnapshotMeta {
            snapshot_id: item.id.clone(),
            view_name: d
                .get("view_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            turn_id: d
                .get("turn_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            agent_id: d
                .get("agent_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            timestamp: item.created_at.clone(),
            label: d.get("label").and_then(|v| v.as_str()).map(str::to_string),
            content_hash: d
                .get("content_hash")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        });
    }
    let next_cursor = if items.len() == limit as usize {
        items.last().map(|it| it.id.clone())
    } else {
        None
    };
    Ok((out, next_cursor))
}

/// Read a snapshot's source bytes back out, decoded as UTF-8.
pub fn read_snapshot(
    svc: &mut DataService,
    artifacts: &ArtifactService,
    snapshot_id: &str,
) -> Result<String> {
    ensure_view_snapshots_collection(svc)?;
    let schema = view_snapshots_schema();
    let item = collection_service::get_item(svc, &schema, snapshot_id)?
        .ok_or_else(|| anyhow!("view snapshot '{snapshot_id}' not found"))?;
    let artifact_id = item
        .data
        .get("artifact_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("view snapshot '{snapshot_id}' missing artifact_id"))?;
    let (_, bytes) = artifacts
        .get(svc, artifact_id)?
        .ok_or_else(|| anyhow!("view snapshot '{snapshot_id}' blob missing"))?;
    String::from_utf8(bytes)
        .with_context(|| format!("snapshot '{snapshot_id}' bytes are not valid UTF-8"))
}

/// Delete a snapshot row and its underlying artifact blob. Returns `false`
/// when the snapshot didn't exist. Used by the atomic `view.write_source`
/// path when compile fails after a fresh snapshot was taken.
pub async fn delete_snapshot(
    svc: &mut DataService,
    kernel: &Arc<Mutex<KernelRouter>>,
    artifacts: &ArtifactService,
    snapshot_id: &str,
) -> Result<bool> {
    ensure_view_snapshots_collection(svc)?;
    let schema = view_snapshots_schema();
    let Some(item) = collection_service::get_item(svc, &schema, snapshot_id)? else {
        return Ok(false);
    };
    let artifact_id = item
        .data
        .get("artifact_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let removed = collection_service::delete_item(
        svc,
        kernel,
        VIEW_SNAPSHOTS_COLLECTION_ID,
        snapshot_id,
    )
    .await?;
    if removed
        && let Some(aid) = artifact_id
    {
        let _ = artifacts.delete(svc, kernel, &aid).await;
    }
    Ok(removed)
}
