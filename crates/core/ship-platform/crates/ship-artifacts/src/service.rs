// FILE_CAP_OVERRIDE: promote requires coordinated blob-write + metadata-insert + old-delete + GC;
// splitting across files would break the atomicity story. All other methods are thin.

//! ArtifactService core — all storage via `SpaceContext.data()` + private
//! `BlobBackend`. No sqlite/sqlx/rusqlite types here.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use tokio::sync::Mutex;

use ship_substrate::ports::{PrimaryKey, SpaceContext};
use ship_types::envelope::EventEnvelope;
use ship_types::payload::{
    ArtifactCreated, ArtifactDeleted, ArtifactMoved, ArtifactUpdated,
};

use crate::backend::{BlobBackend, LocalFsBackend};
use crate::error::ArtifactError;
use crate::helpers::{emit_ctx, new_ulid, now_rfc3339};
use ship_types::sha256_hex;
use crate::projection::{
    TABLE, ensure_artifacts_table, find_by_hash, count_path_refs,
    get_by_id, query_artifacts, record_to_row, record_to_scope,
};
use crate::types::{ArtifactRecord, ArtifactScope};

/// Core artifact service. Stateless — all state lives in `SpaceContext` + backend.
pub struct ArtifactService {
    pub ctx: Arc<SpaceContext>,
    backend: Arc<dyn BlobBackend>,
    /// Per-scope async locks for dedup-safe concurrent store/promote.
    scope_locks: StdMutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl ArtifactService {
    pub fn new(ctx: Arc<SpaceContext>, global_dir: std::path::PathBuf) -> Self {
        Self {
            ctx,
            backend: Arc::new(LocalFsBackend::new(global_dir)),
            scope_locks: StdMutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_backend(ctx: Arc<SpaceContext>, backend: Arc<dyn BlobBackend>) -> Self {
        Self {
            ctx,
            backend,
            scope_locks: StdMutex::new(HashMap::new()),
        }
    }

    fn scope_lock(&self, scope: &ArtifactScope) -> Arc<Mutex<()>> {
        let key = scope.lock_key();
        let mut map = self.scope_locks.lock().expect("scope_locks poisoned");
        map.entry(key).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
    }

    /// Ensure the metadata table exists.
    pub async fn ensure_table(&self) -> Result<(), ArtifactError> {
        ensure_artifacts_table(self.ctx.data()).await.map_err(ArtifactError::from)
    }

    /// Store a new artifact. Content-addressed dedup within the same scope:
    /// if bytes with the same hash already exist, a new metadata row is
    /// created but the blob file is shared.
    pub async fn store(
        &self,
        name: &str,
        mime: &str,
        bytes: &[u8],
        scope: &ArtifactScope,
        collection_id: Option<&str>,
        item_id: Option<&str>,
        owner_actor: Option<&str>,
        expires_at: Option<&str>,
    ) -> Result<ArtifactRecord, ArtifactError> {
        self.ensure_table().await?;
        let lock = self.scope_lock(scope);
        let _guard = lock.lock().await;

        let id = new_ulid();
        let now = now_rfc3339();
        let content_hash = sha256_hex(bytes);

        let rel_path = match find_by_hash(self.ctx.data(), scope, &content_hash).await? {
            Some(existing) => existing,
            None => {
                self.backend.put(scope, &id, bytes)
                    .map_err(|e| ArtifactError::Blob(e.to_string()))?;
                self.backend.rel_path(scope, &id)
            }
        };

        let record = ArtifactRecord {
            id: id.clone(),
            name: name.to_string(),
            mime: mime.to_string(),
            size: bytes.len() as u64,
            path: rel_path,
            scope: scope.kind_str().to_string(),
            space_id: scope.space_id().map(str::to_string),
            run_id: scope.run_id().map(str::to_string),
            // user-scope was removed (artifacts are Run|Project); column kept
            // for the persisted row but never set.
            user_id: None,
            content_hash: Some(content_hash.clone()),
            expires_at: expires_at.map(str::to_string),
            owner_actor: owner_actor.map(str::to_string),
            collection_id: collection_id.map(str::to_string),
            item_id: item_id.map(str::to_string),
            created_at: now,
        };

        let row = record_to_row(&record);
        if let Err(e) = self.ctx.data().insert(TABLE, row).await {
            let _ = self.backend.delete(scope, &id);
            return Err(ArtifactError::from(e));
        }

        let envelope = EventEnvelope::new(
            "artifact.created",
            &id,
            &ArtifactCreated {
                id: id.clone(),
                scope: scope.kind_str().to_string(),
                name: name.to_string(),
                mime: mime.to_string(),
                size: bytes.len() as u64,
                content_hash: Some(content_hash),
                owner_actor: owner_actor.map(str::to_string),
                space_id: scope.space_id().map(str::to_string),
                run_id: scope.run_id().map(str::to_string),
            },
        )
        .map_err(|e| ArtifactError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx(scope.space_id())).await?;

        Ok(record)
    }

    /// Get blob bytes for an artifact.
    pub async fn get_blob(&self, id: &str) -> Result<(ArtifactRecord, Vec<u8>), ArtifactError> {
        let record = self.get_record(id).await?.ok_or_else(|| {
            ArtifactError::NotFound(format!("artifact '{id}' not found"))
        })?;
        let from_scope = record_to_scope(&record)?;
        let bytes = self.backend.get(&from_scope, &record.id)
            .or_else(|_| {
                // Path may be a dedup alias; try reading from the stored path directly.
                // Extract the blob id from the last segment of the stored path.
                let blob_id = record.path.split('/').next_back().unwrap_or(&record.id);
                self.backend.get(&from_scope, blob_id)
            })
            .map_err(|e| ArtifactError::Blob(e.to_string()))?;
        Ok((record, bytes))
    }

    /// Get metadata record without bytes.
    pub async fn get_record(&self, id: &str) -> Result<Option<ArtifactRecord>, ArtifactError> {
        self.ensure_table().await?;
        get_by_id(self.ctx.data(), id).await
    }

    /// List artifacts with optional filters (scope, mime, collection_id, item_id,
    /// owner_actor, content_hash).
    pub async fn list(
        &self,
        filter: BTreeMap<String, serde_json::Value>,
        limit: u32,
    ) -> Result<Vec<ArtifactRecord>, ArtifactError> {
        self.ensure_table().await?;
        query_artifacts(self.ctx.data(), filter, limit, None).await
    }

    /// Patch mutable metadata fields (name, owner_actor, expires_at).
    pub async fn patch(
        &self,
        id: &str,
        name: Option<&str>,
        owner_actor: Option<&str>,
        expires_at: Option<&str>,
    ) -> Result<ArtifactRecord, ArtifactError> {
        self.ensure_table().await?;
        let mut record = self.get_record(id).await?.ok_or_else(|| {
            ArtifactError::NotFound(format!("artifact '{id}' not found"))
        })?;
        let mut changed = serde_json::json!({});
        if let Some(n) = name {
            record.name = n.to_string();
            changed["name"] = serde_json::Value::String(n.to_string());
        }
        if let Some(a) = owner_actor {
            record.owner_actor = Some(a.to_string());
            changed["owner_actor"] = serde_json::Value::String(a.to_string());
        }
        if let Some(e) = expires_at {
            record.expires_at = Some(e.to_string());
            changed["expires_at"] = serde_json::Value::String(e.to_string());
        }
        let row = record_to_row(&record);
        self.ctx.data().update(TABLE, row).await?;

        let scope = record.scope.clone();
        let space = record.space_id.clone();
        let envelope = EventEnvelope::new(
            "artifact.updated",
            id,
            &ArtifactUpdated { id: id.to_string(), scope, changed },
        )
        .map_err(|e| ArtifactError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx(space.as_deref())).await?;

        Ok(record)
    }

    /// Promote an artifact to a new scope. Write order: write new blob →
    /// insert new metadata → delete old metadata → GC old blob.
    pub async fn promote(
        &self,
        id: &str,
        target_scope: ArtifactScope,
        retained: bool,
    ) -> Result<ArtifactRecord, ArtifactError> {
        self.ensure_table().await?;
        let old = self.get_record(id).await?.ok_or_else(|| {
            ArtifactError::NotFound(format!("artifact '{id}' not found"))
        })?;
        let from_scope = record_to_scope(&old)?;
        let old_path = old.path.clone();

        // Acquire scope locks in deterministic order to avoid deadlock.
        let from_key = from_scope.lock_key();
        let to_key = target_scope.lock_key();
        let (first_lock, second_lock) = if from_key == to_key {
            (self.scope_lock(&from_scope), None)
        } else if from_key < to_key {
            (self.scope_lock(&from_scope), Some(self.scope_lock(&target_scope)))
        } else {
            (self.scope_lock(&target_scope), Some(self.scope_lock(&from_scope)))
        };
        let _g1 = first_lock.lock().await;
        let _g2 = if let Some(l) = second_lock { Some(l.lock_owned().await) } else { None };

        // Read bytes via the stored blob id (last path segment).
        let blob_id = old_path.split('/').next_back().unwrap_or(&old.id).to_string();
        let bytes = self.backend.get(&from_scope, &blob_id)
            .map_err(|e| ArtifactError::Blob(e.to_string()))?;

        let new_id = new_ulid();
        let content_hash = sha256_hex(&bytes);

        let rel_path = match find_by_hash(self.ctx.data(), &target_scope, &content_hash).await? {
            Some(existing) => existing,
            None => {
                self.backend.put(&target_scope, &new_id, &bytes)
                    .map_err(|e| ArtifactError::Blob(e.to_string()))?;
                self.backend.rel_path(&target_scope, &new_id)
            }
        };

        let now = now_rfc3339();
        let new_record = ArtifactRecord {
            id: new_id.clone(),
            name: old.name.clone(),
            mime: old.mime.clone(),
            size: old.size,
            path: rel_path,
            scope: target_scope.kind_str().to_string(),
            space_id: target_scope.space_id().map(str::to_string),
            run_id: target_scope.run_id().map(str::to_string),
            user_id: None,
            content_hash: Some(content_hash),
            expires_at: if retained { None } else { old.expires_at.clone() },
            owner_actor: old.owner_actor.clone(),
            collection_id: old.collection_id.clone(),
            item_id: old.item_id.clone(),
            created_at: now,
        };

        let new_row = record_to_row(&new_record);
        if let Err(e) = self.ctx.data().insert(TABLE, new_row).await {
            let _ = self.backend.delete(&target_scope, &new_id);
            return Err(ArtifactError::from(e));
        }

        self.ctx.data()
            .delete(TABLE, PrimaryKey(id.to_string()))
            .await
            .map_err(ArtifactError::from)?;

        // Best-effort old-blob GC.
        match count_path_refs(self.ctx.data(), &from_scope, &old_path).await {
            Ok(0) => {
                let old_blob_id = old_path.split('/').next_back().unwrap_or(id).to_string();
                let _ = self.backend.delete(&from_scope, &old_blob_id);
            }
            _ => {}
        }

        let from_scope_str = old.scope.clone();
        let to_scope_str = target_scope.kind_str().to_string();
        let envelope = EventEnvelope::new(
            "artifact.moved",
            &new_id,
            &ArtifactMoved { id: new_id.clone(), from_scope: from_scope_str, to_scope: to_scope_str },
        )
        .map_err(|e| ArtifactError::Internal(e.to_string()))?;
        self.ctx.kernel().emit(envelope, &emit_ctx(target_scope.space_id())).await?;

        Ok(new_record)
    }

    /// Delete an artifact — removes metadata row and blob (if no other row
    /// references the same path).
    pub async fn delete(&self, id: &str) -> Result<bool, ArtifactError> {
        self.ensure_table().await?;
        let record = match self.get_record(id).await? {
            Some(r) => r,
            None => return Ok(false),
        };
        let scope = record_to_scope(&record)?;
        let rel_path = record.path.clone();

        let deleted = self.ctx.data().delete(TABLE, PrimaryKey(id.to_string())).await?;
        if deleted {
            match count_path_refs(self.ctx.data(), &scope, &rel_path).await {
                Ok(0) => {
                    let blob_id = rel_path.split('/').next_back().unwrap_or(id).to_string();
                    let _ = self.backend.delete(&scope, &blob_id);
                }
                _ => {}
            }

            let envelope = EventEnvelope::new(
                "artifact.deleted",
                id,
                &ArtifactDeleted {
                    id: id.to_string(),
                    scope: record.scope.clone(),
                    space_id: record.space_id.clone(),
                    run_id: record.run_id.clone(),
                },
            )
            .map_err(|e| ArtifactError::Internal(e.to_string()))?;
            self.ctx.kernel().emit(envelope, &emit_ctx(record.space_id.as_deref())).await?;
        }
        Ok(deleted)
    }
}
