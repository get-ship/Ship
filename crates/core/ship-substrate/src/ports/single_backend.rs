//! `SingleBackendDataStore` — `DataStore` impl wrapping today's `DataService`.
//!
//! Wraps `crates/core/ship-substrate/src/data.rs` (`DataService`). Domain
//! services (`ship-collection`, `ship-artifacts`) hold a `SpaceContext` over
//! this store and create their own tables on demand.
//!
//! Every write injects a `space_id` column so the abstraction preserves
//! today's logical per-space partition even within a shared DB file.
//! Phase 2 will route to per-scope DB files; domain code sees no change.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::data::{DataService, SqlParam};

use super::data_store::{
    is_safe_ident, ColumnKind, CountQuery, DataStore, DataStoreError, FilterOp, Predicate,
    PrimaryKey, Query, QueryResult, Row, TableSchema,
};

pub struct SingleBackendDataStore {
    inner: Arc<Mutex<DataService>>,
    space_id: String,
}

impl SingleBackendDataStore {
    pub fn new(svc: DataService, space_id: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(svc)),
            space_id: space_id.into(),
        }
    }
}

#[async_trait]
impl DataStore for SingleBackendDataStore {
    async fn ensure_table(&self, schema: &TableSchema) -> Result<(), DataStoreError> {
        check_ident(&schema.name)?;
        for c in &schema.columns {
            check_ident(&c.name)?;
        }
        let mut svc = self.inner.lock().await;
        let cols: String = schema
            .columns
            .iter()
            .map(|c| {
                let sql_type = match c.kind {
                    ColumnKind::Text => "TEXT",
                    ColumnKind::Integer => "INTEGER",
                    ColumnKind::Real => "REAL",
                    ColumnKind::Boolean => "INTEGER",
                };
                let not_null = if c.required { " NOT NULL" } else { "" };
                format!(", `{}` {sql_type}{not_null}", c.name)
            })
            .collect();
        let ddl = format!(
            "CREATE TABLE IF NOT EXISTS `{name}` (\
             `id` TEXT PRIMARY KEY NOT NULL, \
             `space_id` TEXT NOT NULL{cols}\
             )",
            name = schema.name,
        );
        svc.ensure_table(&ddl).map_err(DataStoreError::from)?;

        // Converge an EXISTING table to the schema: add any column it lacks
        // (additive schema change). CREATE IF NOT EXISTS is a no-op on an
        // existing table, so without this the caller's connection never learns
        // a newly-declared column — and a `SELECT *` then trips on stale schema.
        // ADD COLUMN cannot carry NOT NULL without a default, so added columns
        // are nullable (the relational store's additive-evolution contract).
        let existing: std::collections::HashSet<String> = svc
            .query(&format!("PRAGMA table_info(`{}`)", schema.name), &[])
            .map_err(DataStoreError::from)?
            .into_iter()
            .filter_map(|r| r.get("name").and_then(|v| v.as_str().map(str::to_string)))
            .collect();
        for c in &schema.columns {
            if existing.contains(&c.name) {
                continue;
            }
            let sql_type = match c.kind {
                ColumnKind::Text => "TEXT",
                ColumnKind::Integer => "INTEGER",
                ColumnKind::Real => "REAL",
                ColumnKind::Boolean => "INTEGER",
            };
            let alter = format!("ALTER TABLE `{}` ADD COLUMN `{}` {sql_type}", schema.name, c.name);
            svc.execute(&alter, &[]).map_err(DataStoreError::from)?;
        }
        Ok(())
    }

    async fn insert(&self, table: &str, row: Row) -> Result<(), DataStoreError> {
        check_ident(table)?;
        for k in row.fields.keys() {
            check_ident(k)?;
        }
        let mut svc = self.inner.lock().await;
        let mut cols = vec!["id".to_string(), "space_id".to_string()];
        let mut placeholders = vec!["?".to_string(), "?".to_string()];
        let mut params: Vec<SqlParam> = vec![
            SqlParam::from(row.id.as_str()),
            SqlParam::from(self.space_id.as_str()),
        ];
        for (k, v) in &row.fields {
            cols.push(format!("`{k}`"));
            placeholders.push("?".to_string());
            params.push(json_to_param(v));
        }
        let sql = format!(
            "INSERT OR REPLACE INTO `{table}` ({}) VALUES ({})",
            cols.join(", "),
            placeholders.join(", ")
        );
        svc.execute(&sql, &params).map_err(DataStoreError::from)?;
        Ok(())
    }

    async fn update(&self, table: &str, row: Row) -> Result<(), DataStoreError> {
        check_ident(table)?;
        for k in row.fields.keys() {
            check_ident(k)?;
        }
        let mut svc = self.inner.lock().await;
        if row.fields.is_empty() {
            return Ok(());
        }
        let set_clauses: Vec<String> = row.fields.keys().map(|k| format!("`{k}` = ?")).collect();
        let mut params: Vec<SqlParam> = row.fields.values().map(json_to_param).collect();
        params.push(SqlParam::from(row.id.as_str()));
        params.push(SqlParam::from(self.space_id.as_str()));
        let sql = format!(
            "UPDATE `{table}` SET {} WHERE id = ? AND space_id = ?",
            set_clauses.join(", ")
        );
        svc.execute(&sql, &params).map_err(DataStoreError::from)?;
        Ok(())
    }

    async fn delete(&self, table: &str, key: PrimaryKey) -> Result<bool, DataStoreError> {
        check_ident(table)?;
        let mut svc = self.inner.lock().await;
        let sql = format!("DELETE FROM `{table}` WHERE id = ? AND space_id = ?");
        let params = [SqlParam::from(key.0.as_str()), SqlParam::from(self.space_id.as_str())];
        let affected = svc.execute(&sql, &params).map_err(DataStoreError::from)?;
        Ok(affected > 0)
    }

    async fn query(&self, q: Query) -> Result<QueryResult, DataStoreError> {
        check_ident(&q.table)?;
        if let Some(ref sort) = q.sort {
            check_ident(&sort.column)?;
        }
        let mut svc = self.inner.lock().await;
        let mut conditions = vec!["space_id = ?".to_string()];
        let mut params: Vec<SqlParam> = vec![SqlParam::from(self.space_id.as_str())];

        for pred in &q.filters {
            push_predicate(pred, &mut conditions, &mut params)?;
        }

        if let Some(ref after) = q.after {
            conditions.push("id > ?".to_string());
            params.push(SqlParam::from(after.as_str()));
        }

        let where_clause = conditions.join(" AND ");
        let order = match &q.sort {
            Some(sort) => {
                let dir = if sort.descending { "DESC" } else { "ASC" };
                format!("ORDER BY `{}` {dir}", sort.column)
            }
            None => "ORDER BY id ASC".to_string(),
        };
        let limit = q.limit.clamp(1, 500);
        let sql = format!(
            "SELECT * FROM `{table}` WHERE {where_clause} {order} LIMIT {limit}",
            table = q.table,
        );

        let data_rows = svc.query(&sql, &params).map_err(DataStoreError::from)?;
        let rows: Vec<Row> = data_rows.into_iter().map(data_row_to_port_row).collect();
        let next_cursor = rows.last().map(|r| r.id.clone());
        Ok(QueryResult { rows, next_cursor })
    }

    async fn count(&self, q: CountQuery) -> Result<usize, DataStoreError> {
        check_ident(&q.table)?;
        let mut svc = self.inner.lock().await;
        let mut conditions = vec!["space_id = ?".to_string()];
        let mut params: Vec<SqlParam> = vec![SqlParam::from(self.space_id.as_str())];

        for pred in &q.filters {
            push_predicate(pred, &mut conditions, &mut params)?;
        }

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT COUNT(*) as cnt FROM `{table}` WHERE {where_clause}",
            table = q.table,
        );
        let rows = svc.query(&sql, &params).map_err(DataStoreError::from)?;
        let cnt = rows
            .first()
            .and_then(|r| r.get("cnt"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as usize;
        Ok(cnt)
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Append one predicate's WHERE fragment + bound params. Column is validated;
/// values are always bound, never interpolated.
fn push_predicate(
    pred: &Predicate,
    conditions: &mut Vec<String>,
    params: &mut Vec<SqlParam>,
) -> Result<(), DataStoreError> {
    check_ident(&pred.column)?;
    let col = &pred.column;
    match pred.op {
        FilterOp::Eq | FilterOp::Gt | FilterOp::Gte | FilterOp::Lt | FilterOp::Lte => {
            let sql_op = match pred.op {
                FilterOp::Eq => "=",
                FilterOp::Gt => ">",
                FilterOp::Gte => ">=",
                FilterOp::Lt => "<",
                FilterOp::Lte => "<=",
                FilterOp::In => unreachable!(),
            };
            conditions.push(format!("`{col}` {sql_op} ?"));
            params.push(json_to_param(&pred.value));
        }
        FilterOp::In => {
            let items = pred.value.as_array().ok_or_else(|| {
                DataStoreError::Storage(format!("IN predicate on `{col}` requires an array value"))
            })?;
            if items.is_empty() {
                // `col IN ()` is invalid SQL and matches nothing — emit a
                // contradiction so the row set is empty.
                conditions.push("1 = 0".to_string());
                return Ok(());
            }
            let placeholders = vec!["?"; items.len()].join(", ");
            conditions.push(format!("`{col}` IN ({placeholders})"));
            for v in items {
                params.push(json_to_param(v));
            }
        }
    }
    Ok(())
}

fn check_ident(s: &str) -> Result<(), DataStoreError> {
    if is_safe_ident(s) {
        Ok(())
    } else {
        Err(DataStoreError::UnsafeIdent(s.to_string()))
    }
}

fn json_to_param(v: &serde_json::Value) -> SqlParam {
    match v {
        serde_json::Value::String(s) => SqlParam::from(s.as_str()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                SqlParam::Int(i)
            } else if let Some(f) = n.as_f64() {
                SqlParam::Float(f)
            } else {
                SqlParam::Null
            }
        }
        serde_json::Value::Bool(b) => SqlParam::Bool(*b),
        serde_json::Value::Null => SqlParam::Null,
        other => SqlParam::from(other.to_string().as_str()),
    }
}

fn data_row_to_port_row(data_row: crate::data::DataRow) -> Row {
    let obj = match data_row.0 {
        serde_json::Value::Object(map) => map,
        other => {
            let mut m = serde_json::Map::new();
            m.insert("value".to_string(), other);
            m
        }
    };
    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let fields: indexmap::IndexMap<String, serde_json::Value> = obj
        .into_iter()
        .filter(|(k, _)| k != "id" && k != "space_id")
        .collect();
    Row { id, fields }
}
