//! Registry dispatcher for `collections.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{json, Value};

use super::wire::{
    CollectionDetailResponse, CollectionItemsResponse, CollectionsListResponse,
    CollectionsStateResponse,
};
use super::{required_str, RegistryError};
use crate::ship_runtime::ApiState;
use crate::services::schema::ArgDescriptor;

// Field tuples + field-record builders share one source so `collections.state`
// (tuples for templates) and `collections.list` / `collections.detail` (records
// for view controllers) can't drift.
pub(crate) fn fields_tuples(s: &ship_platform::collections::schema::CollectionSchema) -> Vec<Value> {
    s.fields.iter().map(|(k, v)| json!([k, {
        "field_type": format!("{:?}", v.field_type).to_lowercase(),
        "required": v.required,
        "values": v.values,
        "default": v.default,
    }])).collect()
}

pub(crate) fn fields_records(s: &ship_platform::collections::schema::CollectionSchema) -> Vec<Value> {
    s.fields.iter().map(|(k, v)| {
        let ty = format!("{:?}", v.field_type).to_lowercase();
        json!({
            "name": k,
            "type": ty.clone(),
            "field_type": ty,
            "required": v.required,
            "values": v.values,
            "default": v.default,
        })
    }).collect()
}

fn load_schemas(state: &ApiState) -> Vec<ship_platform::collections::schema::CollectionSchema> {
    let ship_dir = state.ship_dir();
    let project_dir = ship_substrate::project::get_project_dir(None).ok();
    let mut schemas = ship_platform::collections::builtin::user_schemas();
    schemas.extend(
        ship_platform::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
            .unwrap_or_default(),
    );
    schemas
}

pub(crate) fn is_builtin(id: &str) -> bool {
    ship_platform::collections::builtin::user_schemas()
        .iter()
        .any(|s| s.id == id)
}

struct CollectionSummary {
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
fn collection_summary(
    data: Option<&mut ship_substrate::data::DataService>,
    schema: &ship_platform::collections::schema::CollectionSchema,
) -> CollectionSummary {
    let status_field = schema.fields.get("status").and_then(|f| {
        matches!(f.field_type, ship_platform::collections::schema::FieldType::Enum)
            .then(|| "status".to_string())
    });
    let status_values: Vec<String> = schema
        .fields
        .get("status")
        .and_then(|f| f.values.clone())
        .unwrap_or_default();
    let Some(svc) = data else {
        return CollectionSummary {
            item_count: 0,
            status_field,
            status_values,
            status_counts: serde_json::Map::new(),
            last_activity: None,
        };
    };
    if ship_platform::collections::store::ensure_collection_table(svc, schema).is_err() {
        return CollectionSummary {
            item_count: 0,
            status_field,
            status_values,
            status_counts: serde_json::Map::new(),
            last_activity: None,
        };
    }
    let req = ship_platform::collections::query::QueryRequest {
        collection: schema.id.clone(),
        filter: Default::default(),
        sort: Default::default(),
        limit: 500,
        offset: 0,
    };
    let rows = ship_platform::collections::store::query_items(svc, schema, &req).unwrap_or_default();
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
    CollectionSummary {
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
            let ship_dir = state.ship_dir();
            let schemas = load_schemas(state);
            let active_id = args.get("active").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            // `collections.state` keeps the field tuple shape templates iterate on
            // (key, attrs); the view-controller surface uses `collections.list`.
            let summaries: Vec<Value> = schemas.iter().map(|s| json!({
                "id": s.id, "name": s.name,
                "field_count": s.fields.len(),
                "fields": fields_tuples(s),
            })).collect();
            let active_schema = active_id.as_deref()
                .and_then(|id| schemas.iter().find(|s| s.id == id))
                .or_else(|| schemas.first());
            let (schema, items, total_items, active_collection) = match active_schema {
                Some(first) => {
                    let mut items_out: Vec<Value> = Vec::new();
                    if let Ok(mut svc) = ship_platform::collections::store::open_data_db(ship_dir) {
                        let _ = ship_platform::collections::store::ensure_collection_table(&mut svc, first);
                        let req = ship_platform::collections::query::QueryRequest {
                            collection: first.id.clone(),
                            filter: Default::default(),
                            sort: Default::default(),
                            limit: 200, offset: 0,
                        };
                        if let Ok(rows) = ship_platform::collections::store::query_items(&mut svc, first, &req) {
                            items_out = rows.into_iter()
                                .map(|i| serde_json::to_value(i).unwrap_or_default())
                                .collect();
                        }
                    }
                    let schema_val = json!({
                        "id": first.id, "name": first.name,
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
            let mut svc = ship_platform::collections::store::open_data_db(state.ship_dir()).ok();
            let items: Vec<Value> = schemas.iter().map(|s| {
                let summary = collection_summary(svc.as_mut(), s);
                json!({
                    "id": s.id,
                    "name": s.name,
                    "field_count": s.fields.len(),
                    "fields": fields_records(s),
                    "builtin": is_builtin(&s.id),
                    "item_count": summary.item_count,
                    "status_field": summary.status_field,
                    "status_values": summary.status_values,
                    "status_counts": summary.status_counts,
                    "last_activity": summary.last_activity,
                })
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
                "name": schema.name,
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

/// Query up to 200 rows of a collection as serialized JSON values. Shared by
/// `collections.detail` and `collections.items` (identical body before).
fn query_collection_items(
    state: &ApiState,
    schema: &ship_platform::collections::schema::CollectionSchema,
) -> Vec<Value> {
    let mut items_out: Vec<Value> = Vec::new();
    if let Ok(mut svc) = ship_platform::collections::store::open_data_db(state.ship_dir()) {
        let _ = ship_platform::collections::store::ensure_collection_table(&mut svc, schema);
        let req = ship_platform::collections::query::QueryRequest {
            collection: schema.id.clone(),
            filter: Default::default(),
            sort: Default::default(),
            limit: 200,
            offset: 0,
        };
        if let Ok(rows) = ship_platform::collections::store::query_items(&mut svc, schema, &req) {
            items_out = rows
                .into_iter()
                .map(|i| serde_json::to_value(i).unwrap_or_default())
                .collect();
        }
    }
    items_out
}
