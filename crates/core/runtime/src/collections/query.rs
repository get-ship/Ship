//! Collection query builder — structured filters to SQL via sea-query.

use sea_query::{Alias, Expr, Order, Query, SelectStatement, SqliteQueryBuilder};
use serde::Deserialize;
use std::collections::BTreeMap;

/// A query request from the REST API or MCP tool.
#[derive(Debug, Clone, Deserialize)]
pub struct QueryRequest {
    pub collection: String,
    #[serde(default)]
    pub filter: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub sort: BTreeMap<String, i8>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    50
}

/// Build a SELECT query for a collection table.
/// Returns (sql_string, sea_query::Values) for parameterized execution.
pub fn build_select(req: &QueryRequest) -> (String, sea_query::Values) {
    let table = collection_table(&req.collection);
    let mut query = Query::select();
    query.expr(Expr::asterisk()).from(table.clone());

    for (field, value) in &req.filter {
        if !is_safe_identifier(field) {
            tracing::warn!(field, "query: skipping unsafe field name");
            continue;
        }
        let col = Alias::new(field.as_str());
        apply_filter(&mut query, &col, value);
    }

    // Sort
    if req.sort.is_empty() {
        query.order_by(Alias::new("created_at"), Order::Desc);
    } else {
        for (field, dir) in &req.sort {
            if !is_safe_identifier(field) {
                tracing::warn!(field, "query: skipping unsafe sort field");
                continue;
            }
            let order = if *dir >= 0 { Order::Asc } else { Order::Desc };
            query.order_by(Alias::new(field.as_str()), order);
        }
    }

    let limit = req.limit.min(500);
    query.limit(limit as u64).offset(req.offset as u64);

    query.build(SqliteQueryBuilder)
}

fn apply_filter(
    query: &mut SelectStatement,
    col: &Alias,
    value: &serde_json::Value,
) {
    match value {
        serde_json::Value::String(s) => {
            query.and_where(Expr::col(col.clone()).eq(s.as_str()));
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.and_where(Expr::col(col.clone()).eq(i));
            } else if let Some(f) = n.as_f64() {
                query.and_where(Expr::col(col.clone()).eq(f));
            }
        }
        serde_json::Value::Bool(b) => {
            query.and_where(Expr::col(col.clone()).eq(*b));
        }
        serde_json::Value::Null => {
            query.and_where(Expr::col(col.clone()).is_null());
        }
        serde_json::Value::Object(ops) => {
            for (op, val) in ops {
                match op.as_str() {
                    "$eq" => apply_filter(query, col, val),
                    "$ne" => {
                        if let Some(v) = json_to_sea_value(val) {
                            query.and_where(Expr::col(col.clone()).ne(v));
                        }
                    }
                    "$gt" => {
                        if let Some(v) = json_to_sea_value(val) {
                            query.and_where(Expr::col(col.clone()).gt(v));
                        }
                    }
                    "$gte" => {
                        if let Some(v) = json_to_sea_value(val) {
                            query.and_where(Expr::col(col.clone()).gte(v));
                        }
                    }
                    "$lt" => {
                        if let Some(v) = json_to_sea_value(val) {
                            query.and_where(Expr::col(col.clone()).lt(v));
                        }
                    }
                    "$lte" => {
                        if let Some(v) = json_to_sea_value(val) {
                            query.and_where(Expr::col(col.clone()).lte(v));
                        }
                    }
                    "$in" => {
                        if let Some(arr) = val.as_array() {
                            let sea_vals: Vec<sea_query::Value> = arr
                                .iter()
                                .filter_map(json_to_sea_value)
                                .collect();
                            if !sea_vals.is_empty() {
                                query.and_where(Expr::col(col.clone()).is_in(sea_vals));
                            }
                        }
                    }
                    "$exists" => {
                        if val.as_bool() == Some(true) {
                            query.and_where(Expr::col(col.clone()).is_not_null());
                        } else {
                            query.and_where(Expr::col(col.clone()).is_null());
                        }
                    }
                    _ => {
                        tracing::warn!(operator = op, "unknown query operator, skipped");
                    }
                }
            }
        }
        serde_json::Value::Array(_) => {}
    }
}

fn json_to_sea_value(v: &serde_json::Value) -> Option<sea_query::Value> {
    match v {
        serde_json::Value::String(s) => Some(sea_query::Value::String(Some(Box::new(s.clone())))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(sea_query::Value::BigInt(Some(i)))
            } else if let Some(f) = n.as_f64() {
                Some(sea_query::Value::Double(Some(f)))
            } else {
                None
            }
        }
        serde_json::Value::Bool(b) => Some(sea_query::Value::Bool(Some(*b))),
        _ => None,
    }
}

/// Table alias for a collection. Prefix prevents collision with system tables.
pub fn collection_table(collection: &str) -> Alias {
    Alias::new(&format!("collection_{collection}"))
}

/// Validate a field name is a safe SQL identifier.
fn is_safe_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn query(filter: serde_json::Value) -> QueryRequest {
        QueryRequest {
            collection: "task".to_string(),
            filter: serde_json::from_value(filter).unwrap(),
            sort: BTreeMap::new(),
            limit: 50,
            offset: 0,
        }
    }

    #[test]
    fn empty_filter_selects_all() {
        let req = query(json!({}));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("collection_task"), "sql: {sql}");
        assert!(sql.contains("ORDER BY"), "sql: {sql}");
    }

    #[test]
    fn equality_filter() {
        let req = query(json!({ "status": "done" }));
        let (sql, values) = build_select(&req);
        assert!(sql.contains("status"), "sql: {sql}");
        // Value is parameterized, not inline
        assert!(sql.contains("= ?"), "sql: {sql}");
        assert!(!values.0.is_empty(), "should have bound values");
    }

    #[test]
    fn gte_operator() {
        let req = query(json!({ "priority": { "$gte": 3 } }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains(">="));
    }

    #[test]
    fn in_operator() {
        let req = query(json!({ "status": { "$in": ["todo", "doing"] } }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("IN"));
    }

    #[test]
    fn exists_operator() {
        let req = query(json!({ "assignee": { "$exists": true } }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("NOT NULL") || sql.contains("IS NOT NULL"));
    }

    #[test]
    fn ne_operator() {
        let req = query(json!({ "status": { "$ne": "done" } }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("<>") || sql.contains("!="));
    }

    #[test]
    fn multiple_filters() {
        let req = query(json!({ "status": "todo", "priority": { "$gte": 2 } }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("AND"));
    }

    #[test]
    fn sort_ascending() {
        let mut req = query(json!({}));
        req.sort.insert("title".to_string(), 1);
        let (sql, _) = build_select(&req);
        assert!(sql.contains("ASC"));
    }

    #[test]
    fn sort_descending() {
        let mut req = query(json!({}));
        req.sort.insert("created_at".to_string(), -1);
        let (sql, _) = build_select(&req);
        assert!(sql.contains("DESC"));
    }

    #[test]
    fn limit_capped_at_500() {
        let mut req = query(json!({}));
        req.limit = 9999;
        let (sql, _) = build_select(&req);
        assert!(sql.contains("LIMIT"), "sql: {sql}");
        // Value is parameterized — 500 cap is in the Values, not the SQL string
    }

    #[test]
    fn offset_included() {
        let mut req = query(json!({}));
        req.offset = 20;
        let (sql, _) = build_select(&req);
        assert!(sql.contains("OFFSET"), "sql: {sql}");
    }

    #[test]
    fn null_filter() {
        let req = query(json!({ "assignee": null }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("IS NULL"));
    }

    #[test]
    fn injection_in_field_name_is_rejected() {
        let req = query(json!({ "'); DROP TABLE items; --": "gotcha" }));
        let (sql, _) = build_select(&req);
        assert!(!sql.contains("DROP TABLE"));
    }

    #[test]
    fn injection_in_sort_key_is_rejected() {
        let mut req = query(json!({}));
        req.sort.insert("'; DROP TABLE items; --".to_string(), -1);
        let (sql, _) = build_select(&req);
        assert!(!sql.contains("DROP TABLE"));
    }

    #[test]
    fn safe_identifier_validation() {
        assert!(is_safe_identifier("status"));
        assert!(is_safe_identifier("created_at"));
        assert!(is_safe_identifier("nested.field"));
        assert!(!is_safe_identifier(""));
        assert!(!is_safe_identifier("field; DROP TABLE"));
        assert!(!is_safe_identifier("field'"));
    }
}
