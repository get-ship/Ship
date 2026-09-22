//! THE agent-def WRITE door — the one file-access door for authoring
//! `.ship/agents/<id>.jsonc` def files. Mirrors the S2 reader ([`crate::agents`])
//! on the write side.
//!
//! All disk flows through the [`FsBroker`] gated door, confined by an [`FsCap`]
//! ([`scope_cap`], `FsMode::ReadWrite`). Zero `std::fs`/`tokio::fs` on the agent-
//! def write path (grep-gate invariant). Two granularities share the one door:
//! - the **per-agent opaque-value CRUD** ([`write_agent_def`] / [`delete_agent_def`]
//!   / [`rename_agent_def`] / [`read_agent_def_value`] / [`find_agent_def`]) — what
//!   the daemon's `AgentsService` state layer (S5b) routes every create/put/patch/
//!   delete/rename/duplicate/move-scope through; and
//! - the **typed flat-profile BATCH** ([`save_agents_config`]) — what `config/io.rs`
//!   reconciles a whole project's `agents` set through. Both preserve the hand-
//!   authored canonical fields (`hooks`, `plugins`, `rules.inline`,
//!   `harness_settings`, `apps`) on re-save.
//!
//! Delete is trash-not-hard-delete: a removed def is relocated into the sibling
//! `<scope>/.trash/` (recoverable), via the [`FsBroker::rename`] move primitive.
//! The post-write NOTIFY ([`notify_agent_def_saved`], S5) fires at the gate so
//! the read-only index stays live.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ship_substrate::events::EmitContext;
use ship_substrate::ports::{FsBroker, FsCap, FsError, FsMode, FsRoot, KernelLog};
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::AgentDefUpdated;

use crate::agents::{agent_rel, agents_cap, scope_cap, under_prefix};
use crate::error::ConfigError;
use crate::types::AgentProfile;
/// Accepted def extensions, canonical first. `.json` is tolerated so a
/// hand-authored plain-JSON def still resolves on read/locate/delete.
const DEF_EXTS: &[&str] = &["jsonc", "json"];

// ── Per-agent opaque-value door (THE agent-def write surface) ────────────────────
//
// These functions are the ONE write/locate/read-for-write path for a single
// `.ship/agents/<id>.jsonc` def. Every disk op flows through the [`FsBroker`]
// gated door, confined by [`scope_cap`] to the SCOPE dir (project `.ship` or
// user `library`/`.ship`) so the trash relocation (`<scope>/.trash/`) stays
// inside the same cap. Docs are opaque `serde_json::Value` — the CRUD layer is
// shape-agnostic, exactly like the legacy raw-fs path it replaces. Zero
// `std::fs`/`tokio::fs` (grep-gate invariant).

/// Locate the def for `id` under `<scope>/agents/`, returning the matched
/// agents-relative filename (`<id>.jsonc` or `<id>.json`). `Ok(None)` when no
/// file is present (the absent/deleted case).
pub fn find_agent_def(
    fs: &dyn FsBroker,
    scope_dir: &Path,
    id: &str,
) -> Result<Option<String>, ConfigError> {
    let (cap, prefix) = scope_cap(scope_dir, FsMode::ReadOnly);
    for ext in DEF_EXTS {
        let leaf = format!("{id}.{ext}");
        let rel = under_prefix(&prefix, &format!("agents/{leaf}"));
        if fs.exists(&cap, &rel)? {
            return Ok(Some(leaf));
        }
    }
    Ok(None)
}

/// Read + parse a def's opaque value, paired with the matched agents-relative
/// filename. `Ok(None)` when no file is present. Errors only on real I/O; a
/// malformed file surfaces a [`ConfigError::Validation`] so a read-modify-write
/// (patch) never silently drops content.
pub fn read_agent_def_value(
    fs: &dyn FsBroker,
    scope_dir: &Path,
    id: &str,
) -> Result<Option<(String, serde_json::Value)>, ConfigError> {
    let (cap, prefix) = scope_cap(scope_dir, FsMode::ReadOnly);
    for ext in DEF_EXTS {
        let leaf = format!("{id}.{ext}");
        let rel = under_prefix(&prefix, &format!("agents/{leaf}"));
        match fs.read_string(&cap, &rel) {
            Ok(raw) => {
                let stripped = ship_substrate::fs_util::strip_jsonc_comments(&raw);
                let value = serde_json::from_str(&stripped)
                    .map_err(|e| ConfigError::Validation(e.to_string()))?;
                return Ok(Some((leaf, value)));
            }
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(None)
}

/// Write `doc` to `<scope>/agents/<id>.jsonc` atomically. With
/// `allow_overwrite == false` the write refuses an existing def in EITHER
/// extension and returns [`ConfigError::AlreadyExists`] (the create contract);
/// PUT/upsert callers pass `true`. Creates the agents dir on first write.
pub fn write_agent_def(
    fs: &dyn FsBroker,
    scope_dir: &Path,
    id: &str,
    doc: &serde_json::Value,
    allow_overwrite: bool,
) -> Result<(), ConfigError> {
    if !allow_overwrite && find_agent_def(fs, scope_dir, id)?.is_some() {
        return Err(ConfigError::AlreadyExists(id.to_string()));
    }
    let (cap, prefix) = scope_cap(scope_dir, FsMode::ReadWrite);
    fs.create_dir_all(&cap, &under_prefix(&prefix, "agents"))?;
    let body =
        serde_json::to_string_pretty(doc).map_err(|e| ConfigError::Internal(e.to_string()))?;
    let rel = under_prefix(&prefix, &format!("agents/{id}.jsonc"));
    fs.write_atomic(&cap, &rel, body.as_bytes())?;
    Ok(())
}

/// Soft-delete the def for `id`: relocate `<scope>/agents/<id>.<ext>` to
/// `<scope>/.trash/<unix_secs>-<id>.<ext>` (trash-not-hard-delete, the semantics
/// the legacy `FilesService::trash` provided). [`ConfigError::NotFound`] when no
/// file is present.
pub fn delete_agent_def(fs: &dyn FsBroker, scope_dir: &Path, id: &str) -> Result<(), ConfigError> {
    let Some(leaf) = find_agent_def(fs, scope_dir, id)? else {
        return Err(ConfigError::NotFound(id.to_string()));
    };
    let (cap, prefix) = scope_cap(scope_dir, FsMode::ReadWrite);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let src = under_prefix(&prefix, &format!("agents/{leaf}"));
    let dst = under_prefix(&prefix, &format!(".trash/{ts}-{leaf}"));
    fs.rename(&cap, &src, &dst)?;
    Ok(())
}

/// Rename a def within a scope: write `doc` to `<new_id>.jsonc` (refusing an
/// existing `new_id`) then trash `<old_id>`. The caller supplies the doc already
/// reconciled to `new_id` (so `agent.id` tracks the filename). The two-step
/// (create-new + trash-old) keeps the old content recoverable in `.trash/`,
/// matching the legacy rename.
pub fn rename_agent_def(
    fs: &dyn FsBroker,
    scope_dir: &Path,
    old_id: &str,
    new_id: &str,
    doc: &serde_json::Value,
) -> Result<(), ConfigError> {
    write_agent_def(fs, scope_dir, new_id, doc, false)?;
    delete_agent_def(fs, scope_dir, old_id)?;
    Ok(())
}

/// Strip the optional `NNNN-` numeric prefix and `.md` suffix from a rule
/// reference so callers see canonical IDs (`rust` rather than `0001-rust.md`).
pub fn normalize_rule_external_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

/// Write each profile to `<id>.jsonc`, pruning files for agents no longer in the
/// desired set. Preserves hand-authored canonical fields on re-save. Writes via
/// [`FsBroker`] — never `std::fs`.
pub fn save_agents_config(
    fs: &dyn FsBroker,
    ship_dir: &Path,
    agents: &[AgentProfile],
) -> Result<(), ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadWrite);
    fs.create_dir_all(&cap, &agents_dir)?;

    let keep: HashSet<String> = agents.iter().map(|a| a.agent.id.clone()).collect();

    // Prune files for agents no longer in the desired set.
    let entries = match fs.list_dir(&cap, &agents_dir) {
        Ok(e) => e,
        Err(ship_substrate::ports::FsError::NotFound) => Vec::new(),
        Err(e) => return Err(e.into()),
    };
    for entry in entries {
        if entry.extension().and_then(|e| e.to_str()) != Some("jsonc") {
            continue;
        }
        let Some(id) = entry.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !keep.contains(id) {
            fs.remove(&cap, &agent_rel(&agents_dir, id))?;
        }
    }

    for agent in agents {
        let mut profile = agent.clone();
        profile.rules.refs = profile
            .rules
            .refs
            .iter()
            .map(|id| normalize_rule_external_id(id))
            .collect();
        let rel = agent_rel(&agents_dir, &agent.agent.id);
        let existing = fs.read_string(&cap, &rel).ok();
        let content = render_agent_file(existing.as_deref(), &profile)?;
        fs.write_atomic(&cap, &rel, content.as_bytes())?;
    }
    Ok(())
}

/// Post-write NOTIFY (S5): after a gated def-file write, emit ONE
/// `agent.def.updated {path, content_hash}` for `<id>.jsonc` so the read-only
/// index re-reads and stays live — deterministically, NOT via the inotify
/// watcher. The hash is read off the just-written file through the SAME FsBroker
/// door the writer uses (an absent file → empty hash = the deleted marker the
/// index drops the row on). Space-scoped (the def lives in a space's `.ship`).
///
/// This is the DETERMINISTIC half of the agent-def producer→index loop
/// (job/transient-events): a gated CRUD write produces a durable row here,
/// independent of the OS watcher. The watcher → consumer loop
/// (`ship_settings::watcher` → `agent_def::file_changed_consumer`) is the
/// EDIT-TIME half. Both produce the same event shape, and the read-only
/// index in `ship_settings::agents_index` is idempotent on the notification.
/// The writer itself is untouched.
pub async fn notify_agent_def_saved(
    kernel: &Arc<KernelLog>,
    fs: &dyn FsBroker,
    ship_dir: &Path,
    id: &str,
    space: Option<SpaceId>,
) {
    let agents_dir = ship_substrate::project::agents_ns(ship_dir);
    let file = agents_dir.join(format!("{id}.jsonc"));
    let rel = file
        .strip_prefix(&agents_dir)
        .unwrap_or(&file)
        .to_string_lossy()
        .to_string();
    let cap = FsCap::new_unchecked(
        agents_dir.clone(),
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        FsMode::ReadOnly,
    );
    // Empty hash signals a removed file (read fails) — a valid "deleted" marker.
    let content_hash = fs
        .read(&cap, &rel)
        .map(|bytes| ship_types::sha256_hex(&bytes))
        .unwrap_or_default();
    let payload = AgentDefUpdated {
        path: rel.clone(),
        content_hash,
    };
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

/// Render an agent profile to JSONC. When `existing` already holds a canonical
/// nested-shape file, the platform-managed fields are merged in and the
/// remaining hand-authored content (`hooks`, `plugins`, `rules.inline`,
/// `harness_settings`, `apps`, schema reference) is preserved. Otherwise the
/// platform-managed fields are emitted as a minimal canonical nested file.
fn render_agent_file(existing: Option<&str>, flat: &AgentProfile) -> Result<String, ConfigError> {
    let parsed: Option<serde_json::Value> = existing
        .map(ship_substrate::fs_util::strip_jsonc_comments)
        .and_then(|stripped| serde_json::from_str(&stripped).ok());

    let canonical_doc = parsed.filter(|v| v.get("agent").is_some());

    if let Some(mut doc) = canonical_doc {
        merge_flat_into_canonical(&mut doc, flat);
        let json =
            serde_json::to_string_pretty(&doc).map_err(|e| ConfigError::Internal(e.to_string()))?;
        Ok(format!("{json}\n"))
    } else {
        let mut agent_block = serde_json::json!({ "id": flat.agent.id, "name": flat.agent.name });
        set_meta_fields(&mut agent_block, flat);
        let mut canonical = serde_json::json!({
            "agent": agent_block,
            "skills": { "refs": flat.skills.refs },
            "mcp": { "servers": flat.mcp.servers },
        });
        // Thread capabilities when the flat view carries them (Gap C).
        if !flat.capabilities.is_empty() {
            canonical["capabilities"] =
                serde_json::to_value(&flat.capabilities).unwrap_or(serde_json::json!([]));
        }
        let json = serde_json::to_string_pretty(&canonical)
            .map_err(|e| ConfigError::Internal(e.to_string()))?;
        Ok(format!("{json}\n"))
    }
}

/// Set the optional/list meta fields on an `agent` JSON block from the flat view.
fn set_meta_fields(block: &mut serde_json::Value, flat: &AgentProfile) {
    let Some(a) = block.as_object_mut() else {
        return;
    };
    if let Some(desc) = &flat.agent.description {
        a.insert(
            "description".into(),
            serde_json::Value::String(desc.clone()),
        );
    }
}

/// Overlay the platform-managed fields of a flat [`AgentProfile`] onto a
/// canonical nested JSONC document. Fields the platform does not own (`hooks`,
/// `plugins`, `rules.inline`, `harness_settings`) are left untouched.
fn merge_flat_into_canonical(doc: &mut serde_json::Value, flat: &AgentProfile) {
    let Some(root) = doc.as_object_mut() else {
        return;
    };
    let agent_obj = root
        .entry("agent".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(a) = agent_obj.as_object_mut() {
        a.insert(
            "id".into(),
            serde_json::Value::String(flat.agent.id.clone()),
        );
        a.insert(
            "name".into(),
            serde_json::Value::String(flat.agent.name.clone()),
        );
    }
    set_meta_fields(agent_obj, flat);

    let skills_obj = root
        .entry("skills".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(s) = skills_obj.as_object_mut() {
        s.insert("refs".into(), to_str_array(&flat.skills.refs));
    }

    let mcp_obj = root
        .entry("mcp".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(m) = mcp_obj.as_object_mut() {
        m.insert("servers".into(), to_str_array(&flat.mcp.servers));
    }

    // capabilities — only written when the flat view carries them; absent means
    // "no change" so hand-authored capabilities survive a re-save via this path.
    if !flat.capabilities.is_empty() {
        if let Ok(caps) = serde_json::to_value(&flat.capabilities) {
            root.insert("capabilities".into(), caps);
        }
    }
}

fn to_str_array(items: &[String]) -> serde_json::Value {
    serde_json::Value::Array(
        items
            .iter()
            .cloned()
            .map(serde_json::Value::String)
            .collect(),
    )
}

#[cfg(test)]
#[path = "agents_write_tests.rs"]
mod tests;
