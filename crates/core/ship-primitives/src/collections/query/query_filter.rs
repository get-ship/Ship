//! Filter parsing and condition application for Django-style query keys.
//!
//! Parses `field__op` keys into (field_name, FilterOp) and applies
//! the corresponding sea-query conditions to a SELECT statement.

use sea_query::{Alias, Condition, Expr, SelectStatement, SimpleExpr};

use super::is_safe_identifier;

/// Supported filter operations parsed from key suffixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FilterOp {
    Eq, Ne, Gt, Gte, Lt, Lte, In, Nin, Contains, Exists,
}

/// Parse a filter key into (field_name, operator).
/// `"priority__gte"` -> `("priority", FilterOp::Gte)`
/// `"status"` -> `("status", FilterOp::Eq)`
pub(crate) fn parse_filter_key(key: &str) -> (String, FilterOp) {
    if let Some(pos) = key.rfind("__") {
        let field = &key[..pos];
        let op = match &key[pos + 2..] {
            "ne" => FilterOp::Ne,
            "gt" => FilterOp::Gt,
            "gte" => FilterOp::Gte,
            "lt" => FilterOp::Lt,
            "lte" => FilterOp::Lte,
            "in" => FilterOp::In,
            "nin" => FilterOp::Nin,
            "contains" => FilterOp::Contains,
            "exists" => FilterOp::Exists,
            _ => return (key.to_string(), FilterOp::Eq),
        };
        (field.to_string(), op)
    } else {
        (key.to_string(), FilterOp::Eq)
    }
}

/// Apply a single filter condition to the query.
pub(crate) fn apply_condition(
    query: &mut SelectStatement,
    col: &Alias,
    op: &FilterOp,
    value: &serde_json::Value,
) {
    if let Some(expr) = build_expr(col, op, value) {
        query.and_where(expr);
    }
}

/// Apply an `_or` condition: array of filter objects joined with OR.
pub(crate) fn apply_or_group(
    query: &mut SelectStatement,
    conditions: &[serde_json::Value],
) {
    let mut or_cond = Condition::any();
    for condition in conditions {
        if let Some(obj) = condition.as_object() {
            let mut and_cond = Condition::all();
            for (key, value) in obj {
                let (field, op) = parse_filter_key(key);
                if !is_safe_identifier(&field) {
                    continue;
                }
                let col = Alias::new(field.as_str());
                if let Some(expr) = build_expr(&col, &op, value) {
                    and_cond = and_cond.add(expr);
                }
            }
            or_cond = or_cond.add(and_cond);
        }
    }
    query.cond_where(or_cond);
}

// ── Core expression builder ──────────────────────────────────────────────────

/// Build a sea-query SimpleExpr for a (column, op, value) triple.
/// Returns None if the value is incompatible with the operator.
fn build_expr(col: &Alias, op: &FilterOp, value: &serde_json::Value) -> Option<SimpleExpr> {
    match op {
        FilterOp::Eq => build_eq_expr(col, value),
        FilterOp::Ne => json_to_sea_value(value).map(|v| Expr::col(col.clone()).ne(v)),
        FilterOp::Gt => json_to_sea_value(value).map(|v| Expr::col(col.clone()).gt(v)),
        FilterOp::Gte => json_to_sea_value(value).map(|v| Expr::col(col.clone()).gte(v)),
        FilterOp::Lt => json_to_sea_value(value).map(|v| Expr::col(col.clone()).lt(v)),
        FilterOp::Lte => json_to_sea_value(value).map(|v| Expr::col(col.clone()).lte(v)),
        FilterOp::In => build_in_expr(col, value, false),
        FilterOp::Nin => build_in_expr(col, value, true),
        FilterOp::Contains => {
            value.as_str().map(|s| Expr::col(col.clone()).like(format!("%{s}%")))
        }
        FilterOp::Exists => {
            if value.as_bool() == Some(true) {
                Some(Expr::col(col.clone()).is_not_null())
            } else {
                Some(Expr::col(col.clone()).is_null())
            }
        }
    }
}

fn build_eq_expr(col: &Alias, value: &serde_json::Value) -> Option<SimpleExpr> {
    match value {
        serde_json::Value::String(s) => Some(Expr::col(col.clone()).eq(s.as_str())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(Expr::col(col.clone()).eq(i))
            } else {
                n.as_f64().map(|f| Expr::col(col.clone()).eq(f))
            }
        }
        serde_json::Value::Bool(b) => Some(Expr::col(col.clone()).eq(*b)),
        serde_json::Value::Null => Some(Expr::col(col.clone()).is_null()),
        _ => None,
    }
}

fn build_in_expr(
    col: &Alias,
    value: &serde_json::Value,
    negated: bool,
) -> Option<SimpleExpr> {
    let arr = value.as_array()?;
    let sea_vals: Vec<sea_query::Value> = arr.iter().filter_map(json_to_sea_value).collect();
    if sea_vals.is_empty() {
        return None;
    }
    if negated {
        Some(Expr::col(col.clone()).is_not_in(sea_vals))
    } else {
        Some(Expr::col(col.clone()).is_in(sea_vals))
    }
}

fn json_to_sea_value(v: &serde_json::Value) -> Option<sea_query::Value> {
    match v {
        serde_json::Value::String(s) => Some(sea_query::Value::String(Some(Box::new(s.clone())))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(sea_query::Value::BigInt(Some(i)))
            } else {
                n.as_f64().map(|f| sea_query::Value::Double(Some(f)))
            }
        }
        serde_json::Value::Bool(b) => Some(sea_query::Value::Bool(Some(*b))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_field() {
        let (field, op) = parse_filter_key("status");
        assert_eq!(field, "status");
        assert_eq!(op, FilterOp::Eq);
    }

    #[test]
    fn parse_field_with_operator() {
        let (field, op) = parse_filter_key("priority__gte");
        assert_eq!(field, "priority");
        assert_eq!(op, FilterOp::Gte);
    }

    #[test]
    fn parse_unknown_operator_is_equality() {
        let (field, op) = parse_filter_key("field__unknown");
        assert_eq!(field, "field__unknown");
        assert_eq!(op, FilterOp::Eq);
    }

    #[test]
    fn parse_all_operators() {
        assert_eq!(parse_filter_key("f__ne").1, FilterOp::Ne);
        assert_eq!(parse_filter_key("f__gt").1, FilterOp::Gt);
        assert_eq!(parse_filter_key("f__gte").1, FilterOp::Gte);
        assert_eq!(parse_filter_key("f__lt").1, FilterOp::Lt);
        assert_eq!(parse_filter_key("f__lte").1, FilterOp::Lte);
        assert_eq!(parse_filter_key("f__in").1, FilterOp::In);
        assert_eq!(parse_filter_key("f__nin").1, FilterOp::Nin);
        assert_eq!(parse_filter_key("f__contains").1, FilterOp::Contains);
        assert_eq!(parse_filter_key("f__exists").1, FilterOp::Exists);
    }
}
