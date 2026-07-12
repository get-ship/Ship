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
//!
//! [`provenance`] layers a Phase-3 roll-up ingest on top (which extension
//! supplied a row, and whether it lost composition to another one).

use std::path::Path;
use std::sync::Arc;

use ship_substrate::ports::{
    ColumnKind, ColumnSpec, DataStore, FsBroker, LocalFsBroker, PrimaryKey, Query, Row,
    TableSchema,
};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::AgentDefUpdated;

use crate::agents::{read_agent_def, read_agent_defs, AgentDefSource};
use crate::error::ConfigError;

mod provenance;
pub use provenance::DefProvenance;
use provenance::{is_shadow_row, row_doc_with_provenance, row_opt_str};

/// Boot rebuild for one scope: read every def file via the reader and upsert its
/// index row. One-way (files → index), read-only over the files. Called at
/// daemon boot for each active space's `.ship` dir.
pub async fn build_def_index(
    index: &AgentDefIndex,
    fs: &dyn FsBroker,
    ship_dir: &Path,
    scope: &str,
) -> anyhow::Result<()> {
    index.reindex_dir(fs, ship_dir, scope).await.map_err(Into::into)
}

/// Apply one `agent.def.updated` notification to the index. Non-matching event
/// types are ignored (the spawned actor relies on this filter). The path is read
/// from the payload; the index then RE-READS the file (`reindex_path`): present →
/// upsert, absent → drop. The payload `content_hash` is advisory — the row's hash
/// is computed from the re-read, so a stale/lying payload can't poison the index.
pub async fn apply_def_updated(
    index: &AgentDefIndex,
    fs: &dyn FsBroker,
    ship_dir: &Path,
    scope: &str,
    event: &EventEnvelope,
) -> anyhow::Result<()> {
    if event.event_type != event_types::AGENT_DEF_UPDATED {
        return Ok(());
    }
    let payload: AgentDefUpdated = serde_json::from_str(&event.payload_json)?;
    index
        .reindex_path(fs, ship_dir, scope, &payload.path)
        .await
        .map_err(Into::into)
}

/// Spawn the long-lived index consumer: read `agent.def.updated` events off the
/// kernel-log broadcast and apply each to the index. Returns when the channel
/// closes. Errors are logged, never panic (one bad event must not kill the
/// consumer); a `Lagged` gap is logged and skipped — the next boot rebuild
/// reconciles any window missed under burst.
pub fn spawn_def_index_consumer(
    index: Arc<AgentDefIndex>,
    mut rx: broadcast::Receiver<EventEnvelope>,
    ship_dir: std::path::PathBuf,
    scope: String,
) -> JoinHandle<()> {
    let fs = LocalFsBroker::new();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if event.event_type != event_types::AGENT_DEF_UPDATED {
                        continue;
                    }
                    if let Err(e) = apply_def_updated(&index, &fs, &ship_dir, &scope, &event).await {
                        tracing::warn!(
                            "agent-def-index: apply error for {}: {e}",
                            event.entity_id
                        );
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("agent-def-index: lagged by {n} events, continuing");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

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
    /// The extension that supplied this row, as [`stack_model::compose_space`]
    /// emits it (JSON `ExtSource`, verbatim — never a hand-rolled label).
    /// `None` for a row from the plain per-scope `.jsonc`/`.ts` reindex.
    pub ext_source: Option<String>,
    /// The supplying extension's content hash. `None` alongside `ext_source`.
    pub tier_content_hash: Option<String>,
    /// `Some(<winning ExtSource JSON>)` when THIS row's def lost composition
    /// (a shadow row — see [`AgentDefIndex::upsert_composed`]); `None` for
    /// the active row.
    pub shadowed_by: Option<String>,
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
                col("doc_json", ColumnKind::Text, false),
                col("ext_source", ColumnKind::Text, false),
                col("tier_content_hash", ColumnKind::Text, false),
                col("shadowed_by", ColumnKind::Text, false),
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
            let key = row_key(scope, &file_stem(&src.path));
            self.upsert(key, scope, &src, None).await?;
        }
        Ok(())
    }

    /// Upsert one agent def resolved from a `.ts` config (a compiler
    /// [`stack_model::AgentProfile`]), through the SAME row projection the
    /// `.jsonc` reader uses. `provenance` is `Some` only for a Phase-3
    /// roll-up winner ([`Self::upsert_composed`]); plain ingest passes `None`
    /// (no behavior change there).
    pub async fn upsert_profile(
        &self,
        scope: &str,
        profile: &stack_model::AgentProfile,
        provenance: Option<&DefProvenance>,
    ) -> Result<(), ConfigError> {
        let src = crate::agents_ts::source_from_profile(profile);
        let key = row_key(scope, &file_stem(&src.path));
        self.upsert(key, scope, &src, provenance).await
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
            Some(src) => {
                let key = row_key(scope, &stem);
                self.upsert(key, scope, &src, None).await
            }
            None => {
                self.store
                    .delete(TABLE, PrimaryKey(row_key(scope, &stem)))
                    .await?;
                Ok(())
            }
        }
    }

    /// Query the index, filtering by scope and/or name. Rows sorted by id.
    /// This is the provenance AUDIT surface — unlike [`Self::list_docs`] /
    /// [`Self::get_doc`], it also returns shadow rows (`shadowed_by: Some(_)`).
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

    /// List every ACTIVE def in `scope` as `(agent_id, full doc value)` — the
    /// live READ surface the daemon's list view projects from, served with NO
    /// file read. A def SHADOWED by a roll-up ([`Self::upsert_composed`]) is
    /// excluded — not a real, selectable def. Each doc gains a `_provenance`
    /// object for audit without a second read.
    pub async fn list_docs(
        &self,
        scope: &str,
    ) -> Result<Vec<(String, serde_json::Value)>, ConfigError> {
        let mut q = Query::new(TABLE).with_eq("scope", json_str(scope));
        q.limit = 500;
        let result = self.store.query(q).await?;
        Ok(result
            .rows
            .iter()
            .filter(|row| !is_shadow_row(row))
            .filter_map(row_doc_with_provenance)
            .collect())
    }

    /// Get one def's full ACTIVE doc value by `id` within `scope`. `Ok(None)`
    /// when not indexed (or only present as a shadow row). `_provenance` is
    /// merged in, as in [`Self::list_docs`].
    pub async fn get_doc(
        &self,
        id: &str,
        scope: &str,
    ) -> Result<Option<serde_json::Value>, ConfigError> {
        let mut q = Query::new(TABLE)
            .with_eq("agent_id", json_str(id))
            .with_eq("scope", json_str(scope));
        // Room for the active row PLUS any shadow rows a roll-up recorded for
        // the same id — `find` below picks the active one; `limit = 1` would
        // risk grabbing a shadow row nondeterministically.
        q.limit = 50;
        let result = self.store.query(q).await?;
        Ok(result
            .rows
            .iter()
            .find(|row| !is_shadow_row(row))
            .and_then(row_doc_with_provenance)
            .map(|(_, doc)| doc))
    }

    /// Insert-or-replace the row at `key`, tagged `scope`, stamping
    /// `provenance` when the caller has any ([`Self::upsert_composed`]).
    async fn upsert(
        &self,
        key: String,
        scope: &str,
        src: &AgentDefSource,
        provenance: Option<&DefProvenance>,
    ) -> Result<(), ConfigError> {
        let row = Row::new(key)
            .with("agent_id", json_str(&src.id))
            .with("name", json_str(&src.name))
            .with("scope", json_str(scope))
            .with("path", json_str(&src.path))
            .with("content_hash", json_str(&src.content_hash))
            .with("role", src.role.clone().map(json_str).unwrap_or(serde_json::Value::Null))
            .with("harness", json_str(&join_csv(&src.harness)))
            .with("tags", json_str(&join_csv(&src.tags)))
            .with("doc_json", json_str(serde_json::to_string(&src.doc).unwrap_or_default()))
            .with(
                "ext_source",
                provenance.map(|p| json_str(p.ext_source.clone())).unwrap_or(serde_json::Value::Null),
            )
            .with(
                "tier_content_hash",
                provenance.map(|p| json_str(p.tier_hash.clone())).unwrap_or(serde_json::Value::Null),
            )
            .with(
                "shadowed_by",
                provenance
                    .and_then(|p| p.shadowed_by.clone())
                    .map(json_str)
                    .unwrap_or(serde_json::Value::Null),
            );
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
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.strip_suffix(".jsonc")
        .or_else(|| name.strip_suffix(".ts"))
        .unwrap_or(name)
        .to_string()
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
    AgentDefRecord {
        id: get("agent_id"),
        name: get("name"),
        scope: get("scope"),
        path: get("path"),
        content_hash: get("content_hash"),
        role: row_opt_str(row, "role"),
        harness: split_csv(&get("harness")),
        tags: split_csv(&get("tags")),
        ext_source: row_opt_str(row, "ext_source"),
        tier_content_hash: row_opt_str(row, "tier_content_hash"),
        shadowed_by: row_opt_str(row, "shadowed_by"),
    }
}
