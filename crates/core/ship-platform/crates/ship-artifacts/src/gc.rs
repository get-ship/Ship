//! Garbage collection for expired run-scope artifacts.
//!
//! `collect_expired` deletes run-scope artifacts where `expires_at < now`.
//! Triggered on daemon startup and periodically by the supervisor.

use std::collections::BTreeMap;
use std::sync::Arc;

use ship_substrate_core::ports::SpaceContext;

use crate::error::ArtifactError;
use crate::projection::query_artifacts;
use crate::service::ArtifactService;

/// Delete all expired run-scope artifacts. Returns the count GC'd.
pub async fn collect_expired(svc: &ArtifactService) -> Result<usize, ArtifactError> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut filter = BTreeMap::new();
    filter.insert("scope".into(), serde_json::Value::String("run".into()));

    let expired: Vec<_> = query_artifacts(svc.ctx.data(), filter, 500, None)
        .await?
        .into_iter()
        .filter(|r| {
            r.expires_at.as_deref().map(|e| e <= now.as_str()).unwrap_or(false)
        })
        .collect();

    let count = expired.len();
    for record in &expired {
        if let Err(e) = svc.delete(&record.id).await {
            tracing::warn!(id = %record.id, "artifact GC: delete failed: {e}");
        }
    }
    if count > 0 {
        tracing::info!(count, "artifact GC: removed {count} expired run-scope artifact(s)");
    }
    Ok(count)
}

/// Build an `ArtifactService` context for GC sweeps (used by external callers
/// that only have a `SpaceContext` + global dir, not a full service instance).
pub fn make_service_for_gc(
    ctx: Arc<SpaceContext>,
    global_dir: std::path::PathBuf,
) -> ArtifactService {
    ArtifactService::new(ctx, global_dir)
}
