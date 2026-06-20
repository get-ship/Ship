//! Space service — core business logic for the SpaceService RPCs.
//!
//! All storage goes through `SpaceContext.data()` (DataStore port); all events
//! go through `SpaceContext.kernel()` (KernelLog port). No sqlite/sqlx/rusqlite
//! types and no raw fs here.
//!
//! Scope split: the registry (`projects` + active pointer) is Runtime-scoped, so
//! `ctx` binds the Runtime store (runtime.db) — it owns list/active/add/activate
//! and is the kernel handle for every emit. `info` is identity-only; per-space
//! settings live on the unified `settings_overlay` (S4) and are owned by
//! `SettingsService`.
//!
//! After the dual-read-model dissolution (2026-06-10), record reads go through
//! `registry::list/get` which use `kernel.try_ship_dir()` to open the `projects`
//! SQL table; only the active pointer still uses `ctx.data()`.

use std::sync::Arc;

use ship_substrate::ports::SpaceContext;

use crate::error::SpaceError;
use crate::types::{SpaceInfo, SpaceRecord};
use crate::{info, registry};

/// Core service — stateless; all state lives in `SpaceContext`.
pub struct SpaceService {
    /// Runtime-scoped context: the `projects` registry + active pointer store,
    /// and the kernel handle for every `project.*` emit.
    pub ctx: Arc<SpaceContext>,
}

impl SpaceService {
    pub fn new(ctx: Arc<SpaceContext>) -> Self {
        Self { ctx }
    }

    // ── registry (Runtime-scoped) ───────────────────────────────────────────

    /// List all live (non-archived) spaces, newest first.
    pub async fn list(&self) -> Result<Vec<SpaceRecord>, SpaceError> {
        registry::list(self.ctx.kernel()).await
    }

    /// The currently-active space, or `None` on cold boot.
    pub async fn active(&self) -> Result<Option<SpaceRecord>, SpaceError> {
        registry::active(self.ctx.data(), self.ctx.kernel()).await
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
        registry::add(self.ctx.kernel(), path, name).await
    }

    /// Switch the active space. See [`registry::activate`].
    pub async fn activate(&self, id: &str) -> Result<SpaceRecord, SpaceError> {
        registry::activate(self.ctx.data(), self.ctx.kernel(), id).await
    }

    // ── info (identity only) ────────────────────────────────────────────────

    /// Compose identity (id/root/name from `projects`). Per-space settings
    /// moved to `SettingsService` (S4). See [`info::compose`].
    pub async fn info(
        &self,
        space_id: Option<&str>,
    ) -> Result<SpaceInfo, SpaceError> {
        info::compose(self.ctx.kernel(), space_id).await
    }
}
