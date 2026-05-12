//! Registry dispatcher for `collections.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, html_escape, optional_usize, parse_scope, required_str};
use crate::ship_runtime::ApiState;
use ship_platform::services::files::{FilesService, Scope};

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "collections.state" => {
            let ship_dir = state.ship_dir();
            let project_dir = ship_application::project::get_project_dir(None).ok();
            let mut schemas = ship_platform::collections::builtin::user_schemas();
            schemas.extend(
                ship_platform::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
                    .unwrap_or_default(),
            );
            let active_id = args.get("active").and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            // Same shape build_view_context emitted — field tuples stay
            // as [key, {field_type, required, values, default}] pairs so
            // templates iterate without dict-helpers.
            fn fields_tuples(s: &ship_platform::collections::schema::CollectionSchema) -> Vec<serde_json::Value> {
                s.fields.iter().map(|(k, v)| json!([k, {
                    "field_type": format!("{:?}", v.field_type).to_lowercase(),
                    "required": v.required,
                    "values": v.values,
                    "default": v.default,
                }])).collect()
            }
            let summaries: Vec<_> = schemas.iter().map(|s| json!({
                "id": s.id, "name": s.name,
                "field_count": s.fields.len(),
                "fields": fields_tuples(s),
            })).collect();
            let active_schema = active_id.as_deref()
                .and_then(|id| schemas.iter().find(|s| s.id == id))
                .or_else(|| schemas.first());
            let (schema, items, total_items, active_collection) = match active_schema {
                Some(first) => {
                    let mut items_out: Vec<serde_json::Value> = Vec::new();
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
            Ok(json!({
                "collections": summaries,
                "active_collection": active_collection,
                "schema": schema,
                "items": items,
                "total_items": total_items,
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
