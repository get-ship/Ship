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
use std::sync::Arc;

use indexmap::IndexMap;

use ship_artifacts::service::ArtifactService;
use ship_artifacts::types::ArtifactScope;
use ship_collection::projection::{SchemaRegistry, ensure_collection_table};
use ship_collection::service::CollectionService;
use ship_collection::types::{FieldDef, FieldKind, SchemaDoc};
use ship_substrate::ports::{
    KernelLog, Predicate, Query, SingleBackendDataStore, Sort, SpaceContext, SpaceId,
};
use ship_types::sha256_hex;

use super::types::SnapshotMeta;

/// ID of the built-in `view_snapshots` collection.
pub const VIEW_SNAPSHOTS_COLLECTION_ID: &str = "view_snapshots";

fn def(field_type: FieldKind, required: bool) -> FieldDef {
    FieldDef { kind: field_type, required, values: None }
}

pub fn view_snapshots_schema() -> SchemaDoc {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("view_name".into(), def(FieldKind::String, true));
    fields.insert("turn_id".into(), def(FieldKind::String, true));
    fields.insert("agent_id".into(), def(FieldKind::String, true));
    fields.insert("label".into(), def(FieldKind::String, false));
    fields.insert("content_hash".into(), def(FieldKind::String, true));
    fields.insert("size_bytes".into(), def(FieldKind::Number, true));
    // v1: every snapshot is `is_full = true` (written explicitly below); diff
    // storage is a future optimisation, so the field is kept on the schema.
    fields.insert("is_full".into(), def(FieldKind::Boolean, true));
    // The artifact id holding the blob — looked up by `read_snapshot`.
    fields.insert("artifact_id".into(), def(FieldKind::String, true));
    // Project that owns the snapshot, used to scope the underlying artifact.
    // Named `scope_space` (not `space_id`) because `SingleBackendDataStore`
    // auto-adds a `space_id` partition column — a `space_id` field would
    // collide in the CREATE TABLE.
    fields.insert("scope_space".into(), def(FieldKind::String, true));

    SchemaDoc {
        id: VIEW_SNAPSHOTS_COLLECTION_ID.to_string(),
        title: Some("View Snapshots".to_string()),
        system: true,
        fields,
    }
}

/// A `CollectionService` whose registry contains the `view_snapshots` schema —
/// the snapshot collection is system-internal (no wire creates it), so it owns
/// its own registry rather than sharing the daemon's.
fn collection_service(ctx: Arc<SpaceContext>) -> CollectionService {
    let registry = SchemaRegistry::new();
    registry.insert(view_snapshots_schema());
    CollectionService { ctx, schemas: Arc::new(registry) }
}

/// Build a `SpaceContext` over the daemon DataStore for the given space.
///
/// Collection items and artifact metadata are both event-sourced (ADR D6/D7):
/// the read ctx must bind to the per-scope `spaces/<space_id>/space.db` where
/// `CollectionProjection`/`ArtifactProjection` materialize their tables, not the
/// shared data.db (consumer consistency).
pub fn space_context_for(
    kernel: ship_substrate::access::SharedKernel,
    ship_dir: &std::path::Path,
    space_id: &str,
) -> Arc<SpaceContext> {
    let space_db = ship_substrate::db::space_db_path(ship_dir, space_id);
    let data_svc = ship_substrate::data::DataService::open_at(&space_db)
        .expect("view_authoring: open space.db");
    let store = Arc::new(SingleBackendDataStore::new(data_svc, space_id));
    let kernel_log = Arc::new(KernelLog::new(kernel));
    Arc::new(SpaceContext::new(SpaceId::new(space_id), store, kernel_log))
}

/// Ensure the `view_snapshots` collection table exists. Idempotent.
pub async fn ensure_view_snapshots_collection(ctx: &SpaceContext) -> Result<()> {
    ensure_collection_table(ctx.data(), &view_snapshots_schema())
        .await
        .map_err(|e| anyhow!("ensure view_snapshots table: {e}"))
}

/// Persist a snapshot of `source` for `view_name` and return its metadata.
///
/// Steps (in order):
/// 1. Stores the bytes as a project-scoped artifact.
/// 2. Inserts a `view_snapshots` row pointing at that artifact.
/// 3. Reads back the row's `created_at` to build the returned `SnapshotMeta`.
// Eight args: each is a distinct, non-derivable snapshot field (`space_id` is
// intentionally NOT ctx's space — see the scoping comment below), and a
// params struct would churn 8 call sites for no clarity gain.
#[allow(clippy::too_many_arguments)]
pub async fn snapshot_view(
    ctx: Arc<SpaceContext>,
    artifacts: &ArtifactService,
    space_id: &str,
    view_name: &str,
    source: &str,
    turn_id: &str,
    agent_id: &str,
    label: Option<&str>,
) -> Result<SnapshotMeta> {
    ensure_view_snapshots_collection(&ctx).await?;

    let bytes = source.as_bytes();
    let content_hash = sha256_hex(bytes);
    // Scope the snapshot artifact to the CTX's space so its event-sourced
    // metadata materializes in the SAME `space.db` as the `view_snapshots`
    // collection row (read back through this ctx). The view's logical space is
    // recorded in the `scope_space` row column below for filtering. (Before the
    // artifact-metadata flip, direct-write put the row in the ctx-space
    // partition regardless of scope; emit-only routes by scope, so they must
    // agree — ADR D6/D7.)
    let scope = ArtifactScope::Project {
        space_id: ctx.space_id().to_string(),
    };
    let artifact = artifacts
        .store(
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

    let row = serde_json::json!({
        "view_name": view_name,
        "turn_id": turn_id,
        "agent_id": agent_id,
        "label": label,
        "content_hash": content_hash,
        "size_bytes": bytes.len() as u64,
        "is_full": true,
        "artifact_id": artifact.id,
        "scope_space": space_id,
    });
    let row_bytes = serde_json::to_vec(&row).context("serializing view_snapshots row")?;
    let svc = collection_service(ctx);
    let item = svc
        .create_item(VIEW_SNAPSHOTS_COLLECTION_ID, &row_bytes)
        .await
        .context("inserting view_snapshots metadata row")?;

    Ok(SnapshotMeta {
        snapshot_id: item.id,
        view_name: view_name.to_string(),
        turn_id: turn_id.to_string(),
        agent_id: agent_id.to_string(),
        timestamp: item.created_at,
        label: label.map(str::to_string),
        content_hash,
    })
}

/// List snapshots for a view, newest first. Cursor is the `created_at` of the
/// last item from the previous page; pass `None` for the first page.
///
/// The `ports::Query.after` cursor is ascending-id only, but this list pages
/// newest-first. So we sort by `created_at` descending and carry the last
/// row's `created_at` as a `<` predicate cursor (C-2 Gap 6). `created_at` is
/// rfc3339 (lexically sortable) and snapshots are monotonic, so this is a
/// stable strictly-older cursor.
pub async fn list_snapshots(
    ctx: Arc<SpaceContext>,
    view_name: &str,
    limit: u32,
    cursor: Option<&str>,
) -> Result<(Vec<SnapshotMeta>, Option<String>)> {
    ensure_view_snapshots_collection(&ctx).await?;
    let mut filters = vec![Predicate::eq(
        "view_name",
        serde_json::Value::String(view_name.to_string()),
    )];
    if let Some(c) = cursor {
        filters.push(Predicate {
            column: "created_at".into(),
            op: ship_substrate::ports::FilterOp::Lt,
            value: serde_json::Value::String(c.to_string()),
        });
    }
    let q = Query {
        table: format!("collection_{VIEW_SNAPSHOTS_COLLECTION_ID}"),
        filters,
        sort: Some(Sort { column: "created_at".into(), descending: true }),
        limit,
        after: None,
    };
    let svc = collection_service(ctx);
    let (items, _next) = svc
        .query_items_by_query(q)
        .await
        .map_err(|e| anyhow!("list view snapshots: {e}"))?;
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
        items.last().map(|it| it.created_at.clone())
    } else {
        None
    };
    Ok((out, next_cursor))
}

/// Read a snapshot's source bytes back out, decoded as UTF-8.
pub async fn read_snapshot(
    ctx: Arc<SpaceContext>,
    artifacts: &ArtifactService,
    snapshot_id: &str,
) -> Result<String> {
    ensure_view_snapshots_collection(&ctx).await?;
    let svc = collection_service(ctx);
    let item = svc
        .get_item(VIEW_SNAPSHOTS_COLLECTION_ID, snapshot_id)
        .await
        .map_err(|_| anyhow!("view snapshot '{snapshot_id}' not found"))?;
    let artifact_id = item
        .data
        .get("artifact_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("view snapshot '{snapshot_id}' missing artifact_id"))?;
    let (_, bytes) = artifacts
        .get_blob(artifact_id)
        .await
        .map_err(|_| anyhow!("view snapshot '{snapshot_id}' blob missing"))?;
    String::from_utf8(bytes)
        .with_context(|| format!("snapshot '{snapshot_id}' bytes are not valid UTF-8"))
}

/// Delete a snapshot row and its underlying artifact blob. Returns `false`
/// when the snapshot didn't exist. Used by the atomic `view.write_source`
/// path when compile fails after a fresh snapshot was taken.
pub async fn delete_snapshot(
    ctx: Arc<SpaceContext>,
    artifacts: &ArtifactService,
    snapshot_id: &str,
) -> Result<bool> {
    ensure_view_snapshots_collection(&ctx).await?;
    let svc = collection_service(ctx);
    let item = match svc.get_item(VIEW_SNAPSHOTS_COLLECTION_ID, snapshot_id).await {
        Ok(item) => item,
        Err(_) => return Ok(false),
    };
    let artifact_id = item
        .data
        .get("artifact_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    svc.delete_item(VIEW_SNAPSHOTS_COLLECTION_ID, snapshot_id)
        .await
        .map_err(|e| anyhow!("delete view snapshot '{snapshot_id}': {e}"))?;
    if let Some(aid) = artifact_id {
        let _ = artifacts.delete(&aid).await;
    }
    Ok(true)
}
