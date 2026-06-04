//! THE agent-def reader — the one file-access door for `.ship/agents/*.jsonc`.
//!
//! THE EVICTION: the pre-carve `config/agents.rs` used raw `std::fs` to walk and
//! read `.ship/agents/<id>.jsonc`. Here ALL disk goes through the [`FsBroker`]
//! gated door — confined by an [`FsCap`]. Zero `std::fs`/`tokio::fs` in this
//! crate's read path (grep-gate invariant).
//!
//! ONE byte-read serves every granularity (S2 — agents arc, cutover):
//! - **FULL `compiler::AgentProfile`** (nested) — what the compiler's
//!   `load_agent_profiles` consumes; NO field loss.
//! - **raw `serde_json::Value`** — what the daemon view-state projects its
//!   summaries/detail from (it needs fields the nested type drops, e.g.
//!   `rules.refs`, `permissions.tier`).
//! - **SUMMARY [`AgentDefSource`]** — what the read-only index keys on
//!   (id/name/role/tags/harness/path/content_hash); `harness` is read off the
//!   raw value because the nested `compiler::AgentProfile` view drops it.
//!
//! Files are authored against the canonical schema (top-level `agent` block).
//! The WRITE path (gated, S5) lives in the sibling [`crate::agents_write`].

use std::path::{Path, PathBuf};

use ship_substrate::ports::{FsBroker, FsCap, FsError, FsMode, FsRoot};

use crate::error::ConfigError;

// ── Confinement ────────────────────────────────────────────────────────────────

/// Resolve the cap root + the agents-dir rel path for a `ship_dir`.
///
/// The cap roots at the `ship_dir`'s PARENT (the project/library root), which
/// reliably exists, and names `<.ship>/agents` as the rel path. The broker's
/// symlink-escape guard canonicalizes the deepest EXISTING ancestor of a path
/// and rejects anything that escapes the cap root — so a not-yet-created
/// `.ship/agents` cap root would itself read as an escape. Rooting at the
/// existing parent and naming a two-segment rel keeps the guard satisfied while
/// still confining to `.ship/agents`. When `ship_dir` has no parent, the cap
/// roots at `ship_dir` with rel `agents`.
fn cap_root_and_dir(ship_dir: &Path) -> (PathBuf, String) {
    let ship_name = ship_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| ship_substrate::home::ShipHome::ship_dir_name().to_string());
    match ship_dir.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            (parent.to_path_buf(), format!("{ship_name}/agents"))
        }
        _ => (ship_dir.to_path_buf(), "agents".to_string()),
    }
}

/// Resolve the cap root + rel path for an explicit `agents` directory (the
/// daemon view-state's two scopes, `<ship>/library/agents` and
/// `<project>/.ship/agents`; the CLI's `<ship>/agents`). Roots the cap at the
/// agents dir's GRANDPARENT and names a two-segment `<parent>/<leaf>` rel —
/// the SAME bootstrap-safe rooting [`cap_root_and_dir`] uses, so a not-yet-
/// created leaf never reads as an escape.
fn dir_cap_root(agents_dir: &Path) -> (PathBuf, String) {
    let leaf = agents_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "agents".to_string());
    match agents_dir.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            let parent_name = parent
                .file_name()
                .map(|n| n.to_string_lossy().into_owned());
            match (parent.parent(), parent_name) {
                (Some(gp), Some(pn)) if !gp.as_os_str().is_empty() => {
                    (gp.to_path_buf(), format!("{pn}/{leaf}"))
                }
                _ => (parent.to_path_buf(), leaf),
            }
        }
        _ => (agents_dir.to_path_buf(), leaf),
    }
}

/// Mint an FsCap rooted at the project/library dir; the agents projection lives
/// at the returned rel dir under it.
///
/// Bootstrap posture (fs_broker.rs:9-10): until `SpaceContext` minting lands in
/// Phase 5, the cap is `new_unchecked` — not new debt, the documented seam.
pub(crate) fn agents_cap(ship_dir: &Path, mode: FsMode) -> (FsCap, String) {
    cap_for(cap_root_and_dir(ship_dir), mode)
}

fn dir_agents_cap(agents_dir: &Path, mode: FsMode) -> (FsCap, String) {
    cap_for(dir_cap_root(agents_dir), mode)
}

fn cap_for((root, agents_dir): (PathBuf, String), mode: FsMode) -> (FsCap, String) {
    let cap = FsCap::new_unchecked(
        root,
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        mode,
    );
    (cap, agents_dir)
}

/// `<agents_dir>/<id>.jsonc` relative to the cap root.
pub(crate) fn agent_rel(agents_dir: &str, id: &str) -> String {
    format!("{agents_dir}/{id}.jsonc")
}

// ── Raw doc access (one byte-read, one Value parse) ─────────────────────────────

/// One def's raw bytes + parsed value + file stem, off a single read.
struct DefDoc {
    stem: String,
    raw: Vec<u8>,
    value: serde_json::Value,
}

/// Default def extensions. The canonical authoring format is `.jsonc`; `.json`
/// is also accepted so legacy/plain-JSON defs still resolve.
const DEF_EXTS: &[&str] = &["jsonc", "json"];

/// Parse raw def bytes into a JSON value. `None` when the bytes are not valid
/// JSONC — a malformed def is SKIPPED, never a hard error (the lenient contract
/// every consumer relied on: one bad file must not break the walk/lookup).
fn parse_value(raw: &[u8]) -> Option<serde_json::Value> {
    let text = String::from_utf8_lossy(raw);
    ship_substrate::config_doc::parse_doc(&text).ok()
}

/// Read + parse every def under `agents_dir`, restricted to `exts`, sorted by
/// stem. Missing dir → empty. Unparseable files are skipped. Via [`FsBroker`] —
/// never `std::fs`.
fn read_docs(
    fs: &dyn FsBroker,
    cap: &FsCap,
    agents_dir: &str,
    exts: &[&str],
) -> Result<Vec<DefDoc>, ConfigError> {
    let entries = match fs.list_dir(cap, agents_dir) {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let ext = entry.extension().and_then(|e| e.to_str());
        if !ext.is_some_and(|e| exts.contains(&e)) {
            continue;
        }
        let Some(stem) = entry.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let raw = fs.read(cap, &entry.to_string_lossy())?;
        let Some(value) = parse_value(&raw) else {
            continue;
        };
        out.push(DefDoc { stem: stem.to_string(), raw, value });
    }
    out.sort_by(|a, b| a.stem.cmp(&b.stem));
    Ok(out)
}

/// Read + parse one def by file stem, trying each ext in `exts`. `Ok(None)` when
/// no file is present (the deleted/absent case) or the file is unparseable.
fn read_doc(
    fs: &dyn FsBroker,
    cap: &FsCap,
    agents_dir: &str,
    stem: &str,
    exts: &[&str],
) -> Result<Option<DefDoc>, ConfigError> {
    for ext in exts {
        let rel = format!("{agents_dir}/{stem}.{ext}");
        match fs.read(cap, &rel) {
            Ok(raw) => {
                return Ok(parse_value(&raw)
                    .map(|value| DefDoc { stem: stem.to_string(), raw, value }));
            }
            Err(FsError::NotFound) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(None)
}

// ── Full nested profile (compiler input) ───────────────────────────────────────

/// Parse a [`DefDoc`]'s value into the FULL nested `compiler::AgentProfile`.
/// `None` when the doc lacks the canonical top-level `agent` block (the
/// loader's lenient "skip malformed" contract).
fn to_profile(doc: &DefDoc) -> Option<compiler::AgentProfile> {
    if doc.value.get("agent").is_none() {
        return None;
    }
    serde_json::from_value(doc.value.clone()).ok()
}

/// Read every `<id>.jsonc` under `<ship_dir>/agents/` as FULL nested profiles.
/// Sorted by `agent.id`. Malformed files are skipped (loader parity). Via
/// [`FsBroker`] — never `std::fs`.
pub fn read_agent_profiles(
    fs: &dyn FsBroker,
    ship_dir: &Path,
) -> Result<Vec<compiler::AgentProfile>, ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadOnly);
    profiles_from(read_docs(fs, &cap, &agents_dir, &["jsonc"])?)
}

/// Read profiles from an explicit `agents` directory (the daemon view-state's
/// per-scope dirs). Same nested shape, sorted by `agent.id`.
pub fn read_agent_profiles_in(
    fs: &dyn FsBroker,
    agents_dir: &Path,
) -> Result<Vec<compiler::AgentProfile>, ConfigError> {
    let (cap, rel) = dir_agents_cap(agents_dir, FsMode::ReadOnly);
    profiles_from(read_docs(fs, &cap, &rel, &["jsonc"])?)
}

fn profiles_from(docs: Vec<DefDoc>) -> Result<Vec<compiler::AgentProfile>, ConfigError> {
    let mut profiles: Vec<_> = docs.iter().filter_map(to_profile).collect();
    profiles.sort_by(|a, b| a.agent.id.cmp(&b.agent.id));
    Ok(profiles)
}

/// Read a single def's FULL nested profile by stem (project agents dir).
pub fn read_agent_profile(
    fs: &dyn FsBroker,
    ship_dir: &Path,
    stem: &str,
) -> Result<Option<compiler::AgentProfile>, ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadOnly);
    Ok(read_doc(fs, &cap, &agents_dir, stem, &["jsonc"])?.as_ref().and_then(to_profile))
}

/// Read a single def's FULL nested profile from an explicit file path (the CLI's
/// `find_agent_file` → `load_agent_profile` seam). `Ok(None)` when absent.
pub fn read_agent_profile_at(
    fs: &dyn FsBroker,
    path: &Path,
) -> Result<Option<compiler::AgentProfile>, ConfigError> {
    let Some(parent) = path.parent() else {
        return Ok(None);
    };
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return Ok(None);
    };
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jsonc");
    let (cap, rel) = dir_agents_cap(parent, FsMode::ReadOnly);
    Ok(read_doc(fs, &cap, &rel, stem, &[ext])?.as_ref().and_then(to_profile))
}

// ── Raw docs (daemon view-state projects summaries/detail from these) ───────────

/// Read every def under an explicit `agents` dir as `(stem, value)` pairs,
/// sorted by stem. Accepts `.jsonc` + `.json`. The daemon view-state projects
/// its summaries/detail from the raw value (fields the nested type drops).
pub fn read_agent_docs_in(
    fs: &dyn FsBroker,
    agents_dir: &Path,
) -> Result<Vec<(String, serde_json::Value)>, ConfigError> {
    let (cap, rel) = dir_agents_cap(agents_dir, FsMode::ReadOnly);
    Ok(read_docs(fs, &cap, &rel, DEF_EXTS)?
        .into_iter()
        .map(|d| (d.stem, d.value))
        .collect())
}

/// Read one def `(stem, value)` by id from an explicit `agents` dir. Tries
/// `.jsonc` then `.json`. `Ok(None)` when absent.
pub fn read_agent_doc_in(
    fs: &dyn FsBroker,
    agents_dir: &Path,
    id: &str,
) -> Result<Option<(String, serde_json::Value)>, ConfigError> {
    let (cap, rel) = dir_agents_cap(agents_dir, FsMode::ReadOnly);
    Ok(read_doc(fs, &cap, &rel, id, DEF_EXTS)?.map(|d| (d.stem, d.value)))
}

// ── Summary source (read-only index keys on this) ───────────────────────────────

/// One agent def parsed from its file, for the index. The summary granularity:
/// id/name/role/tags + `harness` (canonical `agent.harnesses`, which the nested
/// `compiler::AgentProfile` view drops) + the file-identity fields the index
/// keys on. `content_hash` is the SHA-256 of the raw bytes — the SAME value the
/// `agent.def.updated` notification carries.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentDefSource {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub tags: Vec<String>,
    pub harness: Vec<String>,
    pub path: String,
    pub content_hash: String,
}

/// Project a [`DefDoc`] to the index summary. Reads off the same parsed value +
/// raw bytes (no second read). `harness` comes from the raw value (the nested
/// profile drops it).
fn to_source(doc: &DefDoc) -> AgentDefSource {
    let agent_obj = doc.value.get("agent").unwrap_or(&doc.value);
    let id = str_field(agent_obj, "id").unwrap_or_else(|| doc.stem.clone());
    let name = str_field(agent_obj, "name").unwrap_or_else(|| id.clone());
    AgentDefSource {
        id,
        name,
        role: str_field(agent_obj, "role"),
        tags: str_array(agent_obj, "tags"),
        harness: str_array(agent_obj, "harnesses"),
        path: format!("{}.jsonc", doc.stem),
        content_hash: ship_types::sha256_hex(&doc.raw),
    }
}

/// Read every `<id>.jsonc` under `<ship_dir>/agents/` as index source records.
/// Sorted by id. Missing dir → empty. Via [`FsBroker`] — never `std::fs`.
pub fn read_agent_defs(
    fs: &dyn FsBroker,
    ship_dir: &Path,
) -> Result<Vec<AgentDefSource>, ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadOnly);
    let mut defs: Vec<_> = read_docs(fs, &cap, &agents_dir, &["jsonc"])?
        .iter()
        .map(to_source)
        .collect();
    defs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(defs)
}

/// Read one def by file stem (`<stem>.jsonc`) as an index source record.
/// `Ok(None)` when absent (the deleted case). Via [`FsBroker`] — never
/// `std::fs`.
pub fn read_agent_def(
    fs: &dyn FsBroker,
    ship_dir: &Path,
    stem: &str,
) -> Result<Option<AgentDefSource>, ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadOnly);
    Ok(read_doc(fs, &cap, &agents_dir, stem, &["jsonc"])?
        .as_ref()
        .map(to_source))
}

// ── small value helpers ─────────────────────────────────────────────────────────

fn str_field(obj: &serde_json::Value, key: &str) -> Option<String> {
    obj.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

fn str_array(obj: &serde_json::Value, key: &str) -> Vec<String> {
    obj.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "agents_tests.rs"]
mod tests;
