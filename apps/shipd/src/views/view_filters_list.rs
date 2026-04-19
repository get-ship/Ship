//! In-memory list filters for minijinja: `| where`, `| sort_by`, `| take`.
//!
//! These are general-purpose filters that operate on item lists in templates,
//! separate from `ship_query()` which does SQL pushdown.

use minijinja::Value;

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
        None => val_eq(&field_val, op_or_value),
        Some(cmp_val) => {
            let op = op_or_value.to_string();
            val_cmp(&field_val, op.trim_matches('\''), cmp_val)
        }
    }
}

fn val_eq(a: &Value, b: &Value) -> bool {
    if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
        return sa == sb;
    }
    if let (Ok(fa), Ok(fb)) = (f64_from_val(a), f64_from_val(b)) {
        return (fa - fb).abs() < f64::EPSILON;
    }
    a.to_string() == b.to_string()
}

fn val_cmp(field_val: &Value, op: &str, cmp_val: &Value) -> bool {
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

/// Filter: `items | take(n)` -- limits to first n items.
pub fn filter_take(items: Vec<Value>, n: usize) -> Vec<Value> {
    items.into_iter().take(n).collect()
}
