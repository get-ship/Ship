//! DataStore table management and row queries for the artifacts metadata table.

use std::collections::BTreeMap;

use ship_substrate::ports::{
    ColumnKind, ColumnSpec, DataStore, DataStoreError, Query, Row, TableSchema,
};

use crate::error::ArtifactError;
use crate::types::{ArtifactRecord, ArtifactScope};

pub const TABLE: &str = "collection_artifacts";

// Column name for the artifact's project/workspace space id. Cannot use
// "space_id" because SingleBackendDataStore auto-adds that as a partition key.
const COL_SCOPE_SPACE: &str = "scope_space";

/// Ensure the artifacts metadata table exists with all required columns.
pub async fn ensure_artifacts_table(store: &dyn DataStore) -> Result<(), DataStoreError> {
    let schema = TableSchema {
        name: TABLE.to_string(),
        columns: vec![
            ColumnSpec { name: "created_at".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "name".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "path".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "mime".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "size".into(), kind: ColumnKind::Real, required: true },
            ColumnSpec { name: "scope".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: COL_SCOPE_SPACE.into(), kind: ColumnKind::Text, required: false },
            ColumnSpec { name: "run_id".into(), kind: ColumnKind::Text, required: false },
            ColumnSpec { name: "user_id".into(), kind: ColumnKind::Text, required: false },
            ColumnSpec { name: "content_hash".into(), kind: ColumnKind::Text, required: false },
            ColumnSpec { name: "expires_at".into(), kind: ColumnKind::Text, required: false },
            ColumnSpec { name: "owner_actor".into(), kind: ColumnKind::Text, required: false },
            ColumnSpec { name: "collection_id".into(), kind: ColumnKind::Text, required: false },
            ColumnSpec { name: "item_id".into(), kind: ColumnKind::Text, required: false },
        ],
    };
    store.ensure_table(&schema).await
}

/// Build an `ArtifactRecord` from a DataStore `Row`. Fails if required fields
/// are missing — schema validation prevents insertion of malformed rows.
pub fn row_to_record(row: Row) -> Result<ArtifactRecord, ArtifactError> {
    let req = |k: &str| -> Result<String, ArtifactError> {
        row.fields
            .get(k)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| ArtifactError::Internal(format!("artifact row {} missing '{k}'", row.id)))
    };
    let opt = |k: &str| -> Option<String> {
        row.fields.get(k).and_then(|v| if v.is_null() { None } else { v.as_str().map(str::to_string) })
    };
    let size = row.fields
        .get("size")
        .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
        .ok_or_else(|| ArtifactError::Internal(format!("artifact row {} missing 'size'", row.id)))?;
    Ok(ArtifactRecord {
        id: row.id.clone(),
        name: req("name")?,
        mime: req("mime")?,
        size,
        path: req("path")?,
        scope: req("scope")?,
        space_id: opt(COL_SCOPE_SPACE),
        run_id: opt("run_id"),
        user_id: opt("user_id"),
        content_hash: opt("content_hash"),
        expires_at: opt("expires_at"),
        owner_actor: opt("owner_actor"),
        collection_id: opt("collection_id"),
        item_id: opt("item_id"),
        created_at: req("created_at")?,
    })
}

/// Build a `Row` from an `ArtifactRecord` for insert/update.
pub fn record_to_row(record: &ArtifactRecord) -> Row {
    let mut row = Row::new(record.id.clone())
        .with("created_at", sv(record.created_at.clone()))
        .with("name", sv(record.name.clone()))
        .with("path", sv(record.path.clone()))
        .with("mime", sv(record.mime.clone()))
        .with("size", serde_json::Value::Number(
            serde_json::Number::from(record.size),
        ))
        .with("scope", sv(record.scope.clone()));

    if let Some(ref v) = record.space_id {
        row = row.with(COL_SCOPE_SPACE, sv(v.clone()));
    }
    macro_rules! opt_field {
        ($field:ident) => {
            if let Some(ref v) = record.$field {
                row = row.with(stringify!($field), sv(v.clone()));
            }
        };
    }
    opt_field!(run_id);
    opt_field!(user_id);
    opt_field!(content_hash);
    opt_field!(expires_at);
    opt_field!(owner_actor);
    opt_field!(collection_id);
    opt_field!(item_id);
    row
}

fn sv(s: String) -> serde_json::Value {
    serde_json::Value::String(s)
}

/// Reconstruct the scope from a metadata record.
pub fn record_to_scope(record: &ArtifactRecord) -> Result<ArtifactScope, ArtifactError> {
    match record.scope.as_str() {
        "run" => {
            let space_id = record.space_id.clone()
                .ok_or_else(|| ArtifactError::Internal("run-scope artifact missing space_id".into()))?;
            let run_id = record.run_id.clone()
                .ok_or_else(|| ArtifactError::Internal("run-scope artifact missing run_id".into()))?;
            Ok(ArtifactScope::Run { space_id, run_id })
        }
        "project" => {
            let space_id = record.space_id.clone()
                .ok_or_else(|| ArtifactError::Internal("project-scope artifact missing space_id".into()))?;
            Ok(ArtifactScope::Project { space_id })
        }
        "user" => {
            let user_id = record.user_id.clone()
                .ok_or_else(|| ArtifactError::Internal("user-scope artifact missing user_id".into()))?;
            Ok(ArtifactScope::User { user_id })
        }
        other => Err(ArtifactError::Internal(format!("invalid scope `{other}`"))),
    }
}

/// Get a single artifact record by id, or None if not found.
pub async fn get_by_id(store: &dyn DataStore, id: &str) -> Result<Option<ArtifactRecord>, ArtifactError> {
    let mut filter = BTreeMap::new();
    filter.insert("id".into(), serde_json::Value::String(id.to_string()));
    let q = Query { table: TABLE.to_string(), filter, limit: 1, ..Default::default() };
    let result = store.query(q).await?;
    match result.rows.into_iter().next() {
        Some(row) => Ok(Some(row_to_record(row)?)),
        None => Ok(None),
    }
}

/// Query artifacts with an optional equality filter map.
pub async fn query_artifacts(
    store: &dyn DataStore,
    filter: BTreeMap<String, serde_json::Value>,
    limit: u32,
    after: Option<String>,
) -> Result<Vec<ArtifactRecord>, ArtifactError> {
    let q = Query { table: TABLE.to_string(), filter, limit, after, ..Default::default() };
    let result = store.query(q).await?;
    result.rows.into_iter().map(row_to_record).collect()
}

/// Count artifacts matching a filter (used for dedup check).
pub async fn count_by_filter(
    store: &dyn DataStore,
    filter: BTreeMap<String, serde_json::Value>,
) -> Result<usize, ArtifactError> {
    use ship_substrate::ports::CountQuery;
    let q = CountQuery { table: TABLE.to_string(), filter };
    Ok(store.count(q).await?)
}

/// Find an artifact by content_hash within the same scope (for dedup).
pub async fn find_by_hash(
    store: &dyn DataStore,
    scope: &ArtifactScope,
    content_hash: &str,
) -> Result<Option<String>, ArtifactError> {
    let mut filter = BTreeMap::new();
    filter.insert("scope".into(), sv(scope.kind_str().to_string()));
    filter.insert("content_hash".into(), sv(content_hash.to_string()));
    scope_filter_fields(scope, &mut filter);
    let q = Query { table: TABLE.to_string(), filter, limit: 1, ..Default::default() };
    let result = store.query(q).await?;
    Ok(result.rows.into_iter().next()
        .and_then(|r| r.fields.get("path").and_then(|v| v.as_str().map(str::to_string))))
}

/// Count rows sharing a (scope, rel_path) — guards blob deletion under dedup.
pub async fn count_path_refs(
    store: &dyn DataStore,
    scope: &ArtifactScope,
    rel_path: &str,
) -> Result<usize, ArtifactError> {
    let mut filter = BTreeMap::new();
    filter.insert("scope".into(), sv(scope.kind_str().to_string()));
    filter.insert("path".into(), sv(rel_path.to_string()));
    scope_filter_fields(scope, &mut filter);
    count_by_filter(store, filter).await
}

fn scope_filter_fields(scope: &ArtifactScope, filter: &mut BTreeMap<String, serde_json::Value>) {
    match scope {
        ArtifactScope::Run { space_id, run_id } => {
            filter.insert(COL_SCOPE_SPACE.into(), sv(space_id.clone()));
            filter.insert("run_id".into(), sv(run_id.clone()));
        }
        ArtifactScope::Project { space_id } => {
            filter.insert(COL_SCOPE_SPACE.into(), sv(space_id.clone()));
        }
        ArtifactScope::User { user_id } => {
            filter.insert("user_id".into(), sv(user_id.clone()));
        }
    }
}
