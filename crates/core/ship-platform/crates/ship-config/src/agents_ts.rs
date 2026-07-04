//! The `.ts`-config ingest seam — V8-FREE by design.
//!
//! `.ship/agents/<id>.ts` files are authored with `defineAgent` and EVALUATED by
//! `agent-config-eval` in a ship-js isolate (which pulls V8). That eval lives at
//! the ingest layer (shipd), NEVER here — this crate stays V8-free (grep-gate
//! invariant). This module owns only the two V8-free ends of the seam:
//!
//! - [`read_agent_ts_sources`] — surface the raw `.ts` source text for the ingest
//!   layer to evaluate (through the same [`FsBroker`] door the `.jsonc` reader
//!   uses).
//! - [`source_from_profile`] — project an already-resolved [`compiler::AgentProfile`]
//!   back into the index summary, through the SAME [`crate::agents::to_source`]
//!   projection the `.jsonc` reader uses, so a `.ts`-authored agent indexes
//!   identically to a `.jsonc` one.

use std::path::Path;

use ship_substrate::ports::{FsBroker, FsError, FsMode};

use crate::agents::{agents_cap, to_source, AgentDefSource, DefDoc};
use crate::error::ConfigError;

/// Read every `<stem>.ts` under `<ship_dir>/agents/` as `(stem, source)` pairs,
/// sorted by stem. Missing dir → empty. Via [`FsBroker`] — never `std::fs`.
///
/// The `.ts` SOURCE is returned verbatim; EVALUATION (V8, via `agent-config-eval`)
/// happens at the ingest layer, which turns each source into
/// [`compiler::AgentProfile`]s and feeds them back through [`source_from_profile`].
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

/// Project a resolved [`compiler::AgentProfile`] (the `.ts`-config eval output)
/// into the index summary, through the SAME [`to_source`] projection the `.jsonc`
/// reader uses. The profile is re-serialized to the canonical doc value +
/// content-hashed over those bytes. Identity is the agent id: the rel path is
/// `<id>.ts`, so one `.ts` module emitting N agents yields N distinct rows.
/// `harness` comes from the profile's `target_agents` — the `.ts` model's
/// harness-target list.
pub fn source_from_profile(profile: &compiler::AgentProfile) -> AgentDefSource {
    let value = serde_json::to_value(profile).unwrap_or(serde_json::Value::Null);
    let raw = serde_json::to_vec(&value).unwrap_or_default();
    let mut src = to_source(&DefDoc { stem: profile.agent.id.clone(), raw, value });
    src.path = format!("{}.ts", profile.agent.id);
    src.harness = profile.target_agents.clone();
    src
}
