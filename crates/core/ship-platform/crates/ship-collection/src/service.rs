//! Collection service — core business logic for all 10 RPCs.
//!
//! All storage goes through `SpaceContext.data()` (DataStore port).
//! All events go through `SpaceContext.kernel()` (KernelLog port).
//! No sqlite/rusqlite/sqlx types here.

use std::sync::Arc;

use ship_substrate::{is_safe_ident, Predicate, Query, SpaceContext};
use ship_types::envelope::EventEnvelope;
use crate::events::{
    CollectionCreated, CollectionItemCreated, CollectionItemDeleted, CollectionItemUpdated,
    CollectionSchemaChanged,
};

use crate::error::CollectionError;
use crate::helpers::{emit_ctx, new_ulid, now_rfc3339, row_to_item};
use crate::projection::{
    SchemaRegistry, count_items, ensure_collection_table, ensure_registry_table,
    persist_schema, update_schema_persistent,
};
use crate::types::{ItemRecord, SchemaDoc};

use crate::schema::{parse_item_data, parse_schema_doc, validate_item};

/// Core service — stateless; all state lives in `SpaceContext` + `SchemaRegistry`.
pub struct CollectionService {
    pub ctx: Arc<SpaceContext>,
    pub schemas: Arc<SchemaRegistry>,
}

impl CollectionService {
    /// Resolve a schema, hydrating the registry from the durable
    /// `_schema_registry` on a miss. A fresh service instance (the trigger
    /// engine, a post-restore boot) must see schemas persisted by another
    /// instance before a typed NotFound is honest.
    async fn resolve_schema(&self, id: &str) -> Result<SchemaDoc, CollectionError> {
        if let Some(s) = self.schemas.get(id) {
            return Ok(s);
        }
        let _ = crate::projection::hydrate_from_store(&self.schemas, self.ctx.data()).await;
        self.schemas.get(id).ok_or_else(|| {
            CollectionError::NotFound(format!("collection '{id}' not found"))
        })
    }

    /// List all schemas including system ones.
    pub fn list(&self) -> Vec<SchemaDoc> {
        self.schemas.list()
    }

    /// Get a single schema with its item count.
    pub async fn get(&self, id: &str) -> Result<(SchemaDoc, usize), CollectionError> {
        let schema = self.resolve_schema(id).await?;
        let count = count_items(self.ctx.data(), id).await.unwrap_or(0);
        Ok((schema, count))
    }

    /// Get a single item by id.
    pub async fn get_item(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<ItemRecord, CollectionError> {
        let _schema = self.resolve_schema(collection).await?;
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
        // Resolve the collection from the (physical) `collection_<id>` table
        // the caller asked about. An unknown collection returns a typed
        // `NotFound` — the previous path let the missing-table error from
        // the SQL layer bubble up as an opaque `Internal`, which made
        // `QueryItems` against an unknown collection indistinguishable from
        // a storage fault. The lookup also rejects path-traversal
        // collection ids (`is_safe_ident`) the same way `apply` does.
        let collection = q
            .table
            .strip_prefix("collection_")
            .ok_or_else(|| {
                CollectionError::Validation(format!(
                    "query table must be a 'collection_<id>' table; got {:?}",
                    q.table
                ))
            })?;
        if !is_safe_ident(collection) {
            return Err(CollectionError::NotFound(format!(
                "collection '{collection}' not found"
            )));
        }
        self.resolve_schema(collection).await?;
        let collection = collection.to_string();
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
        let schema = self.resolve_schema(collection).await?;
        let data = parse_item_data(data_json)?;
        validate_item(&schema, &data)?;

        let id = new_ulid();
        let now = now_rfc3339();

        // EMIT-ONLY: validate, then emit the projection-complete event. The
        // `CollectionProjection` on the per-scope event bus writes the row in
        // the SAME append transaction, so the read path sees it immediately.
        // No direct `ctx.data().insert` here — the projection is the sole
        // table writer (ADR D6/D7).
        let envelope = EventEnvelope::new(
            "collection.item_created",
            &id,
            &CollectionItemCreated {
                collection_id: collection.to_string(),
                item_id: id.clone(),
                item_type: None,
                data: data.clone(),
                created_at: now.clone(),
                updated_at: now.clone(),
            },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx(&self.ctx.space_id().to_string())).await?;

        Ok(ItemRecord {
            id,
            collection: collection.to_string(),
            data: data.as_object().cloned().unwrap_or_default(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Update an item with merge/patch semantics — only provided fields are
    /// written; untouched fields retain their existing values.
    pub async fn update_item(
        &self,
        collection: &str,
        id: &str,
        data_json: &[u8],
    ) -> Result<ItemRecord, CollectionError> {
        let schema = self.resolve_schema(collection).await?;
        let patch = parse_item_data(data_json)?;

        let now = now_rfc3339();
        let original = self.get_item(collection, id).await?;

        // Merge patch on top of existing data, then validate the merged result so
        // required-field checks see the full document.
        let mut merged = serde_json::Value::Object(original.data.clone());
        if let (Some(merged_obj), Some(patch_obj)) = (merged.as_object_mut(), patch.as_object()) {
            for (k, v) in patch_obj {
                merged_obj.insert(k.clone(), v.clone());
            }
        }
        validate_item(&schema, &merged)?;

        // EMIT-ONLY: the patch (provided fields only) rides the event; the
        // projection applies a partial column UPDATE in the append transaction
        // and leaves untouched columns alone. No direct `ctx.data().update`.
        let envelope = EventEnvelope::new(
            "collection.item_updated",
            id,
            &CollectionItemUpdated {
                collection_id: collection.to_string(),
                item_id: id.to_string(),
                data: patch.clone(),
                updated_at: now.clone(),
            },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx(&self.ctx.space_id().to_string())).await?;

        Ok(ItemRecord {
            id: id.to_string(),
            collection: collection.to_string(),
            data: merged.as_object().cloned().unwrap_or_default(),
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
        self.resolve_schema(collection).await?;
        // EMIT-ONLY: confirm the row exists against the projection (a delete of
        // a missing item is a NotFound, not a silent no-op), then emit; the
        // projection removes the row in the append transaction. No direct
        // `ctx.data().delete`.
        let _ = self.get_item(collection, id).await?;
        let envelope = EventEnvelope::new(
            "collection.item_deleted",
            id,
            &CollectionItemDeleted {
                collection_id: collection.to_string(),
                item_id: id.to_string(),
            },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx(&self.ctx.space_id().to_string())).await?;
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

        // The service is EMIT-ONLY for `collection_<id>`: it emits
        // `collection.created` and the `CollectionItemProjection` (L1, sole
        // writer per its module header) DDLs the table from the event payload
        // inside the same SQLite transaction as the append. We still bootstrap
        // the `_schema_registry` table here because the projection doesn't own
        // that table — it backs the in-memory `SchemaRegistry`, not collection
        // items.
        ensure_registry_table(self.ctx.data()).await?;

        let schema_str = serde_json::to_string(&doc)
            .map_err(|e| CollectionError::Internal(e.to_string()))?;
        let envelope = EventEnvelope::new(
            "collection.created",
            id,
            &CollectionCreated { collection_id: id.to_string(), schema_json: schema_str },
        )
        .map_err(|e| CollectionError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx(&self.ctx.space_id().to_string())).await?;
        // Durable mirror of the in-memory insert below — the registry's
        // read path (`hydrate_from_store`) restores this row on daemon open.
        // `_schema_registry` lives per-space (Storage Blueprint
        // ship-substrate-db.md); same `self.ctx.data()` the projection and
        // the item-table DDL write to.
        persist_schema(self.ctx.data(), &doc).await?;

        self.schemas.insert(doc.clone());
        Ok(doc)
    }

    /// Update a schema, update table, emit event.
    pub async fn update_schema(
        &self,
        id: &str,
        schema_json: &[u8],
    ) -> Result<SchemaDoc, CollectionError> {
        self.resolve_schema(id).await?;
        let mut doc = parse_schema_doc(schema_json)?;
        doc.id = id.to_string();

        // DDL the table on the read connection (adds any new columns) so its
        // schema cache stays current, then persist the registry row. The
        // `CollectionProjection` independently DDLs on `collection.schema.changed`
        // for the rebuild path; both are idempotent. The dual write is left
        // in place for `update_schema` specifically: removing it exposes a
        // sqlx 0.8.6 auto-reprepare race — the DataService's long-lived
        // SELECT * on `collection_<id>` captures columns metadata before the
        // ALTER and panics on row decode when sqlite auto-reprepares the
        // statement with the new column count. The `create_schema` path
        // (also flagged by the audit) is cleaned up; `update_schema` needs a
        // follow-up that either patches sqlx or moves readers to a
        // per-request connection.
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
        self.ctx.kernel().emit(envelope, &emit_ctx(&self.ctx.space_id().to_string())).await?;

        self.schemas.update(doc.clone());
        Ok(doc)
    }
}

