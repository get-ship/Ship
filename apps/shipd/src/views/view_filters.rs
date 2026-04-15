//! Custom minijinja filters and functions for user-authored views.
//!
//! Registered in `render_view_source` so `.ship/views/` templates can use
//! `ship_items('tasks')`, `| where('status', 'active')`, etc.

use minijinja::Value;

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

/// Filter: `items | where('field', 'value')` or `items | where('field', '>=', 3)`.
pub fn filter_where(
    items: Vec<Value>,
    field: String,
    op_or_value: Value,
    value: Option<Value>,
) -> Vec<Value> {
    items
        .into_iter()
        .filter(|item| matches_where(item, &field, &op_or_value, &value))
        .collect()
}

fn matches_where(
    item: &Value,
    field: &str,
    op_or_value: &Value,
    value: &Option<Value>,
) -> bool {
    // Resolve field from item.data (items have {id, data, ...})
    let field_val = item
        .get_attr("data")
        .ok()
        .and_then(|d| d.get_attr(field).ok())
        .filter(|v| !v.is_undefined());
    let field_val = match field_val {
        Some(v) => v,
        None => return false,
    };

    match value {
        None => {
            // Equality: | where('status', 'active')
            val_eq(&field_val, op_or_value)
        }
        Some(cmp_val) => {
            // Comparison: | where('priority', '>=', 3)
            let op = op_or_value.to_string();
            val_cmp(&field_val, op.trim_matches('\''), cmp_val)
        }
    }
}

fn val_eq(a: &Value, b: &Value) -> bool {
    // Compare as strings if either is a string, otherwise structural
    if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
        return sa == sb;
    }
    if let (Ok(fa), Ok(fb)) = (f64_from_val(a), f64_from_val(b)) {
        return (fa - fb).abs() < f64::EPSILON;
    }
    a.to_string() == b.to_string()
}

fn val_cmp(field_val: &Value, op: &str, cmp_val: &Value) -> bool {
    // Try numeric comparison first
    if let (Ok(fv), Ok(cv)) = (f64_from_val(field_val), f64_from_val(cmp_val)) {
        return match op {
            "==" | "=" => (fv - cv).abs() < f64::EPSILON,
            "!=" | "<>" => (fv - cv).abs() >= f64::EPSILON,
            ">" => fv > cv,
            ">=" => fv >= cv,
            "<" => fv < cv,
            "<=" => fv <= cv,
            _ => false,
        };
    }
    // Fall back to string comparison
    let sv = field_val.to_string();
    let cv = cmp_val.to_string();
    match op {
        "==" | "=" => sv == cv,
        "!=" | "<>" => sv != cv,
        ">" => sv > cv,
        ">=" => sv >= cv,
        "<" => sv < cv,
        "<=" => sv <= cv,
        _ => false,
    }
}

fn f64_from_val(v: &Value) -> Result<f64, ()> {
    // Try i64 first, then parse from string representation
    if let Some(i) = v.as_i64() {
        return Ok(i as f64);
    }
    v.to_string()
        .trim_matches('\'')
        .trim_matches('"')
        .parse::<f64>()
        .map_err(|_| ())
}

/// Filter: `items | sort_by('field')` or `items | sort_by('field', true)` (reverse).
pub fn filter_sort_by(
    mut items: Vec<Value>,
    field: String,
    reverse: Option<bool>,
) -> Vec<Value> {
    let rev = reverse.unwrap_or(false);
    items.sort_by(|a, b| {
        let va = a
            .get_attr("data")
            .ok()
            .and_then(|d| d.get_attr(&field).ok());
        let vb = b
            .get_attr("data")
            .ok()
            .and_then(|d| d.get_attr(&field).ok());
        let cmp = cmp_values(&va, &vb);
        if rev { cmp.reverse() } else { cmp }
    });
    items
}

fn cmp_values(a: &Option<Value>, b: &Option<Value>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(va), Some(vb)) => {
            if let (Ok(fa), Ok(fb)) = (f64_from_val(va), f64_from_val(vb)) {
                return fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal);
            }
            va.to_string().cmp(&vb.to_string())
        }
    }
}

/// Filter: `items | take(n)` — limits to first n items.
pub fn filter_take(items: Vec<Value>, n: usize) -> Vec<Value> {
    items.into_iter().take(n).collect()
}

/// Register all SDK filters and functions on a minijinja Environment.
pub fn register_all(env: &mut minijinja::Environment) {
    env.add_function("ship_items", ship_items);
    env.add_function("ship_schema", ship_schema);
    env.add_filter("where", filter_where);
    env.add_filter("sort_by", filter_sort_by);
    env.add_filter("take", filter_take);
}

#[cfg(test)]
#[path = "view_filters_tests.rs"]
mod tests;
