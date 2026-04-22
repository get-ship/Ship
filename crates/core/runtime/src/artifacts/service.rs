//! Artifact storage service — three-scope blob store with collection-backed
//! metadata and a pluggable storage backend.
//!
//! Scope layout (LocalFsBackend):
//!   run:     {global}/artifacts/{project}/runs/{run_id}/{id[0:2]}/{id}
//!   project: {global}/artifacts/{project}/blobs/{id[0:2]}/{id}
//!   user:    {global}/artifacts/users/{user_id}/{id[0:2]}/{id}

use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::collections::query::QueryRequest;
use crate::collections::service as collection_service;
use crate::data::DataService;
use crate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

use super::backend::{ArtifactBackend, ArtifactScope, LocalFsBackend, sha256_hex};
use super::schema::{ARTIFACTS_COLLECTION_ID, artifacts_schema, ensure_artifacts_collection};

/// Metadata record returned by all service methods.
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactRecord {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    pub path: String,
    pub scope: String,
    pub project_id: Option<String>,
    pub run_id: Option<String>,
    pub user_id: Option<String>,
    pub content_hash: Option<String>,
    pub expires_at: Option<String>,
    pub owner_actor: Option<String>,
    pub collection_id: Option<String>,
    pub item_id: Option<String>,
    pub created_at: String,
}

impl ArtifactRecord {
    fn from_item(item: &crate::collections::store::Item) -> Self {
        let d = &item.data;
        let str_field = |k: &str| d.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let opt_str = |k: &str| d.get(k).and_then(|v| v.as_str()).map(str::to_string);
        Self {
            id: item.id.clone(),
            name: str_field("name"),
            mime: d.get("mime").and_then(|v| v.as_str())
                .unwrap_or("application/octet-stream").to_string(),
            size: d.get("size").and_then(|v| v.as_u64())
                .or_else(|| d.get("size").and_then(|v| v.as_f64()).map(|f| f as u64))
                .unwrap_or(0),
            path: str_field("path"),
            scope: d.get("scope").and_then(|v| v.as_str()).unwrap_or("project").to_string(),
            project_id: opt_str("project_id"),
            run_id: opt_str("run_id"),
            user_id: opt_str("user_id"),
            content_hash: opt_str("content_hash"),
            expires_at: opt_str("expires_at"),
            owner_actor: opt_str("owner_actor"),
            collection_id: opt_str("collection_id"),
            item_id: opt_str("item_id"),
            created_at: item.created_at.clone(),
        }
    }
}

/// Blob store + metadata service for artifacts.
///
/// Constructed with only the global ship dir — scope is a per-call argument.
pub struct ArtifactService {
    backend: Box<dyn ArtifactBackend>,
    global_dir: std::path::PathBuf,
}

impl ArtifactService {
    /// Create a service backed by the local filesystem.
    pub fn new(global_ship_dir: &Path) -> Self {
        Self {
            backend: Box::new(LocalFsBackend::new(global_ship_dir.to_path_buf())),
            global_dir: global_ship_dir.to_path_buf(),
        }
    }

    /// Create a service with a custom backend (test/phase-3 injection point).
    pub fn with_backend(global_ship_dir: &Path, backend: Box<dyn ArtifactBackend>) -> Self {
        Self {
            backend,
            global_dir: global_ship_dir.to_path_buf(),
        }
    }

    /// Store a new artifact. Generates a ULID, writes the blob, and inserts a
    /// metadata row. Content-addressed dedup: if the same bytes (same hash)
    /// already exist under the same scope, a new metadata row is created but
    /// the blob file is shared (written only when missing).
    ///
    /// Emits `artifact.created` and `collection.item_created`.
    pub async fn store(
        &self,
        svc: &mut DataService,
        kernel: &Arc<Mutex<KernelRouter>>,
        name: &str,
        mime: &str,
        bytes: &[u8],
        scope: &ArtifactScope,
        collection_id: Option<&str>,
        item_id: Option<&str>,
        owner_actor: Option<&str>,
        expires_at: Option<&str>,
    ) -> Result<ArtifactRecord> {
        ensure_artifacts_collection(svc)?;

        let id = crate::gen_ulid();
        let content_hash = sha256_hex(bytes);

        // Dedup: only write blob file if it doesn't already exist at this path.
        if !self.backend.exists(scope, &id)? {
            self.backend.put(scope, &id, bytes)
                .with_context(|| format!("writing blob for {id}"))?;
        }

        let backend_fs = self.global_dir.as_path();
        let local = LocalFsBackend::new(backend_fs.to_path_buf());
        let rel_path = local.rel_blob_path(scope, &id);

        let mut data = serde_json::json!({
            "name": name,
            "path": rel_path,
            "mime": mime,
            "size": bytes.len() as u64,
            "scope": scope.kind_str(),
            "content_hash": content_hash,
        });

        if let Some(pid) = scope.project_id() {
            data["project_id"] = serde_json::Value::String(pid.to_string());
        }
        if let Some(rid) = scope.run_id() {
            data["run_id"] = serde_json::Value::String(rid.to_string());
        }
        if let Some(uid) = scope.user_id() {
            data["user_id"] = serde_json::Value::String(uid.to_string());
        }
        if let Some(actor) = owner_actor {
            data["owner_actor"] = serde_json::Value::String(actor.to_string());
        }
        if let Some(exp) = expires_at {
            data["expires_at"] = serde_json::Value::String(exp.to_string());
        }
        if let Some(cid) = collection_id {
            data["collection_id"] = serde_json::Value::String(cid.to_string());
        }
        if let Some(iid) = item_id {
            data["item_id"] = serde_json::Value::String(iid.to_string());
        }

        let schema = artifacts_schema();
        let item = match collection_service::create_item(
            svc, kernel, &schema, &id, &data, owner_actor,
        ).await {
            Ok(item) => item,
            Err(e) => {
                // Roll back blob write on metadata failure.
                let _ = self.backend.delete(scope, &id);
                return Err(e);
            }
        };

        let record = ArtifactRecord::from_item(&item);

        // Emit artifact-specific event alongside the collection event.
        emit_artifact_event(kernel, "artifact.created", &id, serde_json::json!({
            "id": id,
            "scope": scope.kind_str(),
            "name": name,
            "mime": mime,
            "size": bytes.len() as u64,
            "content_hash": content_hash,
            "owner_actor": owner_actor,
            "project_id": scope.project_id(),
            "run_id": scope.run_id(),
        })).await;

        Ok(record)
    }

    /// Get metadata + bytes. `Ok(None)` when the metadata row doesn't exist.
    pub fn get(
        &self,
        svc: &mut DataService,
        id: &str,
    ) -> Result<Option<(ArtifactRecord, Vec<u8>)>> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        let Some(item) = collection_service::get_item(svc, &schema, id)? else {
            return Ok(None);
        };
        let record = ArtifactRecord::from_item(&item);
        let scope = record_to_scope(&record)?;
        let bytes = self.backend.get(&scope, id)?;
        Ok(Some((record, bytes)))
    }

    /// Get just the metadata record.
    pub fn get_record(&self, svc: &mut DataService, id: &str) -> Result<Option<ArtifactRecord>> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        Ok(collection_service::get_item(svc, &schema, id)?
            .as_ref()
            .map(ArtifactRecord::from_item))
    }

    /// Delete an artifact. Returns `false` when the metadata row doesn't
    /// exist. Blob deletion tolerates a missing file. Emits `artifact.deleted`.
    pub async fn delete(
        &self,
        svc: &mut DataService,
        kernel: &Arc<Mutex<KernelRouter>>,
        id: &str,
    ) -> Result<bool> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        let record = match collection_service::get_item(svc, &schema, id)? {
            Some(item) => ArtifactRecord::from_item(&item),
            None => return Ok(false),
        };
        let scope = record_to_scope(&record)?;

        let deleted = collection_service::delete_item(
            svc, kernel, ARTIFACTS_COLLECTION_ID, id,
        ).await?;
        if deleted {
            self.backend.delete(&scope, id)?;
            emit_artifact_event(kernel, "artifact.deleted", id, serde_json::json!({
                "id": id,
                "scope": record.scope,
                "project_id": record.project_id,
                "run_id": record.run_id,
            })).await;
        }
        Ok(deleted)
    }

    /// Update mutable metadata fields (name, owner_actor, expires_at).
    /// Bytes are immutable — changing content means a new artifact.
    /// Emits `artifact.updated`.
    pub async fn patch(
        &self,
        svc: &mut DataService,
        kernel: &Arc<Mutex<KernelRouter>>,
        id: &str,
        name: Option<&str>,
        owner_actor: Option<&str>,
        expires_at: Option<&str>,
    ) -> Result<Option<ArtifactRecord>> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        let Some(existing) = collection_service::get_item(svc, &schema, id)? else {
            return Ok(None);
        };
        let mut data = existing.data.clone();
        let mut changed = serde_json::json!({});
        if let Some(n) = name {
            data["name"] = serde_json::Value::String(n.to_string());
            changed["name"] = serde_json::Value::String(n.to_string());
        }
        if let Some(actor) = owner_actor {
            data["owner_actor"] = serde_json::Value::String(actor.to_string());
            changed["owner_actor"] = serde_json::Value::String(actor.to_string());
        }
        if let Some(exp) = expires_at {
            data["expires_at"] = serde_json::Value::String(exp.to_string());
            changed["expires_at"] = serde_json::Value::String(exp.to_string());
        }
        let updated = collection_service::update_item(svc, kernel, &schema, id, &data).await?;
        let record = ArtifactRecord::from_item(&updated);
        emit_artifact_event(kernel, "artifact.updated", id, serde_json::json!({
            "id": id,
            "scope": record.scope,
            "changed": changed,
        })).await;
        Ok(Some(record))
    }

    /// Promote an artifact from its current scope to `target_scope`.
    ///
    /// Write order ensures crash-safe semantics:
    /// 1. Write blob at new path.
    /// 2. Insert new metadata row.
    /// 3. Delete old metadata row.
    /// 4. Delete old blob.
    ///
    /// Emits `artifact.moved`.
    pub async fn promote(
        &self,
        svc: &mut DataService,
        kernel: &Arc<Mutex<KernelRouter>>,
        id: &str,
        target_scope: ArtifactScope,
    ) -> Result<ArtifactRecord> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        let Some(existing_item) = collection_service::get_item(svc, &schema, id)? else {
            return Err(anyhow!("artifact '{id}' not found"));
        };
        let old_record = ArtifactRecord::from_item(&existing_item);
        let from_scope = record_to_scope(&old_record)?;

        // Read bytes from old location.
        let bytes = self.backend.get(&from_scope, id)?;

        let new_id = crate::gen_ulid();
        let content_hash = sha256_hex(&bytes);

        // 1. Write blob to new location.
        self.backend.put(&target_scope, &new_id, &bytes)?;

        // 2. Build new metadata.
        let local = LocalFsBackend::new(self.global_dir.clone());
        let rel_path = local.rel_blob_path(&target_scope, &new_id);
        let mut new_data = existing_item.data.clone();
        new_data["path"] = serde_json::Value::String(rel_path);
        new_data["scope"] = serde_json::Value::String(target_scope.kind_str().to_string());
        new_data["content_hash"] = serde_json::Value::String(content_hash);
        // Clear scope-specific IDs then set appropriate ones.
        new_data["project_id"] = serde_json::Value::Null;
        new_data["run_id"] = serde_json::Value::Null;
        new_data["user_id"] = serde_json::Value::Null;
        if let Some(pid) = target_scope.project_id() {
            new_data["project_id"] = serde_json::Value::String(pid.to_string());
        }
        if let Some(rid) = target_scope.run_id() {
            new_data["run_id"] = serde_json::Value::String(rid.to_string());
        }
        if let Some(uid) = target_scope.user_id() {
            new_data["user_id"] = serde_json::Value::String(uid.to_string());
        }

        let owner = old_record.owner_actor.as_deref();
        let new_item = match collection_service::create_item(
            svc, kernel, &schema, &new_id, &new_data, owner,
        ).await {
            Ok(item) => item,
            Err(e) => {
                let _ = self.backend.delete(&target_scope, &new_id);
                return Err(e);
            }
        };

        // 3. Delete old metadata.
        let _ = collection_service::delete_item(svc, kernel, ARTIFACTS_COLLECTION_ID, id).await;

        // 4. Delete old blob.
        let _ = self.backend.delete(&from_scope, id);

        let new_record = ArtifactRecord::from_item(&new_item);
        emit_artifact_event(kernel, "artifact.moved", &new_id, serde_json::json!({
            "id": new_id,
            "from_scope": old_record.scope,
            "to_scope": target_scope.kind_str(),
        })).await;

        Ok(new_record)
    }

    /// List artifacts via a standard collection query.
    pub fn list(&self, svc: &mut DataService, mut req: QueryRequest) -> Result<Vec<ArtifactRecord>> {
        ensure_artifacts_collection(svc)?;
        req.collection = ARTIFACTS_COLLECTION_ID.to_string();
        let schema = artifacts_schema();
        let items = collection_service::query_items(svc, &schema, &req)?;
        Ok(items.iter().map(ArtifactRecord::from_item).collect())
    }
}

/// Reconstruct the scope from a metadata record.
fn record_to_scope(record: &ArtifactRecord) -> Result<ArtifactScope> {
    match record.scope.as_str() {
        "run" => {
            let project_id = record.project_id.clone()
                .ok_or_else(|| anyhow!("run-scope artifact missing project_id"))?;
            let run_id = record.run_id.clone()
                .ok_or_else(|| anyhow!("run-scope artifact missing run_id"))?;
            Ok(ArtifactScope::Run { project_id, run_id })
        }
        "user" => {
            let user_id = record.user_id.clone()
                .ok_or_else(|| anyhow!("user-scope artifact missing user_id"))?;
            Ok(ArtifactScope::User { user_id })
        }
        _ => {
            let project_id = record.project_id.clone().unwrap_or_default();
            Ok(ArtifactScope::Project { project_id })
        }
    }
}

/// Fire a best-effort artifact event through the kernel router.
async fn emit_artifact_event(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    entity_id: &str,
    payload: serde_json::Value,
) {
    let envelope = match EventEnvelope::new(event_type, entity_id, &payload) {
        Ok(e) => e.with_actor_id("service.artifacts"),
        Err(e) => {
            tracing::error!(event_type, entity_id, "artifacts: failed to build event: {e}");
            return;
        }
    };
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::error!(event_type, entity_id, "artifacts: event emission failed: {e}");
    }
}
