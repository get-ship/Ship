//! Context builders for the generic SSE handler.
//!
//! Each `data` key in a view manifest maps to a loader function that
//! populates the Jinja template context with the relevant runtime data.

use std::collections::HashMap;
use std::path::Path;

use crate::views::resolve::ManifestData;

/// Build the Jinja context for a view based on manifest data declarations.
pub(crate) fn build_generic_context(
    view_name: &str,
    signals: &serde_json::Value,
    manifest_data: &Option<ManifestData>,
    query_params: &HashMap<String, String>,
) -> serde_json::Value {
    let mut ctx = serde_json::json!({
        "view_name": view_name,
        "query": query_params,
        "fragment": true,
    });
    let Ok(ship_dir) = runtime::project::get_global_dir() else { return ctx };

    load_collections(&mut ctx, signals, manifest_data, query_params, &ship_dir);
    load_agents(&mut ctx, manifest_data, &ship_dir);
    load_skills(&mut ctx, manifest_data, &ship_dir);
    load_settings(&mut ctx, manifest_data);
    load_events(&mut ctx, manifest_data, &ship_dir);
    load_chat(&mut ctx, manifest_data, query_params, &ship_dir);

    ctx
}

fn load_collections(
    ctx: &mut serde_json::Value,
    signals: &serde_json::Value,
    manifest_data: &Option<ManifestData>,
    query_params: &HashMap<String, String>,
    ship_dir: &Path,
) {
    let should_load = manifest_data.as_ref().and_then(|d| d.collections.as_ref()).is_some();
    if !should_load {
        return;
    }
    // Merge built-in schemas (notes + system ones) with user-defined ones
    // (global + project-local) so SSE context matches initial-render context
    // and /api/collections. We include all built-ins here so navigating to any
    // collection (including system ones) works; user-facing listing uses user_schemas().
    let project_dir = runtime::project::get_project_dir(None).ok();
    let mut schemas = runtime::collections::builtin::all_schemas();
    schemas.extend(
        runtime::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
            .unwrap_or_default(),
    );
    let active_id = signals
        .get("collection")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| query_params.get("collection").cloned());

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

fn load_agents(
    ctx: &mut serde_json::Value,
    manifest_data: &Option<ManifestData>,
    ship_dir: &Path,
) {
    if !manifest_data.as_ref().and_then(|d| d.agents).unwrap_or(false) {
        return;
    }
    let agents = crate::views::state::list_agents(ship_dir);
    let total = agents.len();
    ctx["agents"] = serde_json::to_value(&agents).unwrap_or_default();
    ctx["total"] = serde_json::json!(total);
}

fn load_skills(
    ctx: &mut serde_json::Value,
    manifest_data: &Option<ManifestData>,
    ship_dir: &Path,
) {
    if !manifest_data.as_ref().and_then(|d| d.skills).unwrap_or(false) {
        return;
    }
    let skills = crate::views::state::list_skills(ship_dir);
    ctx["total"] = serde_json::json!(skills.len());
    ctx["skills"] = serde_json::to_value(&skills).unwrap_or_default();
}

fn load_settings(ctx: &mut serde_json::Value, manifest_data: &Option<ManifestData>) {
    if !manifest_data.as_ref().and_then(|d| d.settings).unwrap_or(false) {
        return;
    }
    ctx["settings"] = serde_json::json!(true);
}

fn load_events(
    ctx: &mut serde_json::Value,
    manifest_data: &Option<ManifestData>,
    ship_dir: &Path,
) {
    if !manifest_data.as_ref().and_then(|d| d.events_data).unwrap_or(false) {
        return;
    }
    let filter = crate::views::events_state::EventFilter::default();
    let rows = crate::views::events_state::list_events(ship_dir, &filter, 200);
    let types = crate::views::events_state::distinct_types(&rows);
    let actors = crate::views::events_state::distinct_actors(&rows);
    let groups = crate::views::events_state::group_by_entity(rows.clone());
    ctx["total"] = serde_json::json!(rows.len());
    ctx["rows"] = serde_json::to_value(&rows).unwrap_or_default();
    ctx["types"] = serde_json::to_value(&types).unwrap_or_default();
    ctx["actors"] = serde_json::to_value(&actors).unwrap_or_default();
    ctx["groups"] = serde_json::to_value(&groups).unwrap_or_default();
}

fn load_chat(
    ctx: &mut serde_json::Value,
    manifest_data: &Option<ManifestData>,
    query_params: &HashMap<String, String>,
    ship_dir: &Path,
) {
    if !manifest_data.as_ref().and_then(|d| d.chat).unwrap_or(false) {
        return;
    }
    let ws = query_params
        .get("workspace_id")
        .map(|s| s.as_str())
        .filter(|s| !s.is_empty());
    let limit = crate::views::chat_state::DEFAULT_LIMIT;
    let messages = crate::views::chat_state::list_messages(ship_dir, ws, limit);
    ctx["workspace_id"] = serde_json::json!(ws.unwrap_or_default());
    ctx["total"] = serde_json::json!(messages.len());
    ctx["messages"] = serde_json::to_value(&messages).unwrap_or_default();
}

