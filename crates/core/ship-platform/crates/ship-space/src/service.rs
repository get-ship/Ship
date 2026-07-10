//! Space service — core business logic for the SpaceService RPCs.
//!
//! All storage goes through `SpaceContext.data()` (DataStore port); all events
//! go through `SpaceContext.kernel()` (KernelLog port). No sqlite/sqlx/rusqlite
//! types and no raw fs here.
//!
//! The spoke model after the 2026-07-09 cutover:
//!
//! - **registry** is the `spaces` Runtime-scoped projection — the device-local
//!   record of opened spaces (LaunchServices analogy). Identity lives in the
//!   projection, never at the path. The single `Add` RPC is CREATE-new-at-
//!   location: it mints identity once (through `identity::ensure_identity_for_root`).
//! - **identity** is the served-space read: `Info` returns the runtime's
//!   served space's identity and name (today resolved through the live
//!   `ctx`'s `SpaceId`). Per-space settings are read through the unified
//!   `settings_overlay` (S4) — owned by `SettingsService`.
//!
//! `Activate` is GONE (multi-window kills the open-one-at-a-time world).
//! The active pointer table that supported it was retired; only
//! per-space projection rows remain.

use std::sync::Arc;

use ship_substrate::ports::SpaceContext;

use crate::error::SpaceError;
use crate::identity;
use crate::spaces;
use crate::types::{SpaceInfo, SpaceRecord};

/// Core service — stateless; all state lives in `SpaceContext`.
pub struct SpaceService {
    /// The ctx carries the served-space `SpaceId` (entity id) the runtime
    /// minted at boot via `identity::ensure_identity_for_root`. All reads of
    /// identity reach through this id; the wire layer wires the request's
    /// `SpaceContext` per-request so a future per-window model stays
    /// coherent.
    pub ctx: Arc<SpaceContext>,
}

impl SpaceService {
    pub fn new(ctx: Arc<SpaceContext>) -> Self {
        Self { ctx }
    }

    // ── opened-spaces list (Runtime-scoped) ─────────────────────────────────

    /// List every space currently open on this device.
    pub fn list(&self) -> Result<Vec<SpaceRecord>, SpaceError> {
        spaces::list(self.ctx.kernel())
    }

    // ── served-space identity (per-request) ──────────────────────────────────

    /// Compose the served-space identity for THIS request's ctx. Returns an
    /// empty identity when the ctx is the global/space-less Runtime slot
    /// (no served space) — the wire then emits an empty `SpaceInfo`.
    pub fn info(&self) -> Result<SpaceInfo, SpaceError> {
        // The per-request `SpaceContext` carries the served space's id via
        // its `SpaceId` slot. Globally-scoped requests (the Runtime
        // partition) carry an empty id; the wire returns an empty identity
        // so the FE never invents a `global` pseudo-space.
        let id = self.ctx.space_id().as_str();
        if id.is_empty() {
            Ok(SpaceInfo {
                space_id: None,
                root: None,
                name: None,
            })
        } else {
            // The served-space row lives in the Runtime-scoped `spaces`
            // projection. Fetch it so the Info reply carries the canonical
            // path + the projection's recorded name (the user-given name,
            // not the path-derived default).
            let rec = spaces::get(self.ctx.kernel(), id)?;
            Ok(match rec {
                Some(r) => SpaceInfo {
                    space_id: Some(r.space_id),
                    root: Some(r.path),
                    name: Some(r.name),
                },
                None => SpaceInfo {
                    space_id: Some(id.to_string()),
                    root: None,
                    name: None,
                },
            })
        }
    }

    // ── add (CREATE-new-at-location) ───────────────────────────────────────

    /// CREATE a new space at the location `path`. Mints identity once (an
    /// already-open path re-emits `space.seen`; a fresh one mints a
    /// nanoid + emits `space.opened`). Idempotent: re-opening an existing
    /// space returns its existing identity with a bumped `last_seen_at`.
    pub async fn add(
        &self,
        path: &std::path::Path,
        name: Option<&str>,
    ) -> Result<SpaceRecord, SpaceError> {
        identity::ensure_identity_for_root(self.ctx.kernel(), path, name).await
    }

    // ── snapshot ────────────────────────────────────────────────────────────
    //
    // Resolves `space_id` (the request's explicit id, or the served space's
    // id when the request leaves it empty — the "active/served space"
    // fallback the proto documents), validates `target_dir`, and delegates
    // to the substrate's `snapshot_space` to write the bundle. The wire
    // layer is the only caller; the helper here exists so the resolution
    // rule lives next to the rest of the service logic.
    pub async fn snapshot(
        &self,
        space_id: &str,
        target_dir: &std::path::Path,
    ) -> Result<ship_substrate::SnapshotResult, SpaceError> {
        let resolved = resolve_space_id(self.ctx.space_id().as_str(), space_id)?;
        // target_dir must be an absolute, existing directory — the drill
        // spec calls this out (typed invalid-argument otherwise).
        if !target_dir.is_absolute() {
            return Err(SpaceError::BadArgs(format!(
                "target_dir must be an absolute path: {}",
                target_dir.display()
            )));
        }
        if !target_dir.is_dir() {
            return Err(SpaceError::BadArgs(format!(
                "target_dir does not exist or is not a directory: {}",
                target_dir.display()
            )));
        }
        let data_root = ship_substrate::project::get_global_dir().map_err(|e| {
            SpaceError::Internal(format!("resolve ship data root: {e}"))
        })?;
        ship_substrate::snapshot_space(&data_root, &resolved, target_dir)
            .map_err(|e| SpaceError::Internal(format!("snapshot: {e}")))
    }
}

/// Pick the effective space id: the request's `space_id` when present, else
/// the served space from the request's `SpaceContext`. Both empty is an
/// invalid request — a snapshot with no target space has no meaning.
fn resolve_space_id(served: &str, requested: &str) -> Result<String, SpaceError> {
    if !requested.is_empty() {
        return Ok(requested.to_string());
    }
    if !served.is_empty() {
        return Ok(served.to_string());
    }
    Err(SpaceError::BadArgs(
        "space_id is empty and the request has no served space; specify one".to_string(),
    ))
}
