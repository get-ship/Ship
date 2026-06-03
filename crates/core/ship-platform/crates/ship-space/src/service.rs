//! Space service — core business logic for the SpaceV2Service RPCs.
//!
//! All storage goes through `SpaceContext.data()` (DataStore port); all events
//! go through `SpaceContext.kernel()` (KernelLog port). No sqlite/sqlx/rusqlite
//! types and no raw fs here.
//!
//! Scope split: the registry (`projects` + active pointer) is Runtime-scoped, so
//! `ctx` binds the Runtime store (runtime.db) — it owns list/active/add/activate
//! and is the kernel handle for every emit. The per-space settings overlay
//! (`space_settings`) is Space-scoped, so `info` + `set_setting` take an explicit
//! per-space `&dyn DataStore` the caller (the daemon's wire) builds per request
//! from the target space id.

use std::sync::Arc;

use serde_json::Value;

use ship_substrate::ports::{DataStore, SpaceContext};

use crate::error::SpaceError;
use crate::types::{SettingValue, SpaceInfo, SpaceRecord};
use crate::{config, info, registry};

/// Core service — stateless; all state lives in `SpaceContext`.
pub struct SpaceService {
    /// Runtime-scoped context: the `projects` registry + active pointer store,
    /// and the kernel handle for every `project.*` / `space.setting.changed`
    /// emit.
    pub ctx: Arc<SpaceContext>,
}

impl SpaceService {
    pub fn new(ctx: Arc<SpaceContext>) -> Self {
        Self { ctx }
    }

    // ── registry (Runtime-scoped) ───────────────────────────────────────────

    /// List all live (non-archived) spaces, newest first.
    pub async fn list(&self) -> Result<Vec<SpaceRecord>, SpaceError> {
        registry::list(self.ctx.data()).await
    }

    /// The currently-active space, or `None` on cold boot.
    pub async fn active(&self) -> Result<Option<SpaceRecord>, SpaceError> {
        registry::active(self.ctx.data()).await
    }

    /// The active-space id, or `None` on cold boot (used by `list` to surface
    /// the active pointer alongside the registry).
    pub async fn active_id(&self) -> Result<Option<String>, SpaceError> {
        Ok(crate::projection::get_active_id(self.ctx.data()).await?)
    }

    /// Register a directory as a space (idempotent). See [`registry::add`].
    pub async fn add(
        &self,
        path: &std::path::Path,
        name: Option<&str>,
    ) -> Result<SpaceRecord, SpaceError> {
        registry::add(self.ctx.data(), self.ctx.kernel(), path, name).await
    }

    /// Switch the active space. See [`registry::activate`].
    pub async fn activate(&self, id: &str) -> Result<SpaceRecord, SpaceError> {
        registry::activate(self.ctx.data(), self.ctx.kernel(), id).await
    }

    // ── config (Space-scoped) ───────────────────────────────────────────────

    /// Write a per-space settings override against the per-space `settings_store`.
    /// Emits `space.setting.changed` through the runtime kernel handle (the space
    /// rides the envelope, so the kernel routes it to the space's `space.db`).
    pub async fn set_setting(
        &self,
        settings_store: &dyn DataStore,
        space_id: &str,
        key: &str,
        raw_value_json: &str,
    ) -> Result<SettingValue, SpaceError> {
        config::set(settings_store, self.ctx.kernel(), space_id, key, raw_value_json).await
    }

    // ── info (identity + overlay) ───────────────────────────────────────────

    /// Compose identity + effective (global ⟵ space) settings. Identity comes
    /// from the Runtime registry store; the overlay from the per-space
    /// `settings_store`. See [`info::compose`].
    pub async fn info(
        &self,
        settings_store: &dyn DataStore,
        space_id: Option<&str>,
        global_settings: Value,
    ) -> Result<SpaceInfo, SpaceError> {
        info::compose(self.ctx.data(), settings_store, space_id, global_settings).await
    }
}
