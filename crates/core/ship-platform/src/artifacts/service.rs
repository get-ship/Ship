//! Artifact storage service — three-scope blob store with collection-backed
//! metadata and a pluggable storage backend.
//!
//! Scope layout (LocalFsBackend):
//!   run:     {global}/artifacts/{project}/runs/{run_id}/{id[0:2]}/{id}
//!   project: {global}/artifacts/{project}/blobs/{id[0:2]}/{id}
//!   user:    {global}/artifacts/users/{user_id}/{id[0:2]}/{id}

use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex;

use crate::collections::query::QueryRequest;
use crate::collections::service as collection_service;
use ship_substrate::data::DataService;
use ship_substrate::events::{EmitContext, EventEnvelope, KernelRouter};

use super::backend::{ArtifactBackend, ArtifactScope, LocalFsBackend};
use ship_types::sha256_hex;
use super::schema::{ARTIFACTS_COLLECTION_ID, artifacts_schema, ensure_artifacts_collection};

/// Metadata record returned by all service methods.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArtifactRecord {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    pub path: String,
    pub scope: String,
    pub space_id: Option<String>,
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
    /// Project a stored row into a record. All schema-required fields
    /// (`name`, `path`, `mime`, `size`, `scope`) error on absence —
    /// schema validation rejects insertion of rows that lack them, so a
    /// malformed row signals a bug. Numbers read as u64-or-f64 because
    /// SQLite REAL columns roundtrip integers through floats.
    fn from_item(item: &crate::collections::store::Item) -> Result<Self> {
        let d = &item.data;
        let require_str = |k: &str| -> Result<String> {
            d.get(k)
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .ok_or_else(|| anyhow!("artifact row {} missing required field `{}`", item.id, k))
        };
        let opt_str = |k: &str| d.get(k).and_then(|v| v.as_str()).map(str::to_string);
        let size = d
            .get("size")
            .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
            .ok_or_else(|| anyhow!("artifact row {} missing required field `size`", item.id))?;
        Ok(Self {
            id: item.id.clone(),
            name: require_str("name")?,
            mime: require_str("mime")?,
            size,
            path: require_str("path")?,
            scope: require_str("scope")?,
            space_id: opt_str("space_id"),
            run_id: opt_str("run_id"),
            user_id: opt_str("user_id"),
            content_hash: opt_str("content_hash"),
            expires_at: opt_str("expires_at"),
            owner_actor: opt_str("owner_actor"),
            collection_id: opt_str("collection_id"),
            item_id: opt_str("item_id"),
            created_at: item.created_at.clone(),
        })
    }
}

/// Blob store + metadata service for artifacts.
///
/// Constructed with only the global ship dir — scope is a per-call argument.
///
/// Concurrency: `store` and `promote` take a per-scope async lock so that
/// concurrent writes of identical bytes into the same scope can't both
/// miss the dedup check and produce divergent paths. Reads (`get`,
/// `get_record`, `list`) are lock-free.
pub struct ArtifactService {
    backend: Box<dyn ArtifactBackend>,
    global_dir: std::path::PathBuf,
    scope_locks: StdMutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl ArtifactService {
    /// Create a service backed by the local filesystem.
    pub fn new(global_ship_dir: &Path) -> Self {
        Self {
            backend: Box::new(LocalFsBackend::new(global_ship_dir.to_path_buf())),
            global_dir: global_ship_dir.to_path_buf(),
            scope_locks: StdMutex::new(HashMap::new()),
        }
    }

    /// Create a service with a custom backend (test/phase-3 injection point).
    pub fn with_backend(global_ship_dir: &Path, backend: Box<dyn ArtifactBackend>) -> Self {
        Self {
            backend,
            global_dir: global_ship_dir.to_path_buf(),
            scope_locks: StdMutex::new(HashMap::new()),
        }
    }

    /// Return the per-scope async lock, creating it on first use.
    fn scope_lock(&self, scope: &ArtifactScope) -> Arc<Mutex<()>> {
        let key = scope.lock_key();
        let mut map = self.scope_locks.lock().expect("scope_locks poisoned");
        map.entry(key).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
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

        // Per-scope lock serializes dedup-check + blob-write + metadata-insert,
        // so concurrent stores of identical bytes can't both miss the dedup
        // check and produce orphaned blobs.
        let lock = self.scope_lock(scope);
        let _guard = lock.lock().await;

        let id = ship_substrate::gen_ulid();
        let content_hash = sha256_hex(bytes);

        let backend_fs = self.global_dir.as_path();
        let local = LocalFsBackend::new(backend_fs.to_path_buf());

        // Content-addressed dedup within the same scope: if an existing
        // metadata row in this scope has the same content_hash, reuse its
        // blob path in the new row instead of writing a duplicate file.
        let rel_path = match find_blob_by_hash(svc, scope, &content_hash)? {
            Some(existing_path) => existing_path,
            None => {
                self.backend.put(scope, &id, bytes)
                    .with_context(|| format!("writing blob for {id}"))?;
                local.rel_blob_path(scope, &id)
            }
        };

        let mut data = serde_json::json!({
            "name": name,
            "path": rel_path,
            "mime": mime,
            "size": bytes.len() as u64,
            "scope": scope.kind_str(),
            "content_hash": content_hash,
        });

        if let Some(pid) = scope.space_id() {
            data["space_id"] = serde_json::Value::String(pid.to_string());
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
            svc, kernel, &schema, scope.space_id(), &id, &data, owner_actor,
        ).await {
            Ok(item) => item,
            Err(e) => {
                // Roll back blob write on metadata failure.
                let _ = self.backend.delete(scope, &id);
                return Err(e);
            }
        };

        let record = ArtifactRecord::from_item(&item)?;

        // Emit artifact-specific event alongside the collection event, bound to
        // the artifact's own space.
        emit_artifact_event(kernel, "artifact.created", &id, scope.space_id(), serde_json::json!({
            "id": id,
            "scope": scope.kind_str(),
            "name": name,
            "mime": mime,
            "size": bytes.len() as u64,
            "content_hash": content_hash,
            "owner_actor": owner_actor,
            "space_id": scope.space_id(),
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
        let record = ArtifactRecord::from_item(&item)?;
        // Under dedup a row's blob may live at another row's id-derived
        // path. Resolve from the record's stored relative path, not from
        // this row's id.
        let abs = self.global_dir.join(&record.path);
        let bytes = match std::fs::read(&abs) {
            Ok(b) => b,
            // Metadata exists but the blob file is gone — present as a
            // missing artifact rather than an internal error. Lets the
            // route map this case to 404 instead of 500.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(e).with_context(|| format!("reading blob {}", abs.display()));
            }
        };
        Ok(Some((record, bytes)))
    }

    /// Get just the metadata record.
    pub fn get_record(&self, svc: &mut DataService, id: &str) -> Result<Option<ArtifactRecord>> {
        ensure_artifacts_collection(svc)?;
        let schema = artifacts_schema();
        collection_service::get_item(svc, &schema, id)?
            .as_ref()
            .map(ArtifactRecord::from_item)
            .transpose()
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
            Some(item) => ArtifactRecord::from_item(&item)?,
            None => return Ok(false),
        };
        let scope = record_to_scope(&record)?;

        let rel_path = record.path.clone();
        let deleted = collection_service::delete_item(
            svc, kernel, ARTIFACTS_COLLECTION_ID, scope.space_id(), id,
        ).await?;
        if deleted {
            // Only drop the blob bytes when no other metadata row (from
            // dedup) still references this path in the same scope.
            if count_refs_to_path(svc, &scope, &rel_path)? == 0 {
                let abs = self.global_dir.join(&rel_path);
                match std::fs::remove_file(&abs) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
            }
            emit_artifact_event(kernel, "artifact.deleted", id, record.space_id.as_deref(), serde_json::json!({
                "id": id,
                "scope": record.scope,
                "space_id": record.space_id,
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
        // The artifact's owning space, for the Space-scoped collection event.
        let space = existing.data.get("space_id").and_then(|v| v.as_str()).map(str::to_string);
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
        let updated = collection_service::update_item(svc, kernel, &schema, space.as_deref(), id, &data).await?;
        let record = ArtifactRecord::from_item(&updated)?;
        emit_artifact_event(kernel, "artifact.updated", id, record.space_id.as_deref(), serde_json::json!({
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
        // Promote-to-global is a dead paradigm: a space-less (User/global) target
        // would produce a Space-declared `artifact.moved` with no space. Promotion
        // targets a space-bearing scope (Run/Project) only; reject the rest.
        if target_scope.space_id().is_none() {
            return Err(anyhow!(
                "promote target must be a space-scoped artifact (run|project); \
                 promote-to-global is not supported"
            ));
        }
        let schema = artifacts_schema();
        let Some(existing_item) = collection_service::get_item(svc, &schema, id)? else {
            return Err(anyhow!("artifact '{id}' not found"));
        };
        let old_record = ArtifactRecord::from_item(&existing_item)?;
        let from_scope = record_to_scope(&old_record)?;
        let old_rel_path = old_record.path.clone();

        // Acquire both scope locks in deterministic order (sorted by lock_key)
        // so two concurrent promotes between the same pair of scopes can't
        // deadlock. Same-scope promotes acquire one lock only.
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
        let _g2 = if let Some(l) = second_lock {
            Some(l.lock_owned().await)
        } else {
            None
        };

        // Read bytes from the record's stored path (may differ from the
        // id-derived path when this row was a dedup'd reference).
        let old_abs = self.global_dir.join(&old_rel_path);
        let bytes = std::fs::read(&old_abs)
            .with_context(|| format!("reading blob {}", old_abs.display()))?;

        let new_id = ship_substrate::gen_ulid();
        let content_hash = sha256_hex(&bytes);

        let local = LocalFsBackend::new(self.global_dir.clone());
        // Dedup at the target scope: if bytes already exist there, reuse
        // their path rather than writing a duplicate.
        let rel_path = match find_blob_by_hash(svc, &target_scope, &content_hash)? {
            Some(existing_path) => existing_path,
            None => {
                self.backend.put(&target_scope, &new_id, &bytes)?;
                local.rel_blob_path(&target_scope, &new_id)
            }
        };
        let mut new_data = existing_item.data.clone();
        new_data["path"] = serde_json::Value::String(rel_path);
        new_data["scope"] = serde_json::Value::String(target_scope.kind_str().to_string());
        new_data["content_hash"] = serde_json::Value::String(content_hash);
        // Clear scope-specific IDs then set the space-bearing target's ids.
        // The target is guaranteed space-scoped (Run/Project) — promote-to-global
        // (User, no space) is rejected above — so `user_id` stays cleared.
        new_data["space_id"] = serde_json::Value::Null;
        new_data["run_id"] = serde_json::Value::Null;
        new_data["user_id"] = serde_json::Value::Null;
        if let Some(pid) = target_scope.space_id() {
            new_data["space_id"] = serde_json::Value::String(pid.to_string());
        }
        if let Some(rid) = target_scope.run_id() {
            new_data["run_id"] = serde_json::Value::String(rid.to_string());
        }

        let owner = old_record.owner_actor.as_deref();
        let new_item = match collection_service::create_item(
            svc, kernel, &schema, target_scope.space_id(), &new_id, &new_data, owner,
        ).await {
            Ok(item) => item,
            Err(e) => {
                let _ = self.backend.delete(&target_scope, &new_id);
                return Err(e);
            }
        };

        // 3. Delete old metadata. Critical: a failure here leaves two rows
        // pointing at the same content with different scopes — silently
        // duplicated. Surface the error so the caller can retry.
        collection_service::delete_item(svc, kernel, ARTIFACTS_COLLECTION_ID, from_scope.space_id(), id)
            .await
            .with_context(|| format!("promote: deleting old metadata row for {id}"))?;

        // 4. Best-effort old-blob GC. Tolerated to fail: orphaned bytes can
        // be reclaimed by `collect_expired` / future GC. NotFound is fine
        // (already gone or never existed for a dedup'd row).
        if count_refs_to_path(svc, &from_scope, &old_rel_path)? == 0 {
            let abs = self.global_dir.join(&old_rel_path);
            match std::fs::remove_file(&abs) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => tracing::warn!(
                    artifact = id,
                    path = %abs.display(),
                    "promote: old blob removal failed (orphan, will GC): {e}"
                ),
            }
        }

        let new_record = ArtifactRecord::from_item(&new_item)?;
        emit_artifact_event(kernel, "artifact.moved", &new_id, target_scope.space_id(), serde_json::json!({
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
        items.iter().map(ArtifactRecord::from_item).collect()
    }
}

/// Reconstruct the scope from a metadata record.
fn record_to_scope(record: &ArtifactRecord) -> Result<ArtifactScope> {
    match record.scope.as_str() {
        "run" => {
            let space_id = record.space_id.clone()
                .ok_or_else(|| anyhow!("run-scope artifact missing space_id"))?;
            let run_id = record.run_id.clone()
                .ok_or_else(|| anyhow!("run-scope artifact missing run_id"))?;
            Ok(ArtifactScope::Run { space_id, run_id })
        }
        "project" => {
            let space_id = record.space_id.clone()
                .ok_or_else(|| anyhow!("project-scope artifact missing space_id"))?;
            Ok(ArtifactScope::Project { space_id })
        }
        "user" => {
            let user_id = record.user_id.clone()
                .ok_or_else(|| anyhow!("user-scope artifact missing user_id"))?;
            Ok(ArtifactScope::User { user_id })
        }
        other => Err(anyhow!(
            "artifact {} has invalid scope `{}` (expected run|project|user)",
            record.id, other
        )),
    }
}

/// Find an existing artifact blob in the same scope with a matching
/// content hash. Used for dedup: a second upload of identical bytes
/// into the same scope produces a new metadata row whose `path` points
/// at the original blob file rather than writing a duplicate.
fn find_blob_by_hash(
    svc: &mut DataService,
    scope: &ArtifactScope,
    content_hash: &str,
) -> Result<Option<String>> {
    use std::collections::BTreeMap;
    let mut filter = BTreeMap::new();
    filter.insert("scope".into(), serde_json::Value::String(scope.kind_str().to_string()));
    filter.insert("content_hash".into(), serde_json::Value::String(content_hash.to_string()));
    match scope {
        ArtifactScope::Run { space_id, run_id } => {
            filter.insert("space_id".into(), serde_json::Value::String(space_id.clone()));
            filter.insert("run_id".into(), serde_json::Value::String(run_id.clone()));
        }
        ArtifactScope::Project { space_id } => {
            filter.insert("space_id".into(), serde_json::Value::String(space_id.clone()));
        }
        ArtifactScope::User { user_id } => {
            filter.insert("user_id".into(), serde_json::Value::String(user_id.clone()));
        }
    }
    let req = QueryRequest {
        collection: ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort: BTreeMap::new(),
        limit: 1,
        offset: 0,
    };
    let schema = artifacts_schema();
    let items = collection_service::query_items(svc, &schema, &req)?;
    Ok(items
        .into_iter()
        .next()
        .and_then(|it| it.data.get("path").and_then(|v| v.as_str().map(str::to_string))))
}

/// Count how many metadata rows share a given (scope, rel_path). Used
/// before blob deletion to avoid yanking bytes another dedup'd row
/// still points at.
fn count_refs_to_path(
    svc: &mut DataService,
    scope: &ArtifactScope,
    rel_path: &str,
) -> Result<usize> {
    use std::collections::BTreeMap;
    let mut filter = BTreeMap::new();
    filter.insert("scope".into(), serde_json::Value::String(scope.kind_str().to_string()));
    filter.insert("path".into(), serde_json::Value::String(rel_path.to_string()));
    match scope {
        ArtifactScope::Run { space_id, run_id } => {
            filter.insert("space_id".into(), serde_json::Value::String(space_id.clone()));
            filter.insert("run_id".into(), serde_json::Value::String(run_id.clone()));
        }
        ArtifactScope::Project { space_id } => {
            filter.insert("space_id".into(), serde_json::Value::String(space_id.clone()));
        }
        ArtifactScope::User { user_id } => {
            filter.insert("user_id".into(), serde_json::Value::String(user_id.clone()));
        }
    }
    let req = QueryRequest {
        collection: ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort: BTreeMap::new(),
        limit: 100,
        offset: 0,
    };
    let schema = artifacts_schema();
    Ok(collection_service::query_items(svc, &schema, &req)?.len())
}

/// Fire a best-effort artifact event under the artifact's own space.
///
/// `artifact.*` are Space-scoped: a Run/Project artifact's metadata belongs to
/// (and syncs with) its space. The space comes from `ArtifactScope::space_id()`
/// (Run/Project) — bound on the daemon-internal mint so `enforce_scope` stamps
/// the correct partition. User-scoped (per-user) artifacts have no space; the
/// emit is best-effort and is dropped for them (they are not space content).
async fn emit_artifact_event(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    entity_id: &str,
    space: Option<&str>,
    payload: serde_json::Value,
) {
    let envelope = match EventEnvelope::new(event_type, entity_id, &payload) {
        Ok(e) => e.with_actor_id("service.artifacts"),
        Err(e) => {
            tracing::error!(event_type, entity_id, "artifacts: failed to build event: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space.map(ship_types::SpaceId::new));
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::error!(event_type, entity_id, "artifacts: event emission failed: {e}");
    }
}
