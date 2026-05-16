//! Lazy-init helper for the shared `DataService` handle.
//!
//! CollectionsService and ArtifactsService share the same
//! `Arc<Mutex<Option<DataService>>>` handle so cross-service reads see
//! the same lazily-opened DataService. This helper opens it once on
//! first access and registers the built-in artifacts collection so the
//! registration happens exactly once regardless of who holds the
//! mutex first.

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use ship_substrate::data::DataService;

pub async fn ensure_data_service<'a>(
    handle: &'a Arc<Mutex<Option<DataService>>>,
    ship_dir: &Path,
) -> Result<MutexGuard<'a, Option<DataService>>> {
    let mut guard = handle.lock().await;
    if guard.is_none() {
        let mut svc = DataService::open(ship_dir)?;
        crate::artifacts::ensure_artifacts_collection(&mut svc)
            .context("registering built-in artifacts collection")?;
        crate::view_authoring::ensure_view_snapshots_collection(&mut svc)
            .context("registering built-in view_snapshots collection")?;
        *guard = Some(svc);
    }
    Ok(guard)
}
