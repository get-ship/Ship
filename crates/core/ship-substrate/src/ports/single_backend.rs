//! `SingleBackendDataStore` — `DataStore` impl wrapping today's `DataService`.
//!
//! Wraps `crates/core/ship-substrate/src/data.rs` (`DataService`) and
//! `crates/core/ship-platform/src/services/data.rs` (`ensure_data_service`).
//!
//! Every write injects a `space_id` column so the abstraction preserves
//! today's logical per-space partition even within a shared DB file.
//! Phase 2 will route to per-scope DB files; domain code sees no change.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::data::{DataService, SqlParam};

use super::data_store::{
    ColumnKind, CountQuery, DataStore, DataStoreError, PrimaryKey, Query, QueryResult, Row,
    TableSchema,
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
        svc.ensure_table(&ddl).map_err(DataStoreError::from)
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
        for k in q.filter.keys() {
            check_ident(k)?;
        }
        if let Some(ref col) = q.sort_asc {
            check_ident(col)?;
        }
        let mut svc = self.inner.lock().await;
        let mut conditions = vec!["space_id = ?".to_string()];
        let mut params: Vec<SqlParam> = vec![SqlParam::from(self.space_id.as_str())];

        for (k, v) in &q.filter {
            conditions.push(format!("`{k}` = ?"));
            params.push(json_to_param(v));
        }

        if let Some(ref after) = q.after {
            conditions.push("id > ?".to_string());
            params.push(SqlParam::from(after.as_str()));
        }

        let where_clause = conditions.join(" AND ");
        let order = match &q.sort_asc {
            Some(col) => format!("ORDER BY `{col}` ASC"),
            None => "ORDER BY id ASC".to_string(),
        };
        let limit = q.limit.min(500).max(1);
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
        for k in q.filter.keys() {
            check_ident(k)?;
        }
        let mut svc = self.inner.lock().await;
        let mut conditions = vec!["space_id = ?".to_string()];
        let mut params: Vec<SqlParam> = vec![SqlParam::from(self.space_id.as_str())];

        for (k, v) in &q.filter {
            conditions.push(format!("`{k}` = ?"));
            params.push(json_to_param(v));
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

fn is_safe_ident(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
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
