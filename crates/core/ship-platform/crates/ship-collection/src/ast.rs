//! Kysely OperationNode AST translation.
//!
//! Translates a whitelisted subset of Kysely AST JSON to a `DataStore::Query`.
//! Anything outside the whitelist is rejected with `UnsupportedQuery`.
//!
//! Supported:
//!   - SelectQueryNode, single-table FROM
//!   - WHERE: equality (=) BinaryOperationNode only
//!   - ORDER BY: one column, ascending/descending
//!   - LIMIT (from QueryReq.limit field)
//!   - cursor pagination (Query::after)
//!
//! Rejected:
//!   - RawNode anywhere in the tree
//!   - JoinNode, UnionNode, IntersectNode, GroupByNode, HavingNode
//!   - SubqueryNode
//!   - Non-equality operators

use std::collections::BTreeMap;

use ship_substrate::ports::Query;

use crate::error::CollectionError;
use crate::schema::is_safe_identifier;

/// Translate a Kysely inline AST (as JSON bytes) to a DataStore Query.
/// `cursor` and `limit` come from the QueryReq fields.
pub fn translate_ast(
    ast_bytes: &[u8],
    cursor: Option<String>,
    limit: u32,
) -> Result<Query, CollectionError> {
    let ast: serde_json::Value = serde_json::from_slice(ast_bytes)
        .map_err(|e| CollectionError::UnsupportedQuery(format!("invalid AST JSON: {e}")))?;

    reject_forbidden_nodes(&ast)?;

    let kind = ast.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    if kind != "SelectQueryNode" {
        return Err(CollectionError::UnsupportedQuery(format!(
            "unsupported top-level node kind: {kind}"
        )));
    }

    let table = extract_table(&ast)?;
    let filter = extract_where_filter(&ast)?;
    let sort_asc = extract_order_by(&ast)?;
    let effective_limit = if limit == 0 { 50 } else { limit.min(500) };

    Ok(Query {
        table: format!("collection_{table}"),
        filter,
        sort_asc,
        limit: effective_limit,
        after: cursor,
    })
}

/// Walk the AST and reject any forbidden node kinds.
fn reject_forbidden_nodes(v: &serde_json::Value) -> Result<(), CollectionError> {
    match v {
        serde_json::Value::Object(map) => {
            let kind = map.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            match kind {
                "RawNode" => {
                    return Err(CollectionError::UnsupportedQuery(
                        "RawNode is not allowed for security reasons".into(),
                    ))
                }
                "JoinNode" => {
                    return Err(CollectionError::UnsupportedQuery("JoinNode is not supported".into()))
                }
                "UnionNode" | "IntersectNode" | "ExceptNode" => {
                    return Err(CollectionError::UnsupportedQuery(
                        format!("{kind} is not supported"),
                    ))
                }
                "GroupByNode" | "HavingNode" => {
                    return Err(CollectionError::UnsupportedQuery(
                        "aggregate queries are not supported".into(),
                    ))
                }
                "SubqueryNode" => {
                    return Err(CollectionError::UnsupportedQuery("subqueries are not supported".into()))
                }
                _ => {}
            }
            for val in map.values() {
                reject_forbidden_nodes(val)?;
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                reject_forbidden_nodes(val)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn extract_table(ast: &serde_json::Value) -> Result<String, CollectionError> {
    let from_node = ast
        .get("from")
        .ok_or_else(|| CollectionError::UnsupportedQuery("missing FROM clause".into()))?;

    let froms = from_node
        .get("froms")
        .and_then(|f| f.as_array())
        .ok_or_else(|| CollectionError::UnsupportedQuery("FROM must be an array".into()))?;

    if froms.len() != 1 {
        return Err(CollectionError::UnsupportedQuery(
            "exactly one table in FROM is required".into(),
        ));
    }

    let table_node = &froms[0];
    let name = table_node
        .get("table")
        .and_then(|t| t.get("identifier"))
        .and_then(|i| i.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| CollectionError::UnsupportedQuery("cannot extract table name from AST".into()))?;

    if !is_safe_identifier(name) {
        return Err(CollectionError::UnsupportedQuery(format!(
            "table name '{name}' contains unsafe characters"
        )));
    }
    Ok(name.to_string())
}

fn extract_where_filter(
    ast: &serde_json::Value,
) -> Result<BTreeMap<String, serde_json::Value>, CollectionError> {
    let Some(where_node) = ast.get("where") else {
        return Ok(BTreeMap::new());
    };
    let expr = where_node
        .get("where")
        .ok_or_else(|| CollectionError::UnsupportedQuery("malformed WhereNode".into()))?;
    extract_equality_expr(expr)
}

fn extract_equality_expr(
    expr: &serde_json::Value,
) -> Result<BTreeMap<String, serde_json::Value>, CollectionError> {
    let kind = expr.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "BinaryOperationNode" => {
            let op = expr
                .get("op")
                .and_then(|o| o.get("operator"))
                .and_then(|o| o.as_str())
                .unwrap_or("");
            if op != "=" {
                return Err(CollectionError::UnsupportedQuery(format!(
                    "operator '{op}' is not supported (only '=' is)"
                )));
            }
            let col = extract_column_name(expr.get("leftOperand"))?;
            let val = extract_primitive_value(expr.get("rightOperand"))?;
            let mut filter = BTreeMap::new();
            filter.insert(col, val);
            Ok(filter)
        }
        other => Err(CollectionError::UnsupportedQuery(format!(
            "unsupported WHERE expression kind: {other}"
        ))),
    }
}

fn extract_column_name(node: Option<&serde_json::Value>) -> Result<String, CollectionError> {
    let node = node
        .ok_or_else(|| CollectionError::UnsupportedQuery("missing left operand".into()))?;
    let name = node
        .get("column")
        .and_then(|c| c.get("column"))
        .and_then(|c| c.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| CollectionError::UnsupportedQuery("cannot extract column name".into()))?;
    if !is_safe_identifier(name) {
        return Err(CollectionError::UnsupportedQuery(format!(
            "column '{name}' contains unsafe characters"
        )));
    }
    Ok(name.to_string())
}

fn extract_primitive_value(
    node: Option<&serde_json::Value>,
) -> Result<serde_json::Value, CollectionError> {
    let node = node
        .ok_or_else(|| CollectionError::UnsupportedQuery("missing right operand".into()))?;
    node.get("value")
        .and_then(|v| {
            let raw = v.get("value")?;
            Some(raw.clone())
        })
        .ok_or_else(|| CollectionError::UnsupportedQuery("cannot extract filter value".into()))
}

fn extract_order_by(ast: &serde_json::Value) -> Result<Option<String>, CollectionError> {
    let Some(ob_node) = ast.get("orderBy") else {
        return Ok(None);
    };
    let items = ob_node
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or_else(|| CollectionError::UnsupportedQuery("malformed OrderByNode".into()))?;
    if items.len() > 1 {
        return Err(CollectionError::UnsupportedQuery(
            "at most one ORDER BY column is supported".into(),
        ));
    }
    let Some(first) = items.first() else {
        return Ok(None);
    };
    let col_name = first
        .get("orderBy")
        .and_then(|ob| ob.get("column"))
        .and_then(|c| c.get("column"))
        .and_then(|c| c.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| CollectionError::UnsupportedQuery("cannot extract ORDER BY column".into()))?;
    if !is_safe_identifier(col_name) {
        return Err(CollectionError::UnsupportedQuery(format!(
            "ORDER BY column '{col_name}' is unsafe"
        )));
    }
    Ok(Some(col_name.to_string()))
}
