//! Collection service — core business logic for all 10 RPCs.
//!
//! All storage goes through `SpaceContext.data()` (DataStore port).
//! All events go through `SpaceContext.kernel()` (KernelLog port).
//! No sqlite/rusqlite/sqlx types here.

use std::sync::Arc;

use ship_substrate::ports::{Predicate, PrimaryKey, Query, SpaceContext};
use ship_types::envelope::EventEnvelope;
use ship_types::payload::{
    CollectionCreated, CollectionItemCreated, CollectionItemDeleted, CollectionItemUpdated,
    CollectionSchemaChanged,
};

use crate::error::CollectionError;
use crate::helpers::{build_row, emit_ctx, new_ulid, now_rfc3339, row_to_item};
use crate::projection::{
    SchemaRegistry, count_items, ensure_collection_table, ensure_registry_table,
    persist_schema, update_schema_persistent,
};
use crate::schema::{parse_schema_doc, validate_item};
use crate::types::{ItemRecord, SchemaDoc};

/// Core service — stateless; all state lives in `SpaceContext` + `SchemaRegistry`.
pub struct CollectionService {
    pub ctx: Arc<SpaceContext>,
    pub schemas: Arc<SchemaRegistry>,
}

impl CollectionService {
    /// List all schemas including system ones.
    pub fn list(&self) -> Vec<SchemaDoc> {
        self.schemas.list()
    }

    /// Get a single schema with its item count.
    pub async fn get(&self, id: &str) -> Result<(SchemaDoc, usize), CollectionError> {
        let schema = self.schemas.get(id).ok_or_else(|| {
            CollectionError::NotFound(format!("collection '{id}' not found"))
        })?;
        let count = count_items(self.ctx.data(), id).await.unwrap_or(0);
        Ok((schema, count))
    }

    /// Get a single item by id.
    pub async fn get_item(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<ItemRecord, CollectionError> {
        let _schema = self.schemas.get(collection).ok_or_else(|| {
            CollectionError::NotFound(format!("collection '{collection}' not found"))
        })?;
        let q = Query {
            table: format!("collection_{collection}"),
            filters: vec![Predicate::eq("id", serde_json::Value::String(id.into()))],
            limit: 1,
            ..Default::default()
        };
        let result = self.ctx.data().query(q).await?;
        let row = result.rows.into_iter().next().ok_or_else(|| {
            CollectionError::NotFound(format!("item '{id}' not found in '{collection}'"))
        })?;
        Ok(row_to_item(row, collection))
    }

    /// Query items via DataStore::Query (from translated AST or direct).
    pub async fn query_items_by_query(
        &self,
        q: Query,
    ) -> Result<(Vec<ItemRecord>, Option<String>), CollectionError> {
        let collection = q.table.strip_prefix("collection_").unwrap_or(&q.table).to_string();
        let result = self.ctx.data().query(q).await?;
        let next_cursor = result.next_cursor.clone();
        let items = result.rows.into_iter().map(|r| row_to_item(r, &collection)).collect();
        Ok((items, next_cursor))
    }

    /// Translate inline AST and run as a query.
    pub async fn query_items_ast(
        &self,
        ast_bytes: &[u8],
        cursor: Option<String>,
        limit: u32,
    ) -> Result<(Vec<ItemRecord>, Option<String>), CollectionError> {
        let mut q = ship_substrate::query::translate_ast(ast_bytes, cursor, limit)
            .map_err(|e| CollectionError::UnsupportedQuery(e.to_string()))?;
        // The translator returns the logical table name; map it to the physical
        // collection table here (the executor no longer applies the prefix).
        q.table = format!("collection_{}", q.table);
        self.query_items_by_query(q).await
    }

    /// Create a new item, validate against schema, emit event.
    pub async fn create_item(
        &self,
        collection: &str,
        data_json: &[u8],
    ) -> Result<ItemRecord, CollectionError> {
        let schema = self.schemas.get(collection).ok_or_else(|| {
            CollectionError::NotFound(format!("collection '{collection}' not found"))
        })?;
        let data: serde_json::Value = serde_json::from_slice(data_json)
            .map_err(|e| CollectionError::Validation(format!("invalid data JSON: {e}")))?;
        validate_item(&schema, &data)?;

        let id = new_ulid();
        let now = now_rfc3339();
        let row = build_row(&id, &schema, data.as_object().unwrap(), &now, &now);
        self.ctx.data().insert(&format!("collection_{collection}"), row).await?;

        let envelope = EventEnvelope::new(
            "collection.item_created",
            &id,
            &CollectionItemCreated {
                collection_id: collection.to_string(),
                item_id: id.clone(),
                item_type: None,
            },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx()).await?;

        Ok(ItemRecord {
            id,
            collection: collection.to_string(),
            data: data.as_object().cloned().unwrap_or_default(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Update an item, validate against schema, emit event.
    pub async fn update_item(
        &self,
        collection: &str,
        id: &str,
        data_json: &[u8],
    ) -> Result<ItemRecord, CollectionError> {
        let schema = self.schemas.get(collection).ok_or_else(|| {
            CollectionError::NotFound(format!("collection '{collection}' not found"))
        })?;
        let data: serde_json::Value = serde_json::from_slice(data_json)
            .map_err(|e| CollectionError::Validation(format!("invalid data JSON: {e}")))?;
        validate_item(&schema, &data)?;

        let now = now_rfc3339();
        let original = self.get_item(collection, id).await?;
        let row = build_row(id, &schema, data.as_object().unwrap(), &original.created_at, &now);
        self.ctx.data().update(&format!("collection_{collection}"), row).await?;

        let envelope = EventEnvelope::new(
            "collection.item_updated",
            id,
            &CollectionItemUpdated {
                collection_id: collection.to_string(),
                item_id: id.to_string(),
            },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx()).await?;

        Ok(ItemRecord {
            id: id.to_string(),
            collection: collection.to_string(),
            data: data.as_object().cloned().unwrap_or_default(),
            created_at: original.created_at,
            updated_at: now,
        })
    }

    /// Delete an item, emit event.
    pub async fn delete_item(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<(), CollectionError> {
        self.schemas.get(collection).ok_or_else(|| {
            CollectionError::NotFound(format!("collection '{collection}' not found"))
        })?;
        let deleted = self
            .ctx
            .data()
            .delete(&format!("collection_{collection}"), PrimaryKey(id.to_string()))
            .await?;
        if !deleted {
            return Err(CollectionError::NotFound(format!("item '{id}' not found")));
        }
        let envelope = EventEnvelope::new(
            "collection.item_deleted",
            id,
            &CollectionItemDeleted {
                collection_id: collection.to_string(),
                item_id: id.to_string(),
            },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx()).await?;
        Ok(())
    }

    /// Create a new schema, ensure table, emit event.
    pub async fn create_schema(
        &self,
        id: &str,
        schema_json: &[u8],
    ) -> Result<SchemaDoc, CollectionError> {
        use crate::types::RESERVED_IDS;
        if RESERVED_IDS.contains(&id) {
            return Err(CollectionError::ReservedId(format!(
                "id '{id}' is reserved and cannot be used for user schemas"
            )));
        }
        if self.schemas.get(id).is_some() {
            return Err(CollectionError::Validation(format!(
                "collection '{id}' already exists"
            )));
        }
        let mut doc = parse_schema_doc(schema_json)?;
        doc.id = id.to_string();

        ensure_registry_table(self.ctx.data()).await?;
        ensure_collection_table(self.ctx.data(), &doc).await?;
        persist_schema(self.ctx.data(), &doc).await?;

        let schema_str = serde_json::to_string(&doc)
            .map_err(|e| CollectionError::Internal(e.to_string()))?;
        let envelope = EventEnvelope::new(
            "collection.created",
            id,
            &CollectionCreated { collection_id: id.to_string(), schema_json: schema_str },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx()).await?;

        self.schemas.insert(doc.clone());
        Ok(doc)
    }

    /// Update a schema, update table, emit event.
    pub async fn update_schema(
        &self,
        id: &str,
        schema_json: &[u8],
    ) -> Result<SchemaDoc, CollectionError> {
        self.schemas.get(id).ok_or_else(|| {
            CollectionError::NotFound(format!("collection '{id}' not found"))
        })?;
        let mut doc = parse_schema_doc(schema_json)?;
        doc.id = id.to_string();

        ensure_collection_table(self.ctx.data(), &doc).await?;
        update_schema_persistent(self.ctx.data(), &doc).await?;

        let schema_str = serde_json::to_string(&doc)
            .map_err(|e| CollectionError::Internal(e.to_string()))?;
        let envelope = EventEnvelope::new(
            "collection.schema.changed",
            id,
            &CollectionSchemaChanged { collection_id: id.to_string(), schema_json: schema_str },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx()).await?;

        self.schemas.update(doc.clone());
        Ok(doc)
    }
}

