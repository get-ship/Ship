//! Garbage collection for expired run-scope artifacts.
//!
//! Triggered on daemon startup and every hour by the supervisor. Deletes blobs
//! and metadata rows for run-scope artifacts where `expires_at < now`.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::service::ArtifactService;
use crate::collections::query::QueryRequest;
use crate::data::DataService;
use crate::events::KernelRouter;

/// Delete all expired run-scope artifacts. Returns the number of artifacts GC'd.
pub async fn collect_expired(
    global_ship_dir: &Path,
    svc: &mut DataService,
    kernel: &Arc<Mutex<KernelRouter>>,
) -> Result<usize> {
    let now = chrono::Utc::now().to_rfc3339();

    // Query for run-scope rows with an expires_at in the past.
    let mut filter: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    filter.insert("scope".into(), serde_json::Value::String("run".into()));
    filter.insert("expires_at__lte".into(), serde_json::Value::String(now.clone()));

    let req = QueryRequest {
        collection: crate::artifacts::ARTIFACTS_COLLECTION_ID.to_string(),
        filter,
        sort: BTreeMap::new(),
        limit: 500,
        offset: 0,
    };

    let service = ArtifactService::new(global_ship_dir);
    let expired = service.list(svc, req)?;
    let count = expired.len();

    for record in &expired {
        if let Err(e) = service.delete(svc, kernel, &record.id).await {
            tracing::warn!(id = %record.id, "artifact GC: delete failed: {e}");
        }
    }

    if count > 0 {
        tracing::info!(count, "artifact GC: removed {count} expired run-scope artifact(s)");
    }

    Ok(count)
}

/// Rehash blobs for legacy metadata rows that predate the content_hash
/// column. Runs once on daemon startup. Idempotent: rows with a populated
/// content_hash are skipped.
pub fn backfill_content_hashes(global_ship_dir: &Path, svc: &mut DataService) -> Result<usize> {
    use crate::artifacts::backend::sha256_hex;
    use crate::data::SqlParam;

    crate::artifacts::ensure_artifacts_collection(svc)?;

    let rows = svc.query(
        "SELECT id, json_extract(data, '$.path') AS path FROM collection_artifacts \
         WHERE json_extract(data, '$.content_hash') IS NULL",
        &[],
    )?;

    let mut updated = 0usize;
    for row in rows {
        let Some(id) = row.get_str("id") else { continue };
        let Some(rel_path) = row.get_str("path") else { continue };
        let abs = global_ship_dir.join(rel_path);
        let bytes = match std::fs::read(&abs) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(id, path = %abs.display(), "backfill: blob unreadable: {e}");
                continue;
            }
        };
        let hash = sha256_hex(&bytes);
        if let Err(e) = svc.execute(
            "UPDATE collection_artifacts \
             SET data = json_set(data, '$.content_hash', ?) \
             WHERE id = ?",
            &[SqlParam::from(hash.as_str()), SqlParam::from(id)],
        ) {
            tracing::warn!(id, "backfill: update failed: {e}");
            continue;
        }
        updated += 1;
    }

    if updated > 0 {
        tracing::info!(updated, "artifact backfill: populated content_hash on {updated} legacy row(s)");
    }
    Ok(updated)
}
