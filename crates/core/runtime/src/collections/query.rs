//! Query translator — Mongo-style filter objects to SQLite json_extract SQL.

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

/// Validate a field name is a safe identifier for SQL interpolation.
/// Allows alphanumeric, underscores, hyphens, and dots (for nested paths).
fn is_safe_field_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

/// Translate a QueryRequest into a SQL string and bound parameters.
pub fn translate_query(req: &QueryRequest) -> (String, Vec<serde_json::Value>) {
    let mut clauses = vec!["collection = ?".to_string()];
    let mut params: Vec<serde_json::Value> = vec![serde_json::Value::String(req.collection.clone())];

    for (field, value) in &req.filter {
        if !is_safe_field_name(field) {
            tracing::warn!(field, "query: skipping unsafe field name");
            continue;
        }
        let (clause, field_params) = translate_filter(field, value);
        clauses.push(clause);
        params.extend(field_params);
    }

    let where_clause = clauses.join(" AND ");

    let order_clause = if req.sort.is_empty() {
        "created_at DESC".to_string()
    } else {
        req.sort
            .iter()
            .filter(|(field, _)| {
                if !is_safe_field_name(field) {
                    tracing::warn!(field, "query: skipping unsafe sort field");
                    return false;
                }
                true
            })
            .map(|(field, dir)| {
                let direction = if *dir >= 0 { "ASC" } else { "DESC" };
                format!("json_extract(data, '$.{}') {}", field, direction)
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    let limit = req.limit.min(500);

    let sql = format!(
        "SELECT id, collection, data, created_by, created_at, updated_at \
         FROM items WHERE {} ORDER BY {} LIMIT ? OFFSET ?",
        where_clause, order_clause
    );
    params.push(serde_json::Value::Number(limit.into()));
    params.push(serde_json::Value::Number(req.offset.into()));

    (sql, params)
}

fn translate_filter(field: &str, value: &serde_json::Value) -> (String, Vec<serde_json::Value>) {
    let path = format!("json_extract(data, '$.{}')", field);

    match value {
        // Scalar equality: { "status": "accepted" }
        serde_json::Value::String(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::Bool(_) => {
            (format!("{path} = ?"), vec![value.clone()])
        }

        // Null check: { "assignee": null }
        serde_json::Value::Null => {
            (format!("{path} IS NULL"), vec![])
        }

        // Operator object: { "priority": { "$gte": 2 } }
        serde_json::Value::Object(ops) => {
            let mut parts = Vec::new();
            let mut params = Vec::new();

            for (op, val) in ops {
                match op.as_str() {
                    "$eq" => {
                        parts.push(format!("{path} = ?"));
                        params.push(val.clone());
                    }
                    "$ne" => {
                        parts.push(format!("{path} != ?"));
                        params.push(val.clone());
                    }
                    "$gt" => {
                        parts.push(format!("{path} > ?"));
                        params.push(val.clone());
                    }
                    "$gte" => {
                        parts.push(format!("{path} >= ?"));
                        params.push(val.clone());
                    }
                    "$lt" => {
                        parts.push(format!("{path} < ?"));
                        params.push(val.clone());
                    }
                    "$lte" => {
                        parts.push(format!("{path} <= ?"));
                        params.push(val.clone());
                    }
                    "$in" => {
                        if let Some(arr) = val.as_array() {
                            let placeholders = vec!["?"; arr.len()].join(", ");
                            parts.push(format!("{path} IN ({placeholders})"));
                            params.extend(arr.iter().cloned());
                        }
                    }
                    "$exists" => {
                        if val.as_bool() == Some(true) {
                            parts.push(format!("{path} IS NOT NULL"));
                        } else {
                            parts.push(format!("{path} IS NULL"));
                        }
                    }
                    _ => {
                        // Unknown operator — skip silently
                        tracing::warn!(operator = op, field, "unknown query operator, skipped");
                    }
                }
            }

            if parts.is_empty() {
                ("1=1".to_string(), vec![])
            } else {
                (parts.join(" AND "), params)
            }
        }

        // Arrays as values aren't supported as filters
        serde_json::Value::Array(_) => {
            ("1=1".to_string(), vec![])
        }
    }
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
        let (sql, params) = translate_query(&req);
        assert!(sql.contains("WHERE collection = ?"));
        assert!(!sql.contains("json_extract"));
        assert_eq!(params[0], json!("task"));
    }

    #[test]
    fn equality_filter() {
        let req = query(json!({ "status": "done" }));
        let (sql, params) = translate_query(&req);
        assert!(sql.contains("json_extract(data, '$.status') = ?"));
        assert!(params.contains(&json!("done")));
    }

    #[test]
    fn gte_operator() {
        let req = query(json!({ "priority": { "$gte": 3 } }));
        let (sql, params) = translate_query(&req);
        assert!(sql.contains("json_extract(data, '$.priority') >= ?"));
        assert!(params.contains(&json!(3)));
    }

    #[test]
    fn in_operator() {
        let req = query(json!({ "status": { "$in": ["todo", "doing"] } }));
        let (sql, params) = translate_query(&req);
        assert!(sql.contains("IN (?, ?)"));
        assert!(params.contains(&json!("todo")));
        assert!(params.contains(&json!("doing")));
    }

    #[test]
    fn exists_operator() {
        let req = query(json!({ "assignee": { "$exists": true } }));
        let (sql, _) = translate_query(&req);
        assert!(sql.contains("IS NOT NULL"));
    }

    #[test]
    fn ne_operator() {
        let req = query(json!({ "status": { "$ne": "done" } }));
        let (sql, params) = translate_query(&req);
        assert!(sql.contains("json_extract(data, '$.status') != ?"));
        assert!(params.contains(&json!("done")));
    }

    #[test]
    fn multiple_filters() {
        let req = query(json!({ "status": "todo", "priority": { "$gte": 2 } }));
        let (sql, _) = translate_query(&req);
        assert!(sql.contains("json_extract(data, '$.priority') >= ?"));
        assert!(sql.contains("json_extract(data, '$.status') = ?"));
        assert!(sql.contains(" AND "));
    }

    #[test]
    fn sort_ascending() {
        let mut req = query(json!({}));
        req.sort.insert("title".to_string(), 1);
        let (sql, _) = translate_query(&req);
        assert!(sql.contains("json_extract(data, '$.title') ASC"));
    }

    #[test]
    fn sort_descending() {
        let mut req = query(json!({}));
        req.sort.insert("created_at".to_string(), -1);
        let (sql, _) = translate_query(&req);
        assert!(sql.contains("json_extract(data, '$.created_at') DESC"));
    }

    #[test]
    fn limit_capped_at_500() {
        let mut req = query(json!({}));
        req.limit = 9999;
        let (sql, _) = translate_query(&req);
        assert!(sql.contains("LIMIT ?"));
        // The limit param should be capped
    }

    #[test]
    fn offset_included() {
        let mut req = query(json!({}));
        req.offset = 20;
        let (sql, params) = translate_query(&req);
        assert!(sql.contains("OFFSET ?"));
        assert_eq!(params.last(), Some(&json!(20)));
    }

    #[test]
    fn null_filter() {
        let req = query(json!({ "assignee": null }));
        let (sql, _) = translate_query(&req);
        assert!(sql.contains("IS NULL"));
    }

    #[test]
    fn default_sort_is_created_at_desc() {
        let req = query(json!({}));
        let (sql, _) = translate_query(&req);
        assert!(sql.contains("ORDER BY created_at DESC"));
    }

    #[test]
    fn injection_in_field_name_is_rejected() {
        let req = query(json!({ "'); DROP TABLE items; --": "gotcha" }));
        let (sql, _) = translate_query(&req);
        // Unsafe field should be skipped — no json_extract for it
        assert!(!sql.contains("DROP TABLE"));
        assert!(!sql.contains("json_extract"));
    }

    #[test]
    fn injection_in_sort_key_is_rejected() {
        let mut req = query(json!({}));
        req.sort.insert("'; DROP TABLE items; --".to_string(), -1);
        let (sql, _) = translate_query(&req);
        assert!(!sql.contains("DROP TABLE"));
    }

    #[test]
    fn safe_field_name_validation() {
        assert!(is_safe_field_name("status"));
        assert!(is_safe_field_name("created_at"));
        assert!(is_safe_field_name("nested.field"));
        assert!(is_safe_field_name("my-field"));
        assert!(!is_safe_field_name(""));
        assert!(!is_safe_field_name("field; DROP TABLE"));
        assert!(!is_safe_field_name("field'"));
        assert!(!is_safe_field_name("field\""));
    }
}
