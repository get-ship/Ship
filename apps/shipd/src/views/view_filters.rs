//! Custom minijinja filters and functions for user-authored views.
//!
//! Registered in `render_view_source` so `.ship/views/` templates can use
//! `ship_items('tasks')`, `| where('status', 'active')`, etc.

use minijinja::Value;

pub use super::view_filters_list::{filter_sort_by, filter_take, filter_where};

/// Global function: `ship_items(collection_name)`.
/// Returns all items from a collection as a list of objects with
/// `id`, `data`, `created_at`, `updated_at`. Returns empty list on error.
pub fn ship_items(collection: String) -> Vec<Value> {
    let ship_dir = match ship_application::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let project_dir = ship_application::project::get_project_dir(None).ok();
    let schemas = ship_platform::collections::schema::load_schemas_merged(&ship_dir, project_dir.as_deref())
        .unwrap_or_default();
    let schema = match schemas.iter().find(|s| s.id == collection) {
        Some(s) => s,
        None => return vec![],
    };
    let mut svc = match ship_platform::collections::store::open_data_db(&ship_dir) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let _ = ship_platform::collections::store::ensure_collection_table(&mut svc, schema);
    let req = ship_platform::collections::query::QueryRequest {
        collection: collection.clone(),
        filter: std::collections::BTreeMap::new(),
        sort: std::collections::BTreeMap::new(),
        limit: 200,
        offset: 0,
    };
    let items = match ship_platform::collections::store::query_items(&mut svc, schema, &req) {
        Ok(items) => items,
        Err(_) => return vec![],
    };
    items
        .into_iter()
        .map(|item| Value::from_serialize(&item))
        .collect()
}

/// Global function: `ship_schema(collection_name)`.
/// Returns the schema for a collection, or undefined on error.
pub fn ship_schema(collection: String) -> Value {
    let ship_dir = match ship_application::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return Value::UNDEFINED,
    };
    let project_dir = ship_application::project::get_project_dir(None).ok();
    let schemas = ship_platform::collections::schema::load_schemas_merged(&ship_dir, project_dir.as_deref())
        .unwrap_or_default();
    match schemas.into_iter().find(|s| s.id == collection) {
        Some(s) => Value::from_serialize(&s),
        None => Value::UNDEFINED,
    }
}

/// Global function: `ship_query('task', {status: 'active', priority__gte: 3, _sort: '-priority'})`.
/// SQL pushdown through query.rs. Keys starting with `_` are query options.
pub fn ship_query(collection: String, opts: Option<Value>) -> Vec<Value> {
    let ship_dir = match ship_application::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let project_dir = ship_application::project::get_project_dir(None).ok();
    let schemas = ship_platform::collections::schema::load_schemas_merged(&ship_dir, project_dir.as_deref())
        .unwrap_or_default();
    let schema = match schemas.iter().find(|s| s.id == collection) {
        Some(s) => s,
        None => return vec![],
    };
    let mut svc = match ship_platform::collections::store::open_data_db(&ship_dir) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let _ = ship_platform::collections::store::ensure_collection_table(&mut svc, schema);
    let req = build_query_request(&collection, &opts);
    let items = match ship_platform::collections::store::query_items(&mut svc, schema, &req) {
        Ok(items) => items,
        Err(_) => return vec![],
    };
    items
        .into_iter()
        .map(|item| Value::from_serialize(&item))
        .collect()
}

/// Parse a minijinja Value opts object into a QueryRequest.
fn build_query_request(
    collection: &str,
    opts: &Option<Value>,
) -> ship_platform::collections::query::QueryRequest {
    let mut filter = std::collections::BTreeMap::new();
    let mut sort = std::collections::BTreeMap::new();
    let mut limit: u32 = 200;
    let mut offset: u32 = 0;

    if let Some(opts_val) = opts {
        if let Ok(obj) = serde_json::to_value(opts_val.clone()) {
            if let Some(map) = obj.as_object() {
                for (key, val) in map {
                    match key.as_str() {
                        "_sort" => parse_sort_string(val, &mut sort),
                        "_limit" => {
                            if let Some(n) = val.as_u64() { limit = n as u32; }
                        }
                        "_offset" => {
                            if let Some(n) = val.as_u64() { offset = n as u32; }
                        }
                        _ if key.starts_with('_') => {}
                        _ => { filter.insert(key.clone(), val.clone()); }
                    }
                }
            }
        }
    }

    ship_platform::collections::query::QueryRequest {
        collection: collection.to_string(),
        filter,
        sort,
        limit,
        offset,
    }
}

fn parse_sort_string(val: &serde_json::Value, sort: &mut std::collections::BTreeMap<String, i8>) {
    if let Some(s) = val.as_str() {
        for part in s.split(',') {
            let part = part.trim();
            if let Some(field) = part.strip_prefix('-') {
                sort.insert(field.to_string(), -1);
            } else {
                sort.insert(part.to_string(), 1);
            }
        }
    }
}

// Legacy globals `ship_collections()`, `ship_agents()`, `ship_agent(id)`,
// `ship_skills()` were removed after the dashboard migration. Use the
// `ship.<domain>.summaries()/get()/detail()` SDK via the registry instead.

/// Open a project-scoped ArtifactService and shared DataService for canvas
/// queries. Returns `None` when no project workspace is attached or the
/// data store can't be opened — view filter callers degrade to an empty
/// list rather than failing the render.
fn canvas_artifact_service()
-> Option<(ship_platform::artifacts::ArtifactService, ship_substrate::data::DataService)> {
    let global = ship_application::project::get_global_dir().ok()?;
    let project_ship = ship_application::project::get_project_dir(None).ok()?;
    let slug = ship_application::project::project_slug_from_ship_dir(&project_ship);
    let mut svc = ship_substrate::data::DataService::open(&global).ok()?;
    ship_platform::artifacts::ensure_artifacts_collection(&mut svc).ok()?;
    let _ = slug;
    Some((ship_platform::artifacts::ArtifactService::new(&global), svc))
}

/// Global function: `ship_canvas_docs()`.
/// Lists canvas artifacts (rows in the built-in `artifacts` collection
/// tagged with `collection_id = "canvas"`), sorted newest first.
///
/// Returns `[{slug, title, modified, mime, size}]`. `slug` is the artifact
/// id (ULID); `title` is the artifact's `name` field set by the agent at
/// upload time.
pub fn ship_canvas_docs() -> Vec<Value> {
    let Some((artifacts, mut data)) = canvas_artifact_service() else {
        return vec![];
    };
    let mut filter = std::collections::BTreeMap::new();
    filter.insert("collection_id".to_string(), serde_json::Value::String("canvas".to_string()));
    let mut sort = std::collections::BTreeMap::new();
    sort.insert("created_at".to_string(), -1);
    let req = ship_platform::collections::query::QueryRequest {
        collection: ship_platform::artifacts::ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort,
        limit: 100,
        offset: 0,
    };
    match artifacts.list(&mut data, req) {
        Ok(records) => records
            .into_iter()
            .map(|r| {
                Value::from_serialize(&serde_json::json!({
                    "slug": r.id,
                    "title": r.name,
                    "modified": r.created_at,
                    "mime": r.mime,
                    "size": r.size,
                }))
            })
            .collect(),
        Err(_) => vec![],
    }
}

/// Global function: `ship_canvas_doc_content(slug)`.
/// Loads the HTML bytes of a canvas artifact (by id) for inline rendering.
/// Returns an empty string if the artifact is missing or its blob can't be
/// read — the template falls back to its own empty state.
pub fn ship_canvas_doc_content(slug: String) -> String {
    let Some((artifacts, mut data)) = canvas_artifact_service() else {
        return String::new();
    };
    match artifacts.get(&mut data, &slug) {
        Ok(Some((_record, bytes))) => String::from_utf8_lossy(&bytes).into_owned(),
        _ => String::new(),
    }
}

/// Register all SDK filters and functions on a minijinja Environment.
pub fn register_all(env: &mut minijinja::Environment) {
    env.add_function("ship_items", ship_items);
    env.add_function("ship_schema", ship_schema);
    env.add_function("ship_query", ship_query);
    env.add_function("ship_canvas_docs", ship_canvas_docs);
    env.add_function("ship_canvas_doc_content", ship_canvas_doc_content);
    env.add_function("ship_tokens", ship_tokens);
    env.add_filter("where", filter_where);
    env.add_filter("sort_by", filter_sort_by);
    env.add_filter("take", filter_take);
    env.add_filter("tojson", filter_tojson);
    env.add_filter("to_attr_json", filter_to_attr_json);
    // Tests used by SDK macros (e.g. _ship.html.j2's section() macro).
    env.add_test("startingwith", |s: &str, prefix: &str| s.starts_with(prefix));
    env.add_test("endingwith", |s: &str, suffix: &str| s.ends_with(suffix));
}

/// Global function: `ship_tokens()`.
/// Returns Ship's canonical design-token catalog parsed from the
/// embedded `packages/ship-set/tokens.json`. Shape is the parsed JSON
/// (`{ version, layers, tokens }`). Used by views like theme-studio
/// to render token lists server-side.
pub fn ship_tokens() -> Value {
    static TOKENS_JSON: &str =
        include_str!("../../../../packages/ship-set/tokens.json");
    match serde_json::from_str::<serde_json::Value>(TOKENS_JSON) {
        Ok(v) => Value::from_serialize(&v),
        Err(_) => Value::UNDEFINED,
    }
}

/// Filter: `| tojson` — serialize any value to a JSON string.
fn filter_tojson(value: Value) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string())
}

/// Filter: `| to_attr_json` — serialize to JSON with `"` replaced by `&quot;`
/// so the result is safe inside a double-quoted HTML attribute value.
/// Example: `data-item-data="{{ item.data | to_attr_json }}"`
/// The browser decodes &quot; back to " when you read element.dataset.itemData.
fn filter_to_attr_json(value: Value) -> String {
    serde_json::to_string(&value)
        .unwrap_or_else(|_| "null".to_string())
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "view_filters_tests.rs"]
mod tests;
