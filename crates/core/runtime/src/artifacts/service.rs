//! Artifact storage service — writes blobs to disk, tracks metadata in the
//! built-in `artifacts` collection, emits collection events on mutation.
//!
//! Blobs are stored under `{global_ship_dir}/artifacts/{project_slug}/blobs/`.
//! The project slug is supplied at construction — a daemon without a project
//! attached cannot use this service.

use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::collections::query::QueryRequest;
use crate::collections::service as collection_service;
use crate::data::DataService;
use crate::events::KernelRouter;

use super::schema::{ARTIFACTS_COLLECTION_ID, artifacts_schema, ensure_artifacts_collection};

/// Metadata for a stored artifact.
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactRecord {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    pub path: String,
    pub collection_id: Option<String>,
    pub item_id: Option<String>,
    pub created_at: String,
}

impl ArtifactRecord {
    fn from_item(item: &crate::collections::store::Item) -> Self {
        let data = &item.data;
        Self {
            id: item.id.clone(),
            name: data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            mime: data.get("mime").and_then(|v| v.as_str()).unwrap_or("application/octet-stream").to_string(),
            size: data.get("size").and_then(|v| v.as_u64())
                .or_else(|| data.get("size").and_then(|v| v.as_f64()).map(|f| f as u64))
                .unwrap_or(0),
            path: data.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            collection_id: data.get("collection_id").and_then(|v| v.as_str()).map(str::to_string),
            item_id: data.get("item_id").and_then(|v| v.as_str()).map(str::to_string),
            created_at: item.created_at.clone(),
        }
    }
}

/// Project-scoped blob storage for artifacts.
///
/// Constructed with a global `~/.ship/` directory and a project slug.
/// Blobs live at `{global}/artifacts/{slug}/blobs/{id[0:2]}/{id}`.
pub struct ArtifactService {
    blobs_dir: PathBuf,
}

impl ArtifactService {
    /// Create a service for the given project.
    pub fn new(global_ship_dir: &Path, project_slug: &str) -> Self {
        Self {
            blobs_dir: global_ship_dir
                .join("artifacts")
                .join(project_slug)
                .join("blobs"),
        }
    }

    /// Absolute root where this project's blobs live.
    pub fn blobs_dir(&self) -> &Path {
        &self.blobs_dir
    }

    /// Absolute path where the blob for `id` would live.
    pub fn blob_path(&self, id: &str) -> PathBuf {
        let prefix: String = id.chars().take(2).collect();
        self.blobs_dir.join(prefix).join(id)
    }

    /// Relative blob path stored in the metadata row (stable across ship-dir
    /// relocations — still useful for diagnostics even though lookups go
    /// through `blob_path`).
    fn rel_blob_path(id: &str) -> String {
        let prefix: String = id.chars().take(2).collect();
        format!("blobs/{prefix}/{id}")
    }

    /// Store a new blob. Generates a ULID, writes the bytes to disk, and
    /// inserts a metadata row in the artifacts collection. Emits
    /// `collection.item_created`.
    pub async fn store(
        &self,
        svc: &mut DataService,
        kernel: &Arc<Mutex<KernelRouter>>,
        name: &str,
        mime: &str,
        bytes: &[u8],
        collection_id: Option<&str>,
        item_id: Option<&str>,
        created_by: Option<&str>,
    ) -> Result<ArtifactRecord> {
        ensure_artifacts_collection(svc)?;

        let id = crate::gen_ulid();
        let blob_path = self.blob_path(&id);
        if let Some(parent) = blob_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&blob_path, bytes)
            .with_context(|| format!("writing blob {}", blob_path.display()))?;

        let size = bytes.len() as u64;
        let rel_path = Self::rel_blob_path(&id);
        let mut data = serde_json::json!({
            "name": name,
            "path": rel_path,
            "mime": mime,
            "size": size,
        });
        if let Some(cid) = collection_id {
            data["collection_id"] = serde_json::Value::String(cid.to_string());
        }
        if let Some(iid) = item_id {
            data["item_id"] = serde_json::Value::String(iid.to_string());
        }

        let schema = artifacts_schema();
        let item = match collection_service::create_item(
            svc, kernel, &schema, &id, &data, created_by,
        ).await {
            Ok(item) => item,
            Err(e) => {
                let _ = std::fs::remove_file(&blob_path);
                return Err(e);
            }
        };

        Ok(ArtifactRecord::from_item(&item))
    }

    /// Read an artifact's metadata + bytes. `Ok(None)` when no such metadata
    /// exists. Returns a specific error when metadata exists but the blob
    /// is missing on disk — callers map that to 404.
    pub fn get(&self, svc: &mut DataService, id: &str) -> Result<Option<(ArtifactRecord, Vec<u8>)>> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        let Some(item) = collection_service::get_item(svc, &schema, id)? else {
            return Ok(None);
        };
        let record = ArtifactRecord::from_item(&item);
        let blob_path = self.blob_path(id);
        let bytes = match std::fs::read(&blob_path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(anyhow!("blob missing: {}", blob_path.display()));
            }
            Err(e) => return Err(e.into()),
        };
        Ok(Some((record, bytes)))
    }

    /// Get just the metadata for an artifact.
    pub fn get_record(&self, svc: &mut DataService, id: &str) -> Result<Option<ArtifactRecord>> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        Ok(collection_service::get_item(svc, &schema, id)?
            .as_ref()
            .map(ArtifactRecord::from_item))
    }

    /// Delete an artifact's blob and metadata. Returns `Ok(false)` when no
    /// such metadata exists. Emits `collection.item_deleted`. A missing blob
    /// on disk is tolerated — metadata is removed regardless.
    pub async fn delete(
        &self,
        svc: &mut DataService,
        kernel: &Arc<Mutex<KernelRouter>>,
        id: &str,
    ) -> Result<bool> {
        ensure_artifacts_collection(svc)?;
        let blob_path = self.blob_path(id);
        let deleted = collection_service::delete_item(
            svc, kernel, ARTIFACTS_COLLECTION_ID, id,
        ).await?;
        if deleted {
            match std::fs::remove_file(&blob_path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(deleted)
    }

    /// List artifacts via a standard collection query. The `collection` field
    /// on the request is ignored — queries always run against `artifacts`.
    pub fn list(&self, svc: &mut DataService, mut req: QueryRequest) -> Result<Vec<ArtifactRecord>> {
        ensure_artifacts_collection(svc)?;
        req.collection = ARTIFACTS_COLLECTION_ID.to_string();
        let schema = artifacts_schema();
        let items = collection_service::query_items(svc, &schema, &req)?;
        Ok(items.iter().map(ArtifactRecord::from_item).collect())
    }
}
