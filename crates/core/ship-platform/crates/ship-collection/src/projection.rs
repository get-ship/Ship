//! Schema registry projection — in-memory map seeded from built-ins and
//! updated on CreateSchema/UpdateSchema. Backed by a DataStore table
//! `_schema_registry` for persistence across restarts.
//!
//! The projection is the authoritative read surface; event emission is the
//! authoritative write surface. In this first implementation writes are
//! eagerly applied to the projection immediately after event emission.

use std::collections::HashMap;
use std::sync::Mutex;

use ship_substrate::{
    ColumnKind, ColumnSpec, CountQuery, DataStore, DataStoreError, Row, TableSchema,
};

use crate::builtin::all_builtin_schemas;
use crate::error::CollectionError;
use crate::types::SchemaDoc;

const REGISTRY_TABLE: &str = "_schema_registry";

/// In-memory schema registry. Seeded with built-ins; updated on Create/Update.
pub struct SchemaRegistry {
    inner: Mutex<HashMap<String, SchemaDoc>>,
}

impl SchemaRegistry {
    /// Create a new registry seeded with the built-in schemas.
    pub fn new() -> Self {
        let mut map = HashMap::new();
        for schema in all_builtin_schemas() {
            map.insert(schema.id.clone(), schema);
        }
        Self {
            inner: Mutex::new(map),
        }
    }

    pub fn list(&self) -> Vec<SchemaDoc> {
        self.inner.lock().unwrap().values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<SchemaDoc> {
        self.inner.lock().unwrap().get(id).cloned()
    }

    pub fn insert(&self, doc: SchemaDoc) {
        self.inner.lock().unwrap().insert(doc.id.clone(), doc);
    }

    pub fn update(&self, doc: SchemaDoc) {
        self.inner.lock().unwrap().insert(doc.id.clone(), doc);
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Ensure the `_schema_registry` table exists in the DataStore.
pub async fn ensure_registry_table(store: &dyn DataStore) -> Result<(), DataStoreError> {
    let schema = TableSchema {
        name: REGISTRY_TABLE.to_string(),
        columns: vec![ColumnSpec {
            name: "schema_json".into(),
            kind: ColumnKind::Text,
            required: true,
        }],
    };
    store.ensure_table(&schema).await
}

/// Persist a schema doc to the DataStore registry table.
pub async fn persist_schema(store: &dyn DataStore, doc: &SchemaDoc) -> Result<(), CollectionError> {
    let json = serde_json::to_string(doc).map_err(|e| CollectionError::Internal(e.to_string()))?;
    let row = Row::new(doc.id.clone()).with("schema_json", serde_json::Value::String(json));
    store
        .insert(REGISTRY_TABLE, row)
        .await
        .map_err(CollectionError::from)
}

/// Update an existing schema doc in the DataStore registry table.
pub async fn update_schema_persistent(
    store: &dyn DataStore,
    doc: &SchemaDoc,
) -> Result<(), CollectionError> {
    let json = serde_json::to_string(doc).map_err(|e| CollectionError::Internal(e.to_string()))?;
    let row = Row::new(doc.id.clone()).with("schema_json", serde_json::Value::String(json));
    store
        .update(REGISTRY_TABLE, row)
        .await
        .map_err(CollectionError::from)
}

/// Hydrate `registry` from the durable `_schema_registry` rows in `store`,
/// inserting every schema the API or an Update path persisted. The
/// in-memory map already carries built-ins; rows with a reserved id
/// (built-ins, system tables like `view_snapshots`) are skipped — a
/// late row shadowing a built-in is rejected here rather than overwriting
/// the code-defined shape.
///
/// Per-space: `_schema_registry` lives in this scope's db only —
/// `Storage Blueprint (ship-substrate-db.md)` "Fold `_schema_registry`
/// per-space". Built-ins stay code-defined (`all_builtin_schemas`); the
/// API-persisted rows layer ON TOP, file-overlay rows layer ON TOP of
/// those via the caller (`ship_store::collection_schemas::hydrate_registry`).
pub async fn hydrate_from_store(
    registry: &SchemaRegistry,
    store: &dyn DataStore,
) -> Result<(), CollectionError> {
    use crate::types::RESERVED_IDS;
    let q = ship_substrate::Query::new(REGISTRY_TABLE);
    let result = store.query(q).await.map_err(CollectionError::from)?;
    for row in result.rows {
        let id = row.id.clone();
        if RESERVED_IDS.contains(&id.as_str()) {
            continue;
        }
        let Some(json) = row.fields.get("schema_json") else {
            continue;
        };
        let Ok(doc) = serde_json::from_value::<SchemaDoc>(json.clone()) else {
            tracing::warn!(id = %id, "hydrate_from_store: skipping malformed schema_json row");
            continue;
        };
        registry.insert(doc);
    }
    Ok(())
}

/// Ensure a collection's item table exists with the correct columns.
pub async fn ensure_collection_table(
    store: &dyn DataStore,
    doc: &SchemaDoc,
) -> Result<(), CollectionError> {
    let mut columns = vec![
        ColumnSpec {
            name: "created_at".into(),
            kind: ColumnKind::Text,
            required: true,
        },
        ColumnSpec {
            name: "updated_at".into(),
            kind: ColumnKind::Text,
            required: true,
        },
    ];
    for (name, field) in &doc.fields {
        columns.push(ColumnSpec {
            name: name.clone(),
            kind: field.kind.to_column_kind(),
            required: field.required,
        });
    }
    let table = TableSchema {
        name: format!("collection_{}", doc.id),
        columns,
    };
    store
        .ensure_table(&table)
        .await
        .map_err(CollectionError::from)
}

/// Count items in a collection table.
pub async fn count_items(
    store: &dyn DataStore,
    collection_id: &str,
) -> Result<usize, CollectionError> {
    store
        .count(CountQuery::new(format!("collection_{collection_id}")))
        .await
        .map_err(CollectionError::from)
}
