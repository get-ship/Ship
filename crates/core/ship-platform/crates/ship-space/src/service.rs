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
}
