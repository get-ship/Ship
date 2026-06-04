//! Read-only agent-def index — a queryable read model derived FROM the def
//! files (`.ship/agents/<id>.jsonc`), one-way: files → index, NEVER written
//! back. Mirrors `ship-collection`'s DataStore-backed read model, but the source
//! is FILES, not event payloads.
//!
//! The def file is truth. The index serves list/filter/lookup; it is rebuilt on
//! startup ([`AgentDefIndex::reindex_dir`]) and on a change notification
//! ([`AgentDefIndex::reindex_path`]: file changed → re-read → upsert; deleted →
//! drop). There is deliberately NO method that writes a def file — the index is
//! read-only over the file tier. Writes flow through the gated op (S5), not here.
//!
//! THE def reader is [`crate::agents`] (the FsBroker reader). This module reuses
//! its `.ship/agents` confinement (one cap definition, no parallel scan) and
//! adds the index source record + the DataStore-backed query surface.

use std::path::Path;
use std::sync::Arc;

use ship_substrate::ports::{
    ColumnKind, ColumnSpec, DataStore, FsBroker, PrimaryKey, Query, Row, TableSchema,
};

use crate::agents::{read_agent_def, read_agent_defs, AgentDefSource};
use crate::error::ConfigError;

/// The index table name (per-scope db). Columns mirror [`AgentDefRecord`].
const TABLE: &str = "agent_def_index";

/// One agent def as the read-only index needs it, derived FROM the def file.
///
/// Adds `tags` + `harness` (which the platform's flat `AgentProfile` view drops)
/// and the file-identity fields (`path`, `content_hash`) the index keys on. The
/// `content_hash` is the SHA-256 of the raw file bytes — the SAME value the
/// `agent.def.updated` notification carries, so a re-index can diff against the
/// ping.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentDefRecord {
    pub id: String,
    pub name: String,
    /// Source scope label (e.g. `project` / `user`) — set by the indexer, not
    /// the file. Lets the read model partition rows by where the def lives.
    pub scope: String,
    /// Rel file name under `.ship/agents/` (e.g. `code-reviewer.jsonc`).
    pub path: String,
    pub content_hash: String,
    pub role: Option<String>,
    /// Target harnesses (canonical `agent.harnesses`).
    pub harness: Vec<String>,
    pub tags: Vec<String>,
}

/// Filter for [`AgentDefIndex::query`]. Both predicates are optional and AND-ed.
#[derive(Debug, Clone, Default)]
pub struct AgentDefQuery {
    pub scope: Option<String>,
    pub name: Option<String>,
}

/// The read-only def index over a substrate [`DataStore`] (the per-scope db).
pub struct AgentDefIndex {
    store: Arc<dyn DataStore>,
}

impl AgentDefIndex {
    pub fn new(store: Arc<dyn DataStore>) -> Self {
        Self { store }
    }

    /// Ensure the index table exists. Idempotent (additive-evolution contract of
    /// the DataStore). Called on startup before the first re-index.
    pub async fn ensure_table(&self) -> Result<(), ConfigError> {
        let schema = TableSchema {
            name: TABLE.to_string(),
            columns: vec![
                col("agent_id", ColumnKind::Text, true),
                col("name", ColumnKind::Text, true),
                col("scope", ColumnKind::Text, true),
                col("path", ColumnKind::Text, true),
                col("content_hash", ColumnKind::Text, true),
                col("role", ColumnKind::Text, false),
                col("harness", ColumnKind::Text, false),
                col("tags", ColumnKind::Text, false),
            ],
        };
        self.store.ensure_table(&schema).await.map_err(Into::into)
    }

    /// Full rebuild for one scope: read every def file via the reader and upsert
    /// its row. Used on startup. Read-only over the files.
    pub async fn reindex_dir(
        &self,
        fs: &dyn FsBroker,
        ship_dir: &Path,
        scope: &str,
    ) -> Result<(), ConfigError> {
        for src in read_agent_defs(fs, ship_dir)? {
            self.upsert(scope, &src).await?;
        }
        Ok(())
    }

    /// Re-index one def after a change notification: file present → re-read +
    /// upsert; file absent → drop the row. `rel` is the def's rel file name
    /// (`<stem>.jsonc`); the row key is derived from `(scope, stem)` so a delete
    /// needs no file read.
    pub async fn reindex_path(
        &self,
        fs: &dyn FsBroker,
        ship_dir: &Path,
        scope: &str,
        rel: &str,
    ) -> Result<(), ConfigError> {
        let stem = file_stem(rel);
        match read_agent_def(fs, ship_dir, &stem)? {
            Some(src) => self.upsert(scope, &src).await,
            None => {
                self.store
                    .delete(TABLE, PrimaryKey(row_key(scope, &stem)))
                    .await?;
                Ok(())
            }
        }
    }

    /// Query the index, filtering by scope and/or name. Rows sorted by id.
    pub async fn query(&self, filter: AgentDefQuery) -> Result<Vec<AgentDefRecord>, ConfigError> {
        let mut q = Query::new(TABLE);
        if let Some(scope) = filter.scope {
            q = q.with_eq("scope", serde_json::Value::String(scope));
        }
        if let Some(name) = filter.name {
            q = q.with_eq("name", serde_json::Value::String(name));
        }
        q.limit = 500;
        let result = self.store.query(q).await?;
        Ok(result.rows.iter().map(row_to_record).collect())
    }

    /// Insert-or-replace the row for `(scope, def)`.
    async fn upsert(&self, scope: &str, src: &AgentDefSource) -> Result<(), ConfigError> {
        let row = Row::new(row_key(scope, &file_stem(&src.path)))
            .with("agent_id", json_str(&src.id))
            .with("name", json_str(&src.name))
            .with("scope", json_str(scope))
            .with("path", json_str(&src.path))
            .with("content_hash", json_str(&src.content_hash))
            .with("role", src.role.clone().map(json_str).unwrap_or(serde_json::Value::Null))
            .with("harness", json_str(&join_csv(&src.harness)))
            .with("tags", json_str(&join_csv(&src.tags)));
        self.store.insert(TABLE, row).await.map_err(Into::into)
    }
}

// ── small helpers ─────────────────────────────────────────────────────────────

fn col(name: &str, kind: ColumnKind, required: bool) -> ColumnSpec {
    ColumnSpec { name: name.to_string(), kind, required }
}

fn row_key(scope: &str, stem: &str) -> String {
    format!("{scope}:{stem}")
}

fn file_stem(rel: &str) -> String {
    rel.rsplit('/').next().unwrap_or(rel).trim_end_matches(".jsonc").to_string()
}

fn json_str(s: impl Into<String>) -> serde_json::Value {
    serde_json::Value::String(s.into())
}

/// Encode a string list as a newline-joined scalar for the relational store.
/// Empty list → empty string. Round-trips via [`split_csv`].
fn join_csv(items: &[String]) -> String {
    items.join("\n")
}

fn split_csv(s: &str) -> Vec<String> {
    if s.is_empty() {
        Vec::new()
    } else {
        s.split('\n').map(str::to_string).collect()
    }
}

fn row_to_record(row: &Row) -> AgentDefRecord {
    let get = |k: &str| row.fields.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role = row
        .fields
        .get("role")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    AgentDefRecord {
        id: get("agent_id"),
        name: get("name"),
        scope: get("scope"),
        path: get("path"),
        content_hash: get("content_hash"),
        role,
        harness: split_csv(&get("harness")),
        tags: split_csv(&get("tags")),
    }
}
