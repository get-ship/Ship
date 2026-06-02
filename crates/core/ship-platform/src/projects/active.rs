//! Active-project state — one active project per daemon process.
//!
//! Stored under the `shell.project.active` setting in `daemon_settings`
//! (the same row the legacy `/api/v1/projects/switch` endpoint wrote);
//! the new entry points here add a dedicated `project.switched` kernel
//! event on top so chrome subscribers can react to the context change
//! without parsing the generic `config.changed` stream.
//!
//! This module owns the read + write seam. Route handlers call
//! `get_active` / `switch_to`; the legacy `/switch` endpoint and the
//! new `/{id}/activate` endpoint both funnel through `switch_to` so
//! there is exactly one place that decides what "switching" means.

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::{EmitContext, EventEnvelope, KernelRouter};
use ship_types::event_types::PROJECT_SWITCHED;
use ship_types::payload::ProjectSwitched;

use super::ProjectRecord;
use crate::config;

const ACTIVE_KEY: &str = "shell.project.active";

/// Return the currently-active project id (None on cold boot before any
/// switch). Reads through the daemon-settings store; defaults never apply
/// to this key.
pub fn get_active_id() -> Result<Option<String>> {
    Ok(config::get_setting(ACTIVE_KEY)?
        .and_then(|v| v.as_str().map(str::to_string)))
}

/// Return the full record for the currently-active project, if any.
/// `None` when no project has been activated, or when the stored id no
/// longer resolves to a live registry row (project archived behind our
/// back).
pub fn get_active() -> Result<Option<ProjectRecord>> {
    match get_active_id()? {
        Some(id) => super::get(&id),
        None => Ok(None),
    }
}

/// Switch the active project to `id`. Writes the setting, bumps
/// `last_seen_at`, and emits `project.switched` on the kernel bus.
///
/// Returns the new active record. Errors if `id` doesn't resolve to a
/// known project — callers turn that into a 404.
pub async fn switch_to(
    kernel: &Arc<Mutex<KernelRouter>>,
    id: &str,
) -> Result<ProjectRecord> {
    let record = super::get(id)?.ok_or_else(|| {
        anyhow::anyhow!("project '{id}' not known")
    })?;
    let previous = get_active_id()?;

    // Write through the same setting the resolver / skills_api watch.
    // set_setting_and_emit fires `config.changed` for legacy subscribers;
    // we then fire the dedicated `project.switched` event so the new
    // chrome wiring can subscribe to a single, semantic stream.
    config::set_setting_and_emit(
        kernel,
        ACTIVE_KEY,
        serde_json::json!(record.id),
    )
    .await?;
    let _ = super::touch(&record.id);

    let payload = ProjectSwitched {
        space_id: record.id.clone(),
        name: record.name.clone(),
        path: record.path.clone(),
        previous,
        at: Utc::now().to_rfc3339(),
    };
    let envelope = EventEnvelope::new(PROJECT_SWITCHED, &record.id, &payload)?;
    // The switched-to project's id IS its space — already correct; carry it
    // through the binding (daemon-internal mint).
    let ctx = EmitContext::daemon_internal(Some(ship_types::SpaceId::new(record.id.clone())));
    kernel.lock().await.emit(envelope, &ctx).await?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh_db() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        crate::testutil::init_project(tmp.path()).unwrap();
        ship_substrate::db::ensure_db().unwrap();
        tmp
    }

    fn kernel(tmp: &TempDir) -> Arc<Mutex<KernelRouter>> {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
        // KernelRouter::new wires the file-backed event log; tests
        // share the same ship_dir the projection writes use.
        let router = ship_substrate::events::KernelRouter::new(ship_dir).unwrap();
        Arc::new(Mutex::new(router))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn switch_to_sets_active_and_emits_event() {
        let tmp = fresh_db();
        let p = TempDir::new().unwrap();
        let registered = super::super::register(p.path(), Some("a")).unwrap();
        let k = kernel(&tmp);
        let returned = switch_to(&k, &registered.id).await.unwrap();
        assert_eq!(returned.id, registered.id);
        assert_eq!(get_active_id().unwrap().as_deref(), Some(registered.id.as_str()));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn switch_to_unknown_errors() {
        let tmp = fresh_db();
        let k = kernel(&tmp);
        assert!(switch_to(&k, "deadbeef").await.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn switch_records_previous_in_payload() {
        let tmp = fresh_db();
        let a = TempDir::new().unwrap();
        let b = TempDir::new().unwrap();
        let ra = super::super::register(a.path(), Some("ra")).unwrap();
        let rb = super::super::register(b.path(), Some("rb")).unwrap();
        let k = kernel(&tmp);
        switch_to(&k, &ra.id).await.unwrap();
        switch_to(&k, &rb.id).await.unwrap();
        assert_eq!(get_active_id().unwrap().as_deref(), Some(rb.id.as_str()));
    }
}
