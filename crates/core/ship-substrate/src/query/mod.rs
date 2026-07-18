//! Kysely OperationNode AST translation.
//!
//! Translates a whitelisted subset of Kysely AST JSON into a `DataStore::Query`.
//! Anything outside the whitelist is rejected with `QueryError::Unsupported`.
//! The accept-policy is a strict subset of Kysely (see ADR
//! `relational-collections-and-bounded-query.md`, D5) — it never adds semantics
//! Kysely lacks.
//!
//! Supported:
//!   - SelectQueryNode, single-table FROM
//!   - WHERE: BinaryOperationNode with `= > >= < <=` or `IN`; AndNode chains
//!   - ORDER BY: one column, ascending or descending
//!   - LIMIT (from the caller's `limit` field)
//!   - cursor pagination (`Query::after`)
//!
//! Rejected:
//!   - RawNode anywhere in the tree
//!   - JoinNode, UnionNode, IntersectNode, ExceptNode, GroupByNode, HavingNode
//!   - SubqueryNode
//!   - Any operator outside the supported set
//!
//! The translator returns the *logical* table name (no physical prefix); the
//! caller maps logical → physical.

use crate::ports::{is_safe_ident, FilterOp, Predicate, Query, Sort};

/// Errors from AST translation. A substrate-level type — consumers map it onto
/// their own error surface.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("unsupported query: {0}")]
    Unsupported(String),
}

fn unsupported(msg: impl Into<String>) -> QueryError {
    QueryError::Unsupported(msg.into())
}

/// Maximum nesting depth walked into the caller-supplied AST (both the
/// whole-tree forbidden-node sweep and the WHERE `AndNode` chain). Real
/// queries never approach this; it exists purely to bound stack usage
/// against an adversarial AST before it can exhaust the stack.
const MAX_AST_DEPTH: usize = 64;

fn too_deep() -> QueryError {
    unsupported(format!("AST exceeds maximum nesting depth of {MAX_AST_DEPTH}"))
}

/// Scan raw JSON bytes for `{}`/`[]` nesting depth, ignoring bytes inside
/// string literals. Purely iterative (no recursion) so it is safe to run on
/// arbitrarily deep/adversarial input ahead of `serde_json::from_slice`,
/// whose own recursive-descent parser is not depth-bounded.
fn check_byte_nesting_depth(bytes: &[u8]) -> Result<(), QueryError> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escaped = false;
    for &b in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth += 1;
                if depth > MAX_AST_DEPTH {
                    return Err(too_deep());
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

/// Translate a Kysely inline AST (as JSON bytes) into a `DataStore` `Query`.
/// `cursor` and `limit` come from the caller's request fields.
pub fn translate_ast(
    ast_bytes: &[u8],
    cursor: Option<String>,
    limit: u32,
) -> Result<Query, QueryError> {
    // Bound nesting depth on the raw bytes BEFORE parsing: serde_json's own
    // recursive-descent parser can exhaust the stack while building a `Value`
    // for a sufficiently deep payload, so the check must run ahead of
    // `from_slice`, not only on the parsed tree.
    check_byte_nesting_depth(ast_bytes)?;
    let ast: serde_json::Value = serde_json::from_slice(ast_bytes)
        .map_err(|e| unsupported(format!("invalid AST JSON: {e}")))?;

    reject_forbidden_nodes(&ast, 0)?;

    let kind = ast.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    if kind != "SelectQueryNode" {
        return Err(unsupported(format!("unsupported top-level node kind: {kind}")));
    }

    let table = extract_table(&ast)?;
    let filters = extract_where(&ast)?;
    let sort = extract_order_by(&ast)?;
    let effective_limit = if limit == 0 { 50 } else { limit.min(500) };

    Ok(Query { table, filters, sort, limit: effective_limit, after: cursor })
}

/// Walk the AST and reject any forbidden node kinds. `depth` bounds
/// recursion against an adversarial AST (see `MAX_AST_DEPTH`).
fn reject_forbidden_nodes(v: &serde_json::Value, depth: usize) -> Result<(), QueryError> {
    if depth > MAX_AST_DEPTH {
        return Err(too_deep());
    }
    match v {
        serde_json::Value::Object(map) => {
            let kind = map.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            match kind {
                "RawNode" => {
                    return Err(unsupported("RawNode is not allowed for security reasons"))
                }
                "JoinNode" => return Err(unsupported("JoinNode is not supported")),
                "UnionNode" | "IntersectNode" | "ExceptNode" => {
                    return Err(unsupported(format!("{kind} is not supported")))
                }
                "GroupByNode" | "HavingNode" => {
                    return Err(unsupported("aggregate queries are not supported"))
                }
                "SubqueryNode" => return Err(unsupported("subqueries are not supported")),
                _ => {}
            }
            for val in map.values() {
                reject_forbidden_nodes(val, depth + 1)?;
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                reject_forbidden_nodes(val, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn extract_table(ast: &serde_json::Value) -> Result<String, QueryError> {
    let from_node = ast
        .get("from")
        .ok_or_else(|| unsupported("missing FROM clause"))?;
    let froms = from_node
        .get("froms")
        .and_then(|f| f.as_array())
        .ok_or_else(|| unsupported("FROM must be an array"))?;
    if froms.len() != 1 {
        return Err(unsupported("exactly one table in FROM is required"));
    }
    let name = froms[0]
        .get("table")
        .and_then(|t| t.get("identifier"))
        .and_then(|i| i.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| unsupported("cannot extract table name from AST"))?;
    if !is_safe_ident(name) {
        return Err(unsupported(format!("table name '{name}' contains unsafe characters")));
    }
    Ok(name.to_string())
}

fn extract_where(ast: &serde_json::Value) -> Result<Vec<Predicate>, QueryError> {
    let Some(where_node) = ast.get("where") else {
        return Ok(Vec::new());
    };
    let expr = where_node
        .get("where")
        .ok_or_else(|| unsupported("malformed WhereNode"))?;
    let mut preds = Vec::new();
    collect_predicates(expr, &mut preds, 0)?;
    Ok(preds)
}

/// Collect predicates from a WHERE expression. `AndNode` chains flatten into
/// multiple AND-ed predicates; a `BinaryOperationNode` yields exactly one.
/// `depth` bounds recursion against an adversarial AND-chain (see
/// `MAX_AST_DEPTH`).
fn collect_predicates(
    expr: &serde_json::Value,
    out: &mut Vec<Predicate>,
    depth: usize,
) -> Result<(), QueryError> {
    if depth > MAX_AST_DEPTH {
        return Err(too_deep());
    }
    let kind = expr.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "AndNode" => {
            let left = expr.get("left").ok_or_else(|| unsupported("AndNode missing left"))?;
            let right = expr.get("right").ok_or_else(|| unsupported("AndNode missing right"))?;
            collect_predicates(left, out, depth + 1)?;
            collect_predicates(right, out, depth + 1)?;
            Ok(())
        }
        "BinaryOperationNode" => {
            out.push(extract_binary_predicate(expr)?);
            Ok(())
        }
        other => Err(unsupported(format!("unsupported WHERE expression kind: {other}"))),
    }
}

fn extract_binary_predicate(expr: &serde_json::Value) -> Result<Predicate, QueryError> {
    let operator = expr
        .get("op")
        .and_then(|o| o.get("operator"))
        .and_then(|o| o.as_str())
        .unwrap_or("");
    let op = map_operator(operator)?;
    let column = extract_column_name(expr.get("leftOperand"))?;
    let value = match op {
        FilterOp::In => extract_value_list(expr.get("rightOperand"))?,
        _ => extract_primitive_value(expr.get("rightOperand"))?,
    };
    Ok(Predicate { column, op, value })
}

fn map_operator(operator: &str) -> Result<FilterOp, QueryError> {
    match operator {
        "=" => Ok(FilterOp::Eq),
        ">" => Ok(FilterOp::Gt),
        ">=" => Ok(FilterOp::Gte),
        "<" => Ok(FilterOp::Lt),
        "<=" => Ok(FilterOp::Lte),
        "in" => Ok(FilterOp::In),
        other => Err(unsupported(format!("operator '{other}' is not supported"))),
    }
}

fn extract_column_name(node: Option<&serde_json::Value>) -> Result<String, QueryError> {
    let node = node.ok_or_else(|| unsupported("missing left operand"))?;
    let name = node
        .get("column")
        .and_then(|c| c.get("column"))
        .and_then(|c| c.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| unsupported("cannot extract column name"))?;
    if !is_safe_ident(name) {
        return Err(unsupported(format!("column '{name}' contains unsafe characters")));
    }
    Ok(name.to_string())
}

/// A `ValueNode`/`PrimitiveValueNode` right operand → its inner scalar value.
fn extract_primitive_value(node: Option<&serde_json::Value>) -> Result<serde_json::Value, QueryError> {
    let node = node.ok_or_else(|| unsupported("missing right operand"))?;
    node.get("value")
        .and_then(|v| v.get("value").cloned())
        .ok_or_else(|| unsupported("cannot extract filter value"))
}

/// A `ValueListNode` (Kysely's `IN` right operand) → a JSON array of the inner
/// scalar values, one per `ValueNode`/`PrimitiveValueNode` element.
fn extract_value_list(node: Option<&serde_json::Value>) -> Result<serde_json::Value, QueryError> {
    let node = node.ok_or_else(|| unsupported("missing right operand"))?;
    let kind = node.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    if kind != "ValueListNode" && kind != "PrimitiveValueListNode" {
        return Err(unsupported(format!("IN requires a value list, got: {kind}")));
    }
    // PrimitiveValueListNode carries raw `values`; ValueListNode wraps each in a
    // ValueNode/PrimitiveValueNode. Handle both.
    let values = node
        .get("values")
        .and_then(|v| v.as_array())
        .ok_or_else(|| unsupported("IN value list missing 'values'"))?;
    let mut out = Vec::with_capacity(values.len());
    for v in values {
        if v.is_object() {
            out.push(extract_primitive_value(Some(v))?);
        } else {
            out.push(v.clone());
        }
    }
    Ok(serde_json::Value::Array(out))
}

fn extract_order_by(ast: &serde_json::Value) -> Result<Option<Sort>, QueryError> {
    let Some(ob_node) = ast.get("orderBy") else {
        return Ok(None);
    };
    let items = ob_node
        .get("items")
        .and_then(|i| i.as_array())
        .ok_or_else(|| unsupported("malformed OrderByNode"))?;
    if items.len() > 1 {
        return Err(unsupported("at most one ORDER BY column is supported"));
    }
    let Some(first) = items.first() else {
        return Ok(None);
    };
    let column = first
        .get("orderBy")
        .and_then(|ob| ob.get("column"))
        .and_then(|c| c.get("column"))
        .and_then(|c| c.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| unsupported("cannot extract ORDER BY column"))?;
    if !is_safe_ident(column) {
        return Err(unsupported(format!("ORDER BY column '{column}' is unsafe")));
    }
    let descending = matches!(
        first.get("direction").and_then(|d| d.as_str()),
        Some("desc") | Some("DESC")
    );
    Ok(Some(Sort { column: column.to_string(), descending }))
}
