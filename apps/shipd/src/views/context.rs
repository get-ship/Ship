//! Legacy per-view context builder used by `generic_sse.rs` to re-render
//! on kernel events. Each `data` key in a view manifest maps to a
//! loader function that populates the Jinja template context.
//!
//! This pathway predates the `ship.*` SDK. File-routed views do NOT
//! use it — they pull data inline via registry ops and re-render
//! through the routed SSE at `/api/sse/{*path}`. Remaining consumers:
//! canvas (manifest declares collections), setup, overview.
//!
//! When those views migrate to `ship.*`, this file + `generic_sse.rs`
//! + the `ManifestData` fields disappear.

use std::collections::HashMap;
use std::path::Path;

use crate::views::resolve::ManifestData;

pub(crate) fn build_generic_context(
    view_name: &str,
    _signals: &serde_json::Value,
    manifest_data: &Option<ManifestData>,
    query_params: &HashMap<String, String>,
) -> serde_json::Value {
    let mut ctx = serde_json::json!({
        "view_name": view_name,
        "query": query_params,
        "fragment": true,
    });
    let Ok(ship_dir) = runtime::project::get_global_dir() else { return ctx };

    load_collections(&mut ctx, manifest_data, query_params, &ship_dir);
    ctx
}

fn load_collections(
    ctx: &mut serde_json::Value,
    manifest_data: &Option<ManifestData>,
    query_params: &HashMap<String, String>,
    ship_dir: &Path,
) {
    if manifest_data
        .as_ref()
        .and_then(|d| d.collections.as_ref())
        .is_none()
    {
        return;
    }
    let project_dir = runtime::project::get_project_dir(None).ok();
    let mut schemas = runtime::collections::builtin::all_schemas();
    schemas.extend(
        runtime::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
            .unwrap_or_default(),
    );
    let active_id = query_params.get("collection").cloned();

    let summaries: Vec<serde_json::Value> = schemas
        .iter()
        .map(|s| {
            let fields: Vec<serde_json::Value> = s
                .fields
                .iter()
                .map(|(k, v)| {
                    serde_json::json!([k, {
                        "field_type": format!("{:?}", v.field_type).to_lowercase(),
                        "required": v.required,
                        "values": v.values,
                        "default": v.default,
                    }])
                })
                .collect();
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "field_count": s.fields.len(),
                "fields": fields,
            })
        })
        .collect();
    ctx["collections"] = serde_json::json!(summaries);

    let active_schema = active_id
        .as_deref()
        .and_then(|id| schemas.iter().find(|s| s.id == id))
        .or_else(|| schemas.first());

    let Some(first) = active_schema else { return };

    ctx["active_collection"] = serde_json::json!(first.id);
    let fields_list: Vec<serde_json::Value> = first
        .fields
        .iter()
        .map(|(k, v)| {
            serde_json::json!([k, {
                "field_type": format!("{:?}", v.field_type).to_lowercase(),
                "required": v.required,
                "values": v.values,
                "default": v.default,
            }])
        })
        .collect();
    ctx["schema"] = serde_json::json!({
        "id": first.id,
        "name": first.name,
        "fields": fields_list,
    });

    if let Ok(mut svc) = runtime::collections::store::open_data_db(ship_dir) {
        let _ = runtime::collections::store::ensure_collection_table(&mut svc, first);
        let req = runtime::collections::query::QueryRequest {
            collection: first.id.clone(),
            filter: std::collections::BTreeMap::new(),
            sort: std::collections::BTreeMap::new(),
            limit: 200,
            offset: 0,
        };
        if let Ok(items) = runtime::collections::store::query_items(&mut svc, first, &req) {
            let items_json: Vec<serde_json::Value> = items
                .into_iter()
                .map(|i| serde_json::to_value(i).unwrap_or_default())
                .collect();
            ctx["total_items"] = serde_json::json!(items_json.len());
            ctx["items"] = serde_json::json!(items_json);
        }
    }
}
