//! Declarative-tier event authorship — the config SERVICE (not the OS watcher)
//! names the domain `event_type`, sets `subject`/`entity_id`, and mints the
//! scope binding.
//!
//! THE EVICTION: the pre-carve `config/watcher.rs` content-hashed a changed file
//! with raw `std::fs::read`. Here the hash read goes through the [`FsBroker`]
//! gated door (empty hash on read failure = deleted marker, behavior preserved).
//!
//! The inotify primitive itself (the substrate `FsWatch` port that wraps
//! `notify` and feeds a debounced delta stream) is a collaborator OUT OF SCOPE
//! for this crate — config subscribes domains against it, it does not absorb it.
//! This module owns only the authorship the watch drives, which is what the
//! keystone test exercises directly with a synthetic signal (no inotify, not
//! flaky).

use std::path::Path;
use std::sync::Arc;

use ship_substrate::events::validator::EmitContext;
use ship_substrate::ports::{FsBroker, FsCap, FsMode, FsRoot, KernelLog};
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::{AgentDefUpdated, ConfigChanged, ConfigFileChanged};

/// Author the coarse `<domain>.changed` config event with the SAME `event_type`
/// / `subject` / `entity_id` the watcher used to emit, so config reactivity
/// (compile invalidation, Studio) is unbroken.
///
/// `<domain>.changed` events are Space-scoped (registry); when `space` is
/// `Some`, the kernel sink routes the event to the bound space's db.
pub async fn author_change(
    kernel: &Arc<KernelLog>,
    event_type: &str,
    subject: &str,
    space: Option<SpaceId>,
) {
    let payload = ConfigChanged { subject: subject.to_string(), detail: None };
    let envelope = match EventEnvelope::new(event_type, subject, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config: failed to build {event_type} envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space);
    if let Err(e) = kernel.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit {event_type}: {e}");
    }
}

/// Author the fine-grained `config.file_changed` for one changed file: the path
/// relative to the watched domain dir + its content hash (empty when removed).
///
/// `config.file_changed` is Runtime-scoped → it lands in runtime.db. The content
/// hash is read through `fs` confined by a cap rooted at `dir`; a read failure
/// yields an empty hash = a valid "deleted" marker (behavior preserved from the
/// raw-`std::fs::read` predecessor).
pub async fn author_file_changed(
    kernel: &Arc<KernelLog>,
    fs: &dyn FsBroker,
    domain: &str,
    dir: &Path,
    path: &Path,
    space: Option<SpaceId>,
) {
    let rel = path.strip_prefix(dir).unwrap_or(path).to_string_lossy().to_string();
    let cap = FsCap::new_unchecked(
        dir.to_path_buf(),
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        FsMode::ReadOnly,
    );
    // Empty hash signals a removed file (read fails) — a valid "deleted" marker.
    let content_hash = fs
        .read(&cap, &rel)
        .map(|bytes| ship_types::sha256_hex(&bytes))
        .unwrap_or_default();
    let payload = ConfigFileChanged {
        domain: domain.to_string(),
        path: rel.clone(),
        content_hash,
    };
    let envelope = match EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, &rel, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config: failed to build config.file_changed envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space);
    if let Err(e) = kernel.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit config.file_changed: {e}");
    }
}

/// Author the fine-grained `agent.def.updated` notification for one changed agent
/// def file: the rel `path` under `.ship/agents/` + its `content_hash` (empty
/// when removed). The Format-2, space-stamped, content-hashed successor to the
/// coarse `agent.changed` for the agents domain.
///
/// `agent.def.updated` is Space-scoped → the def lives in a space's `.ship`, so
/// the kernel routes it to the bound space's db. The content hash is read through
/// `fs` confined by a cap rooted at `dir`; a read failure yields an empty hash =
/// a "deleted" marker the read-only index uses to drop the row. The notification
/// NEVER carries file content (Bucket-2 invariant) — the index re-reads on it.
pub async fn author_agent_def_updated(
    kernel: &Arc<KernelLog>,
    fs: &dyn FsBroker,
    dir: &Path,
    path: &Path,
    space: Option<SpaceId>,
) {
    let rel = path.strip_prefix(dir).unwrap_or(path).to_string_lossy().to_string();
    let cap = FsCap::new_unchecked(
        dir.to_path_buf(),
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        FsMode::ReadOnly,
    );
    let content_hash = fs
        .read(&cap, &rel)
        .map(|bytes| ship_types::sha256_hex(&bytes))
        .unwrap_or_default();
    let payload = AgentDefUpdated { path: rel.clone(), content_hash };
    let envelope = match EventEnvelope::new(event_types::AGENT_DEF_UPDATED, &rel, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config: failed to build agent.def.updated envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space);
    if let Err(e) = kernel.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit agent.def.updated: {e}");
    }
}
