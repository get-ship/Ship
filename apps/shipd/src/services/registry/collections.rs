//! Registry dispatcher for `collections.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).
//!
//! Reads route through the new `ship-collection` service over the daemon's
//! global `SpaceContext` (C-2). Registry ops are sync (`ship_ops!` `run:`
//! closures), so the async query bridges via `block_in_place` + the current
//! runtime handle.

use serde_json::{json, Value};

use ship_collection::types::SchemaDoc;
use ship_substrate::ports::Query;

use super::wire::{
    CollectionDetailResponse, CollectionFieldRecord, CollectionItemsResponse,
    CollectionSummary, CollectionsListResponse, CollectionsStateResponse,
};
use super::{required_str, RegistryError};
use crate::ship_runtime::ApiState;
use crate::services::schema::ArgDescriptor;

// Field tuples + field-record builders share one source so `collections.state`
// (tuples for templates) and `collections.list` / `collections.detail` (records
// for view controllers) can't drift. `default` is dropped (C-2 Gap 3).
pub(crate) fn fields_tuples(s: &SchemaDoc) -> Vec<Value> {
    s.fields.iter().map(|(k, v)| json!([k, {
        "field_type": format!("{:?}", v.kind).to_lowercase(),
        "required": v.required,
        "values": v.values,
    }])).collect()
}

pub(crate) fn fields_records(s: &SchemaDoc) -> Vec<CollectionFieldRecord> {
    s.fields.iter().map(|(k, v)| {
        let ty = format!("{:?}", v.kind).to_lowercase();
        CollectionFieldRecord {
            name: k.clone(),
            r#type: ty.clone(),
            field_type: ty,
            required: v.required,
            values: v.values.clone(),
        }
    }).collect()
}

/// All user-visible schemas (built-in user schemas + file-overlay schemas).
/// Sources from the new crate's built-ins (filtered to `!system`) plus the
/// relocated file loader.
fn load_schemas(state: &ApiState) -> Vec<SchemaDoc> {
    let ship_dir = state.ship_dir();
    let project_dir = ship_substrate::project::get_project_dir(None).ok();
    let mut schemas: Vec<SchemaDoc> = ship_collection::builtin::all_builtin_schemas()
        .into_iter()
        .filter(|s| !s.system)
        .collect();
    schemas.extend(
        ship_platform::registry::collection_schemas::load_schemas_merged(
            ship_dir,
            project_dir.as_deref(),
        )
        .unwrap_or_default(),
    );
    schemas
}

pub(crate) fn is_builtin(id: &str) -> bool {
    ship_collection::builtin::all_builtin_schemas()
        .iter()
        .any(|s| !s.system && s.id == id)
}

struct CollectionStats {
    item_count: usize,
    status_field: Option<String>,
    status_values: Vec<String>,
    status_counts: serde_json::Map<String, Value>,
    last_activity: Option<String>,
}

/// Per-collection aggregate used by `collections.list`. Reads up to 500 rows;
/// keeps the index responsive while still giving accurate counts for the
/// typical project. The status summary attaches when the schema declares a
/// field literally named `status` of type `enum`.
fn collection_summary(state: &ApiState, schema: &SchemaDoc) -> CollectionStats {
    use ship_collection::types::FieldKind;
    let status_field = schema.fields.get("status").and_then(|f| {
        matches!(f.kind, FieldKind::Enum).then(|| "status".to_string())
    });
    let status_values: Vec<String> = schema
        .fields
        .get("status")
        .and_then(|f| f.values.clone())
        .unwrap_or_default();
    let rows = query_items(state, &schema.id, 500);
    let mut counts: serde_json::Map<String, Value> = serde_json::Map::new();
    for v in &status_values { counts.insert(v.clone(), json!(0)); }
    let mut last: Option<String> = None;
    for item in &rows {
        if status_field.is_some() {
            if let Some(s) = item.data.get("status").and_then(|v| v.as_str()) {
                let key = s.to_string();
                let next = counts.get(&key).and_then(|v| v.as_u64()).unwrap_or(0) + 1;
                counts.insert(key, json!(next));
            }
        }
        if !item.updated_at.is_empty() && last.as_ref().is_none_or(|p| item.updated_at > *p) {
            last = Some(item.updated_at.clone());
        }
    }
    CollectionStats {
        item_count: rows.len(),
        status_field,
        status_values,
        status_counts: counts,
        last_activity: last,
    }
}

ship_ops! {
    COLLECTIONS_OPS;
    {
        name: "collections.state",
        desc: "Collections list + active schema + items (tier-3 app data).",
        perm: "ship.collections.read",
        args: &[ArgDescriptor { name: "active", kind: "string?", required: false }],
        ret: CollectionsStateResponse,
        run: |args, state, _files| {
            let schemas = load_schemas(state);
            let active_id = args.get("active").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            // `collections.state` keeps the field tuple shape templates iterate on
            // (key, attrs); the view-controller surface uses `collections.list`.
            let summaries: Vec<Value> = schemas.iter().map(|s| json!({
                "id": s.id, "name": s.title,
                "field_count": s.fields.len(),
                "fields": fields_tuples(s),
            })).collect();
            let active_schema = active_id.as_deref()
                .and_then(|id| schemas.iter().find(|s| s.id == id))
                .or_else(|| schemas.first());
            let (schema, items, total_items, active_collection) = match active_schema {
                Some(first) => {
                    let items_out: Vec<Value> = query_items(state, &first.id, 200)
                        .into_iter()
                        .map(|i| serde_json::to_value(i).unwrap_or_default())
                        .collect();
                    let schema_val = json!({
                        "id": first.id, "name": first.title,
                        "fields": fields_tuples(first),
                    });
                    let total = items_out.len();
                    (schema_val, items_out, total, first.id.clone())
                }
                None => (json!(null), Vec::new(), 0, String::new()),
            };
            Ok(CollectionsStateResponse {
                collections: summaries,
                active_collection,
                schema,
                items,
                total_items,
            })
        },
    },
    {
        name: "collections.list",
        desc: "Collection summaries with field records (id, name, fields[name,type,required]).",
        perm: "ship.collections.read",
        args: &[],
        ret: CollectionsListResponse,
        run: |_args, state, _files| {
            let schemas = load_schemas(state);
            let items: Vec<CollectionSummary> = schemas.iter().map(|s| {
                let summary = collection_summary(state, s);
                CollectionSummary {
                    id: s.id.clone(),
                    name: s.title.clone(),
                    field_count: s.fields.len(),
                    fields: fields_records(s),
                    builtin: is_builtin(&s.id),
                    item_count: summary.item_count,
                    status_field: summary.status_field,
                    status_values: summary.status_values,
                    status_counts: summary.status_counts,
                    last_activity: summary.last_activity,
                }
            }).collect();
            Ok(CollectionsListResponse { items })
        },
    },
    {
        name: "collections.detail",
        desc: "Single collection: schema + items + total count.",
        perm: "ship.collections.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        ret: CollectionDetailResponse,
        run: |args, state, _files| {
            let id = required_str(args, "id")?;
            let schemas = load_schemas(state);
            let schema = schemas.iter().find(|s| s.id == id)
                .ok_or_else(|| RegistryError::not_found(format!("collection `{id}`")))?;
            let items = query_collection_items(state, schema);
            let total_items = items.len();
            let schema_val = json!({
                "id": schema.id,
                "name": schema.title,
                "fields": fields_records(schema),
                "builtin": is_builtin(&schema.id),
            });
            Ok(CollectionDetailResponse { schema: schema_val, items, total_items })
        },
    },
    {
        name: "collections.items",
        desc: "Rows for a single collection.",
        perm: "ship.collections.read",
        args: &[ArgDescriptor { name: "collection", kind: "string", required: true }],
        ret: CollectionItemsResponse,
        run: |args, state, _files| {
            let collection = required_str(args, "collection")?;
            let schemas = load_schemas(state);
            let schema = schemas.iter().find(|s| s.id == collection)
                .ok_or_else(|| RegistryError::not_found(format!("collection `{collection}`")))?;
            let items = query_collection_items(state, schema);
            let total_items = items.len();
            Ok(CollectionItemsResponse { items, total_items })
        },
    },
}

/// Query up to `limit` rows of a collection via the new service, bridging the
/// async call onto the sync registry dispatch path (C-2 Gap 5). Returns the
/// raw `ItemRecord`s; the schema is unknown-to-the-registry only by id.
fn query_items(state: &ApiState, collection_id: &str, limit: u32) -> Vec<ship_collection::types::ItemRecord> {
    let q = Query {
        table: format!("collection_{collection_id}"),
        limit,
        ..Default::default()
    };
    // Space-less: no active space → no rows (no `global` fallback, S2).
    let Ok(svc) = state.collections() else {
        return Vec::new();
    };
    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(svc.query_items_by_query(q))
    });
    match result {
        Ok((rows, _)) => rows,
        Err(_) => Vec::new(),
    }
}

/// Query up to 200 rows of a collection as serialized JSON values. Shared by
/// `collections.detail` and `collections.items`.
fn query_collection_items(state: &ApiState, schema: &SchemaDoc) -> Vec<Value> {
    query_items(state, &schema.id, 200)
        .into_iter()
        .map(|i| serde_json::to_value(i).unwrap_or_default())
        .collect()
}
