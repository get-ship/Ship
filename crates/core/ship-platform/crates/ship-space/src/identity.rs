//! Identity — the SINGLE mint-once accessor for a space's `space_id`.
//!
//! The ruled model: a space owns its identity, never the path. To open a
//! directory as a space (or to attach to one the runtime has opened before),
//! every caller MUST reach for [`ensure_identity_for_root`]. This function is
//! the only place in the codebase that mints a fresh `space_id`; identity's
//! exact on-disk home is the `spaces` Runtime-scoped projection (folded from
//! `space.opened` events by `spaces_projection::SpacesProjection`).
//!
//! The "fenced space-config session" naming in the spec refers to a future
//! plan to relocate identity to a per-space manifest or signed file under
//! the space's own tree. When that lands, the change happens HERE — every
//! caller reaches this single accessor, so a relocation touches one module.
//!
//! Behaviour:
//!
//! - If the runtime has opened the canonical path before, return the
//!   existing row's `space_id` and emit a `space.seen` (bumps
//!   `last_seen_at`) so the projection stays fresh on every boot.
//! - Otherwise mint a fresh `space_id` (via the daemon's local
//!   [`ship_substrate::gen_nanoid`]), insert it via `space.opened`, and
//!   return the new id. A subsequent `ensure_identity_for_root` with the
//!   same path returns the same id (idempotent).
//!
//! On every successful call the produced row materialises in the `spaces`
//! projection through the per-scope bus's same-TX guarantee — a duplicate
//! mint is impossible.

use std::path::Path;

use ship_substrate::gen_nanoid;
use ship_substrate::ports::KernelLog;
use ship_substrate::events::validator::EmitContext;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{SPACE_OPENED, SPACE_SEEN};
use ship_types::payload::{SpaceOpened, SpaceSeen};

use crate::error::SpaceError;
use crate::helpers::{now_rfc3339, runtime_emit_ctx};
use crate::types::SpaceRecord;

/// Ensure identity for `root`: read the existing `spaces` row for the
/// canonical path, or mint a fresh `space_id` + emit `space.opened`. The
/// returned `SpaceRecord` reflects the post-emit projection state (with the
/// fresh `opened_at` on mint and a bumped `last_seen_at` on re-open).
///
/// `root` is the project's canonical filesystem path. The runtime canonicalises
/// it (via `std::fs::canonicalize` with a silent fallback to the raw path —
/// the L0 read-side canonicalization at `project::resolve_active_space` does
/// the same, so both sides agree on the storage key).
pub async fn ensure_identity_for_root(
    kernel: &KernelLog,
    root: &Path,
    name: Option<&str>,
) -> Result<SpaceRecord, SpaceError> {
    let canonical = std::fs::canonicalize(root)
        .unwrap_or_else(|_| root.to_path_buf())
        .to_string_lossy()
        .into_owned();
    let now = now_rfc3339();

    // Re-open: a row already exists for this canonical path. Just bump the
    // `last_seen_at` via `space.seen` (the projection folds it into the
    // existing row inside the same kernel transaction).
    if let Some(existing) = crate::spaces::find_by_path(kernel, Path::new(&canonical))? {
        let seen_env = EventEnvelope::new(
            SPACE_SEEN,
            &existing.space_id,
            &SpaceSeen { seen_at: now.clone() },
        )
        .map_err(|e| SpaceError::Internal(e.to_string()))?;
        kernel.emit(seen_env, &runtime_emit_ctx()).await?;
        return Ok(SpaceRecord {
            last_seen_at: now,
            ..existing
        });
    }

    // Mint. The nanoid is the space's identity forever — never derived from
    // the path, never re-derived. A user-supplied `name` overrides the
    // path-derived default (the final path segment); the projection records
    // both the canonical path AND the explicit name so a move (`space.moved`
    // event, future) keeps the user-given label.
    let space_id = gen_nanoid();
    let resolved_name = name
        .map(str::to_string)
        .unwrap_or_else(|| derive_name_from_path(Path::new(&canonical)));

    let payload = SpaceOpened {
        path: canonical.clone(),
        name: resolved_name.clone(),
        opened_at: now.clone(),
    };
    let envelope = EventEnvelope::new(SPACE_OPENED, &space_id, &payload)
        .map_err(|e| SpaceError::Internal(e.to_string()))?;
    let ctx: EmitContext = runtime_emit_ctx();
    kernel.emit(envelope, &ctx).await?;

    Ok(SpaceRecord {
        space_id,
        path: canonical,
        name: resolved_name,
        opened_at: now.clone(),
        last_seen_at: now,
    })
}

fn derive_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("space")
        .to_string()
}
