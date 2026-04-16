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
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let schemas = runtime::collections::schema::load_schemas(&ship_dir).unwrap_or_default();
    let schema = match schemas.iter().find(|s| s.id == collection) {
        Some(s) => s,
        None => return vec![],
    };
    let mut svc = match runtime::collections::store::open_docs_db(&ship_dir) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let _ = runtime::collections::store::ensure_collection_table(&mut svc, schema);
    let req = runtime::collections::query::QueryRequest {
        collection: collection.clone(),
        filter: std::collections::BTreeMap::new(),
        sort: std::collections::BTreeMap::new(),
        limit: 200,
        offset: 0,
    };
    let items = match runtime::collections::store::query_items(&mut svc, schema, &req) {
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
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return Value::UNDEFINED,
    };
    let schemas = runtime::collections::schema::load_schemas(&ship_dir).unwrap_or_default();
    match schemas.into_iter().find(|s| s.id == collection) {
        Some(s) => Value::from_serialize(&s),
        None => Value::UNDEFINED,
    }
}

/// Global function: `ship_query('task', {status: 'active', priority__gte: 3, _sort: '-priority'})`.
/// SQL pushdown through query.rs. Keys starting with `_` are query options.
pub fn ship_query(collection: String, opts: Option<Value>) -> Vec<Value> {
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let schemas = runtime::collections::schema::load_schemas(&ship_dir).unwrap_or_default();
    let schema = match schemas.iter().find(|s| s.id == collection) {
        Some(s) => s,
        None => return vec![],
    };
    let mut svc = match runtime::collections::store::open_docs_db(&ship_dir) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let _ = runtime::collections::store::ensure_collection_table(&mut svc, schema);
    let req = build_query_request(&collection, &opts);
    let items = match runtime::collections::store::query_items(&mut svc, schema, &req) {
        Ok(items) => items,
        Err(_) => return vec![],
    };
    items
        .into_iter()
        .map(|item| Value::from_serialize(&item))
        .collect()
}

/// Global function: `ship_collections()`.
/// Returns all collection schemas as `[{id, name, field_count, fields}]`.
pub fn ship_collections() -> Vec<Value> {
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let schemas = runtime::collections::schema::load_schemas(&ship_dir).unwrap_or_default();
    schemas
        .iter()
        .map(|s| {
            Value::from_serialize(&serde_json::json!({
                "id": s.id,
                "name": s.name,
                "field_count": s.fields.len(),
                "fields": s.fields,
            }))
        })
        .collect()
}

/// Parse a minijinja Value opts object into a QueryRequest.
fn build_query_request(
    collection: &str,
    opts: &Option<Value>,
) -> runtime::collections::query::QueryRequest {
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

    runtime::collections::query::QueryRequest {
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

/// Global function: `ship_agents()`.
/// Returns all agents as a list of serialized AgentSummary objects.
pub fn ship_agents() -> Vec<Value> {
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let agents = crate::views::state::list_agents(&ship_dir);
    agents
        .into_iter()
        .map(|a| Value::from_serialize(&a))
        .collect()
}

/// Global function: `ship_agent(id)`.
/// Returns a single agent detail, or UNDEFINED if not found.
pub fn ship_agent(id: String) -> Value {
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return Value::UNDEFINED,
    };
    match crate::views::state::load_agent(&ship_dir, &id) {
        Some(a) => Value::from_serialize(&a),
        None => Value::UNDEFINED,
    }
}

/// Global function: `ship_skills()`.
/// Returns all skills as a list of serialized SkillSummary objects.
pub fn ship_skills() -> Vec<Value> {
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let skills = crate::views::state::list_skills(&ship_dir);
    skills
        .into_iter()
        .map(|s| Value::from_serialize(&s))
        .collect()
}

/// Scan a canvas docs directory, populating seen map with slug -> modified time.
/// Skips slugs already in the map (project takes precedence over global).
fn scan_canvas_docs(dir: &std::path::Path, seen: &mut std::collections::HashMap<String, u64>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(slug) = name.strip_suffix(".html.j2") else { continue };
        if seen.contains_key(slug) {
            continue;
        }
        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        seen.insert(slug.to_string(), modified);
    }
}

/// Global function: `ship_canvas_docs()`.
/// Lists canvas documents at `.ship/views/canvas/docs/*.html.j2`.
/// Returns `[{slug, title, modified}]` sorted by modified desc.
/// Scans both project .ship/ and global ~/.ship/ — project wins on slug collisions.
pub fn ship_canvas_docs() -> Vec<Value> {
    let mut seen = std::collections::HashMap::<String, u64>::new();
    // Project dir first (takes precedence)
    if let Ok(proj) = runtime::project::get_project_dir(None) {
        scan_canvas_docs(&proj.join("views").join("canvas").join("docs"), &mut seen);
    }
    // Global dir as fallback
    if let Ok(global) = runtime::project::get_global_dir() {
        scan_canvas_docs(&global.join("views").join("canvas").join("docs"), &mut seen);
    }
    let mut docs: Vec<(String, u64)> = seen.into_iter().collect();
    docs.sort_by(|a, b| b.1.cmp(&a.1));
    docs.into_iter()
        .map(|(slug, modified)| {
            // Derive a human-readable title from the slug (kebab → Title Case)
            let title = slug
                .split('-')
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        Some(c) => c.to_uppercase().chain(chars).collect::<String>(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            Value::from_serialize(&serde_json::json!({
                "slug": slug,
                "title": title,
                "modified": modified,
            }))
        })
        .collect()
}

/// Register all SDK filters and functions on a minijinja Environment.
pub fn register_all(env: &mut minijinja::Environment) {
    env.add_function("ship_items", ship_items);
    env.add_function("ship_schema", ship_schema);
    env.add_function("ship_query", ship_query);
    env.add_function("ship_collections", ship_collections);
    env.add_function("ship_agents", ship_agents);
    env.add_function("ship_agent", ship_agent);
    env.add_function("ship_skills", ship_skills);
    env.add_function("ship_canvas_docs", ship_canvas_docs);
    env.add_filter("where", filter_where);
    env.add_filter("sort_by", filter_sort_by);
    env.add_filter("take", filter_take);
}

#[cfg(test)]
#[path = "view_filters_tests.rs"]
mod tests;
