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
