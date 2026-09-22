//! Legacy `.ts`-config ingest seam — V8-FREE by design.
//!
//! ARC A5: agent nouns are JSONC-only (`agents/<id>.jsonc`). This module remains
//! for residual `.ts` sources still on disk (read-only surface) and for
//! projecting an already-resolved [`agent_schema::AgentProfile`] into the index
//! summary. Evaluation of `defineAgent` is retired — do not reintroduce it.
//!
//! - [`read_agent_ts_sources`] — surface raw `.ts` source text if any remain.
//! - [`source_from_profile`] — project a resolved profile through the SAME
//!   [`crate::agents::to_source`] projection the `.jsonc` reader uses.

use std::path::Path;

use ship_substrate::ports::{FsBroker, FsError, FsMode};

use crate::agents::{AgentDefSource, DefDoc, agents_cap, to_source};
use crate::error::ConfigError;

/// Read every `<stem>.ts` under `<ship_dir>/agents/` as `(stem, source)` pairs,
/// sorted by stem. Missing dir → empty. Via [`FsBroker`] — never `std::fs`.
///
/// The `.ts` SOURCE is returned verbatim; EVALUATION (V8, via `agent-config-eval`)
/// happens at the ingest layer, which turns each source into
/// [`agent_schema::AgentProfile`]s and feeds them back through [`source_from_profile`].
pub fn read_agent_ts_sources(
    fs: &dyn FsBroker,
    ship_dir: &Path,
) -> Result<Vec<(String, String)>, ConfigError> {
    let (cap, agents_dir) = agents_cap(ship_dir, FsMode::ReadOnly);
    let entries = match fs.list_dir(&cap, &agents_dir) {
        Ok(e) => e,
        Err(FsError::NotFound) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        if entry.extension().and_then(|e| e.to_str()) != Some("ts") {
            continue;
        }
        let Some(stem) = entry.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let raw = fs.read(&cap, &entry.to_string_lossy())?;
        out.push((stem.to_string(), String::from_utf8_lossy(&raw).into_owned()));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Project a resolved [`agent_schema::AgentProfile`] (the `.ts`-config eval output)
/// into the index summary, through the SAME [`to_source`] projection the `.jsonc`
/// reader uses. The profile is re-serialized to the canonical doc value +
/// content-hashed over those bytes. Identity is the agent id: the rel path is
/// `<id>.ts`, so one `.ts` module emitting N agents yields N distinct rows.
/// Harnesses are not agent-declared; summary `harness` stays empty unless the
/// serialized doc carries `agent.harnesses`.
pub fn source_from_profile(profile: &agent_schema::AgentProfile) -> AgentDefSource {
    let value = serde_json::to_value(profile).unwrap_or(serde_json::Value::Null);
    let raw = serde_json::to_vec(&value).unwrap_or_default();
    let mut src = to_source(&DefDoc {
        stem: profile.agent.id.clone(),
        raw,
        value,
    });
    src.path = format!("{}.ts", profile.agent.id);
    src
}
