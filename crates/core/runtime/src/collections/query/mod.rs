//! Collection query builder — structured filters to SQL via sea-query.
//!
//! Filter keys use Django-style `field__op` format:
//!   - `"field": value`         -> equality (field = value)
//!   - `"field__ne": value`     -> not equal
//!   - `"field__gt": value`     -> greater than
//!   - `"field__gte": value`    -> greater than or equal
//!   - `"field__lt": value`     -> less than
//!   - `"field__lte": value`    -> less than or equal
//!   - `"field__in": [a,b,c]`   -> IN list
//!   - `"field__nin": [a,b,c]`  -> NOT IN list
//!   - `"field__contains": "s"` -> LIKE '%s%'
//!   - `"field__exists": bool`  -> IS NOT NULL / IS NULL
//!   - `"_or": [{...}, {...}]`  -> OR conditions

use sea_query::{Alias, Expr, Order, Query, SqliteQueryBuilder};
use serde::Deserialize;
use std::collections::BTreeMap;

mod query_filter;
use query_filter::{apply_condition, apply_or_group, parse_filter_key};

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

    for (key, value) in &req.filter {
        if key == "_or" {
            if let Some(conditions) = value.as_array() {
                apply_or_group(&mut query, conditions);
            }
            continue;
        }
        let (field, op) = parse_filter_key(key);
        if !is_safe_identifier(&field) {
            tracing::warn!(field = %field, "query: skipping unsafe field name");
            continue;
        }
        let col = Alias::new(field.as_str());
        apply_condition(&mut query, &col, &op, value);
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

/// Table alias for a collection. Prefix prevents collision with system tables.
pub fn collection_table(collection: &str) -> Alias {
    Alias::new(&format!("collection_{collection}"))
}

/// Validate a field name is a safe SQL identifier.
pub(crate) fn is_safe_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
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
        assert!(sql.contains("= ?"), "sql: {sql}");
        assert!(!values.0.is_empty(), "should have bound values");
    }

    #[test]
    fn gte_operator() {
        let req = query(json!({ "priority__gte": 3 }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains(">="), "sql: {sql}");
    }

    #[test]
    fn in_operator() {
        let req = query(json!({ "status__in": ["todo", "doing"] }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("IN"), "sql: {sql}");
    }

    #[test]
    fn exists_operator() {
        let req = query(json!({ "assignee__exists": true }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("NOT NULL") || sql.contains("IS NOT NULL"), "sql: {sql}");
    }

    #[test]
    fn ne_operator() {
        let req = query(json!({ "status__ne": "done" }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("<>") || sql.contains("!="), "sql: {sql}");
    }

    #[test]
    fn multiple_filters() {
        let req = query(json!({ "status": "todo", "priority__gte": 2 }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("AND"), "sql: {sql}");
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
    fn nin_operator() {
        let req = query(json!({ "status__nin": ["done", "doing"] }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("NOT IN"), "sql: {sql}");
    }

    #[test]
    fn contains_operator() {
        let req = query(json!({ "title__contains": "test" }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("LIKE"), "sql: {sql}");
    }

    #[test]
    fn or_operator() {
        let req = query(json!({ "_or": [{"status": "todo"}, {"status": "doing"}] }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("OR"), "sql: {sql}");
    }

    #[test]
    fn gt_operator() {
        let req = query(json!({ "priority__gt": 2 }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains(">"), "sql: {sql}");
        // Must not contain >= (that would be gte)
        assert!(!sql.contains(">="), "sql: {sql}");
    }

    #[test]
    fn lt_operator() {
        let req = query(json!({ "priority__lt": 5 }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("<"), "sql: {sql}");
    }

    #[test]
    fn lte_operator() {
        let req = query(json!({ "priority__lte": 3 }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("<="), "sql: {sql}");
    }

    #[test]
    fn exists_false_operator() {
        let req = query(json!({ "assignee__exists": false }));
        let (sql, _) = build_select(&req);
        assert!(sql.contains("IS NULL"), "sql: {sql}");
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
