//! Kysely OperationNode → SQL executor spike.
//!
//! Validates that a Rust crate can consume Kysely's JSON AST, validate against
//! a whitelist, emit parameterized SQL, and execute against sqlite — all
//! without a JS runtime on the server. Companion: `src/rq.rs` proves rquickjs
//! can host Kysely + emit the AST in-process.
//!
//! Covered: SELECT + SelectAll/columns + WHERE (=, !=, <, <=, >, >=, like,
//! not like) + And/Or + ORDER BY (asc/desc) + LIMIT.
//! Not covered: JOIN, GROUP BY, subqueries, INSERT/UPDATE/DELETE (separate
//! scope; writes go through event emission).

use serde::Deserialize;
use thiserror::Error;

pub mod rq;

// ─── AST ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum Node {
    SelectQueryNode(SelectQueryBody),
    FromNode(From_),
    TableNode(Table),
    SchemableIdentifierNode(SchemableIdent),
    IdentifierNode(Ident),
    SelectionNode(Selection),
    SelectAllNode,
    ReferenceNode(Reference),
    ColumnNode(Column),
    WhereNode(Where),
    BinaryOperationNode(Binary),
    AndNode(LogicalPair),
    OrNode(LogicalPair),
    OperatorNode(Operator),
    ValueNode(Value_),
    OrderByNode(OrderBy),
    OrderByItemNode(OrderByItem),
    RawNode(Raw),
    LimitNode(Limit),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectQueryBody {
    pub from: Box<Node>,
    pub selections: Vec<Node>,
    #[serde(default)]
    pub r#where: Option<Box<Node>>,
    #[serde(default, rename = "orderBy")]
    pub order_by: Option<Box<Node>>,
    #[serde(default)]
    pub limit: Option<Box<Node>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct From_ {
    pub froms: Vec<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Table {
    pub table: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchemableIdent {
    pub identifier: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ident {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Selection {
    pub selection: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Reference {
    pub column: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Column {
    pub column: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Where {
    pub r#where: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Binary {
    #[serde(rename = "leftOperand")]
    pub left: Box<Node>,
    pub operator: Box<Node>,
    #[serde(rename = "rightOperand")]
    pub right: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogicalPair {
    pub left: Box<Node>,
    pub right: Box<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Operator {
    pub operator: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Value_ {
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBy {
    pub items: Vec<Node>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderByItem {
    #[serde(rename = "orderBy")]
    pub order_by: Box<Node>,
    #[serde(default)]
    pub direction: Option<Box<Node>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Raw {
    #[serde(rename = "sqlFragments")]
    pub sql_fragments: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Limit {
    pub limit: Box<Node>,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("expected node kind {expected}, got {actual}")]
    WrongKind { expected: &'static str, actual: &'static str },
    #[error("unsupported operator: {0}")]
    UnsupportedOperator(String),
    #[error("unsafe identifier: {0}")]
    UnsafeIdent(String),
    #[error("unsupported order-by direction: {0}")]
    UnsupportedDirection(String),
    #[error("RawNode is only allowed in order-by direction, found elsewhere")]
    RawNodeOutOfPlace,
    #[error("unsupported node in this position: {0}")]
    UnsupportedNode(&'static str),
}

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error(transparent)]
    Compile(#[from] CompileError),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

// ─── Whitelists ───────────────────────────────────────────────────────────────

const ALLOWED_BINARY_OPS: &[&str] = &["=", "!=", "<", "<=", ">", ">=", "like", "not like"];
const ALLOWED_DIRECTIONS: &[&str] = &["asc", "desc"];

fn is_safe_ident(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

// ─── Compiler: AST → SQL + parameters ─────────────────────────────────────────

pub struct Compiled {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
}

pub fn compile(ast_json: &str) -> Result<Compiled, ExecuteError> {
    let root: Node = serde_json::from_str(ast_json)?;
    let Node::SelectQueryNode(q) = root else {
        return Err(CompileError::WrongKind {
            expected: "SelectQueryNode",
            actual: node_kind(&root),
        }
        .into());
    };

    let mut sql = String::new();
    let mut params = Vec::new();

    sql.push_str("SELECT ");
    compile_selections(&q.selections, &mut sql)?;
    sql.push_str(" FROM ");
    compile_from(&q.from, &mut sql)?;

    if let Some(w) = &q.r#where {
        sql.push_str(" WHERE ");
        compile_where(w, &mut sql, &mut params)?;
    }
    if let Some(o) = &q.order_by {
        sql.push_str(" ORDER BY ");
        compile_order_by(o, &mut sql)?;
    }
    if let Some(l) = &q.limit {
        sql.push_str(" LIMIT ");
        compile_limit(l, &mut sql, &mut params)?;
    }

    Ok(Compiled { sql, params })
}

fn compile_selections(sels: &[Node], sql: &mut String) -> Result<(), CompileError> {
    let mut first = true;
    for s in sels {
        let Node::SelectionNode(sel) = s else {
            return Err(CompileError::WrongKind {
                expected: "SelectionNode",
                actual: node_kind(s),
            });
        };
        match &*sel.selection {
            Node::SelectAllNode => {
                if !first {
                    sql.push_str(", ");
                }
                sql.push('*');
            }
            Node::ReferenceNode(r) => {
                let name = expect_column_name(&r.column)?;
                if !first {
                    sql.push_str(", ");
                }
                push_quoted(sql, name);
            }
            other => {
                return Err(CompileError::WrongKind {
                    expected: "SelectAllNode|ReferenceNode",
                    actual: node_kind(other),
                });
            }
        }
        first = false;
    }
    Ok(())
}

fn compile_from(from: &Node, sql: &mut String) -> Result<(), CompileError> {
    let Node::FromNode(f) = from else {
        return Err(CompileError::WrongKind {
            expected: "FromNode",
            actual: node_kind(from),
        });
    };
    let first = f
        .froms
        .first()
        .ok_or(CompileError::UnsupportedNode("FromNode requires froms[0]"))?;
    let Node::TableNode(t) = first else {
        return Err(CompileError::WrongKind {
            expected: "TableNode",
            actual: node_kind(first),
        });
    };
    let Node::SchemableIdentifierNode(si) = &*t.table else {
        return Err(CompileError::WrongKind {
            expected: "SchemableIdentifierNode",
            actual: node_kind(&t.table),
        });
    };
    let Node::IdentifierNode(id) = &*si.identifier else {
        return Err(CompileError::WrongKind {
            expected: "IdentifierNode",
            actual: node_kind(&si.identifier),
        });
    };
    if !is_safe_ident(&id.name) {
        return Err(CompileError::UnsafeIdent(id.name.clone()));
    }
    push_quoted(sql, &id.name);
    Ok(())
}

fn compile_where(
    where_node: &Node,
    sql: &mut String,
    params: &mut Vec<serde_json::Value>,
) -> Result<(), CompileError> {
    let Node::WhereNode(w) = where_node else {
        return Err(CompileError::WrongKind {
            expected: "WhereNode",
            actual: node_kind(where_node),
        });
    };
    compile_expr(&w.r#where, sql, params)
}

fn compile_expr(
    node: &Node,
    sql: &mut String,
    params: &mut Vec<serde_json::Value>,
) -> Result<(), CompileError> {
    match node {
        Node::BinaryOperationNode(b) => {
            let Node::ReferenceNode(r) = &*b.left else {
                return Err(CompileError::WrongKind {
                    expected: "ReferenceNode (left)",
                    actual: node_kind(&b.left),
                });
            };
            let name = expect_column_name(&r.column)?;
            let Node::OperatorNode(op) = &*b.operator else {
                return Err(CompileError::WrongKind {
                    expected: "OperatorNode",
                    actual: node_kind(&b.operator),
                });
            };
            if !ALLOWED_BINARY_OPS.contains(&op.operator.as_str()) {
                return Err(CompileError::UnsupportedOperator(op.operator.clone()));
            }
            let Node::ValueNode(v) = &*b.right else {
                return Err(CompileError::WrongKind {
                    expected: "ValueNode (right)",
                    actual: node_kind(&b.right),
                });
            };
            push_quoted(sql, name);
            sql.push(' ');
            sql.push_str(&op.operator);
            sql.push_str(" ?");
            params.push(v.value.clone());
            Ok(())
        }
        Node::AndNode(a) => {
            sql.push('(');
            compile_expr(&a.left, sql, params)?;
            sql.push_str(") AND (");
            compile_expr(&a.right, sql, params)?;
            sql.push(')');
            Ok(())
        }
        Node::OrNode(o) => {
            sql.push('(');
            compile_expr(&o.left, sql, params)?;
            sql.push_str(") OR (");
            compile_expr(&o.right, sql, params)?;
            sql.push(')');
            Ok(())
        }
        Node::RawNode(_) => Err(CompileError::RawNodeOutOfPlace),
        other => Err(CompileError::UnsupportedNode(node_kind(other))),
    }
}

fn compile_order_by(order_by: &Node, sql: &mut String) -> Result<(), CompileError> {
    let Node::OrderByNode(o) = order_by else {
        return Err(CompileError::WrongKind {
            expected: "OrderByNode",
            actual: node_kind(order_by),
        });
    };
    let mut first = true;
    for item in &o.items {
        let Node::OrderByItemNode(it) = item else {
            return Err(CompileError::WrongKind {
                expected: "OrderByItemNode",
                actual: node_kind(item),
            });
        };
        let Node::ReferenceNode(r) = &*it.order_by else {
            return Err(CompileError::WrongKind {
                expected: "ReferenceNode (order_by)",
                actual: node_kind(&it.order_by),
            });
        };
        let name = expect_column_name(&r.column)?;
        if !first {
            sql.push_str(", ");
        }
        push_quoted(sql, name);
        if let Some(dir) = &it.direction {
            let Node::RawNode(raw) = &**dir else {
                return Err(CompileError::WrongKind {
                    expected: "RawNode (direction)",
                    actual: node_kind(dir),
                });
            };
            let frag = raw
                .sql_fragments
                .first()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if !ALLOWED_DIRECTIONS.contains(&frag.as_str()) {
                return Err(CompileError::UnsupportedDirection(frag));
            }
            sql.push(' ');
            sql.push_str(&frag);
        }
        first = false;
    }
    Ok(())
}

fn compile_limit(
    limit: &Node,
    sql: &mut String,
    params: &mut Vec<serde_json::Value>,
) -> Result<(), CompileError> {
    let Node::LimitNode(l) = limit else {
        return Err(CompileError::WrongKind {
            expected: "LimitNode",
            actual: node_kind(limit),
        });
    };
    let Node::ValueNode(v) = &*l.limit else {
        return Err(CompileError::WrongKind {
            expected: "ValueNode (limit)",
            actual: node_kind(&l.limit),
        });
    };
    sql.push('?');
    params.push(v.value.clone());
    Ok(())
}

fn expect_column_name(col_node: &Node) -> Result<&str, CompileError> {
    let Node::ColumnNode(col) = col_node else {
        return Err(CompileError::WrongKind {
            expected: "ColumnNode",
            actual: node_kind(col_node),
        });
    };
    let Node::IdentifierNode(id) = &*col.column else {
        return Err(CompileError::WrongKind {
            expected: "IdentifierNode",
            actual: node_kind(&col.column),
        });
    };
    if !is_safe_ident(&id.name) {
        return Err(CompileError::UnsafeIdent(id.name.clone()));
    }
    Ok(&id.name)
}

fn push_quoted(sql: &mut String, name: &str) {
    sql.push('"');
    sql.push_str(name);
    sql.push('"');
}

fn node_kind(n: &Node) -> &'static str {
    match n {
        Node::SelectQueryNode(_) => "SelectQueryNode",
        Node::FromNode(_) => "FromNode",
        Node::TableNode(_) => "TableNode",
        Node::SchemableIdentifierNode(_) => "SchemableIdentifierNode",
        Node::IdentifierNode(_) => "IdentifierNode",
        Node::SelectionNode(_) => "SelectionNode",
        Node::SelectAllNode => "SelectAllNode",
        Node::ReferenceNode(_) => "ReferenceNode",
        Node::ColumnNode(_) => "ColumnNode",
        Node::WhereNode(_) => "WhereNode",
        Node::BinaryOperationNode(_) => "BinaryOperationNode",
        Node::AndNode(_) => "AndNode",
        Node::OrNode(_) => "OrNode",
        Node::OperatorNode(_) => "OperatorNode",
        Node::ValueNode(_) => "ValueNode",
        Node::OrderByNode(_) => "OrderByNode",
        Node::OrderByItemNode(_) => "OrderByItemNode",
        Node::RawNode(_) => "RawNode",
        Node::LimitNode(_) => "LimitNode",
    }
}

// ─── Executor ─────────────────────────────────────────────────────────────────

pub fn execute(
    ast_json: &str,
    conn: &rusqlite::Connection,
) -> Result<Vec<serde_json::Value>, ExecuteError> {
    let compiled = compile(ast_json)?;
    let params: Vec<rusqlite::types::Value> =
        compiled.params.iter().map(json_to_sqlite_value).collect();
    let param_refs: Vec<&dyn rusqlite::ToSql> =
        params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

    let mut stmt = conn.prepare(&compiled.sql)?;
    let col_names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(String::from)
        .collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        let mut obj = serde_json::Map::new();
        for (i, name) in col_names.iter().enumerate() {
            let v: rusqlite::types::Value = row.get(i)?;
            obj.insert(name.clone(), sqlite_value_to_json(&v));
        }
        Ok(serde_json::Value::Object(obj))
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn json_to_sqlite_value(v: &serde_json::Value) -> rusqlite::types::Value {
    use rusqlite::types::Value;
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Integer(if *b { 1 } else { 0 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Real(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::Text(s.clone()),
        _ => Value::Null,
    }
}

fn sqlite_value_to_json(v: &rusqlite::types::Value) -> serde_json::Value {
    use rusqlite::types::Value;
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Integer(i) => serde_json::Value::from(*i),
        Value::Real(f) => serde_json::Value::from(*f),
        Value::Text(s) => serde_json::Value::String(s.clone()),
        Value::Blob(b) => serde_json::Value::String(format!("<blob:{}>", b.len())),
    }
}
