//! Phase-3 roll-up provenance (F4, `census-phase3-agents.md`): ingests a
//! `stack_model::compose_space` result, upserting every winning profile as the
//! ACTIVE row and every `AgentProfile`-noun `ShadowRecord` as a SEPARATE
//! shadow row — so the index can answer both "where did this def come from"
//! and "why is it not active" without a second write path. Split out of
//! `agents_index.rs`'s core (schema, plain reindex, read surface) to keep
//! that file under the file-size cap.

use std::collections::HashMap;

use ship_substrate::ports::Row;

use super::AgentDefIndex;
use crate::error::ConfigError;

/// Where an [`super::AgentDefRecord`] row's content came from within a
/// Phase-3 extension roll-up ([`AgentDefIndex::upsert_composed`]). Carries
/// the SAME `ExtSource` JSON + content hash the law's own
/// `stack_model::ExtensionDescriptor` carries — the index re-derives provenance
/// from `rollup_inputs`, it does not invent a parallel vocabulary (the wire
/// scope strings `project`/`user` are a separate, untouched concern).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefProvenance {
    pub ext_source: String,
    pub tier_hash: String,
    /// `Some(<winning ExtSource JSON>)` for a shadow (losing) row; `None` for
    /// the active row.
    pub shadowed_by: Option<String>,
}

impl AgentDefIndex {
    /// Upsert one SHADOWED agent profile — a def a Phase-3 roll-up composed
    /// OVER. Keyed apart from the active row (same id, different key:
    /// `<scope>:<id>#shadow:<tier_hash>`) so the loser is retained as its OWN
    /// row instead of clobbering the winner.
    async fn upsert_shadow(
        &self,
        scope: &str,
        profile: &agent_schema::AgentProfile,
        provenance: &DefProvenance,
    ) -> Result<(), ConfigError> {
        let src = crate::agents_ts::source_from_profile(profile);
        let key = shadow_row_key(scope, &super::file_stem(&src.path), &provenance.tier_hash);
        self.upsert(key, scope, &src, Some(provenance)).await
    }

    /// Ingest a Phase-3 composed roll-up (`stack_model::compose_space`'s
    /// output, Phase-2 J2's `space_resolver::roll_up`): upsert every winning
    /// [`agent_schema::AgentProfile`] as the ACTIVE row, plus one row per
    /// `AgentProfile`-noun `stack_model::ShadowRecord` the law recorded as a
    /// SHADOW row.
    ///
    /// A winner's provenance is derived by re-applying the SAME
    /// rank-then-declared-order rule `compose_space` used, over
    /// `composed.library.rollup_inputs`'s own extensions + precedence table
    /// — the index has no privileged view into the law, only its inputs. A
    /// shadow row's provenance comes directly off its `ShadowRecord`; the
    /// losing profile's content is looked up in `rollup_inputs` by matching
    /// that same `(source, content_hash)` pair. Best-effort: a single
    /// profile's upsert failure is logged and skipped, never aborts the
    /// ingest.
    pub async fn upsert_composed(&self, scope: &str, composed: &stack_model::ComposedLibrary) {
        let rollup = composed.library.rollup_inputs.as_ref();
        let owners = rollup.map(agent_profile_owners).unwrap_or_default();

        for profile in &composed.library.agent_profiles {
            let id = profile.agent.id.clone();
            let provenance = owners.get(&id).map(|(source, hash)| DefProvenance {
                ext_source: ext_source_json(source),
                tier_hash: hash.clone(),
                shadowed_by: None,
            });
            if let Err(e) = self
                .upsert_profile(scope, profile, provenance.as_ref())
                .await
            {
                tracing::warn!("agent-def index: composed upsert failed for {id}: {e}");
            }
        }

        for shadow in &composed.shadows {
            if shadow.noun != stack_model::NounKind::AgentProfile {
                continue;
            }
            let Some(profile) = find_shadowed_profile(rollup, shadow) else {
                continue;
            };
            let provenance = DefProvenance {
                ext_source: ext_source_json(&shadow.losing_source),
                tier_hash: shadow.losing_tier_hash.clone(),
                shadowed_by: Some(ext_source_json(&shadow.winning_source)),
            };
            if let Err(e) = self.upsert_shadow(scope, profile, &provenance).await {
                tracing::warn!(
                    "agent-def index: shadow upsert failed for {}: {e}",
                    shadow.id
                );
            }
        }
    }
}

/// Row key for a shadow record — same `(scope, stem)` id-space as the active
/// row, disambiguated by the LOSING tier's content hash so a multi-tier
/// collision on one id (more than one loser) doesn't overwrite itself.
fn shadow_row_key(scope: &str, stem: &str, tier_hash: &str) -> String {
    format!("{scope}:{stem}#shadow:{tier_hash}")
}

/// Serialize a `stack_model::ExtSource` EXACTLY as the roll-up law emits it —
/// the vocabulary pass is fenced; this must never hand-roll a label.
fn ext_source_json(source: &stack_model::ExtSource) -> String {
    serde_json::to_string(source).unwrap_or_default()
}

/// For each agent-profile id in a Phase-3 roll-up, the extension tier that
/// currently owns it — re-applying the SAME rank-then-declared-order rule
/// `stack_model::compose_space` uses, so a winner's provenance matches the
/// law's own resolution with no duplicated judgment call.
fn agent_profile_owners(
    rollup: &stack_model::RollupInputs,
) -> HashMap<String, (stack_model::ExtSource, String)> {
    let mut ranked: Vec<(u32, &stack_model::ExtensionDescriptor)> = rollup
        .extensions
        .iter()
        .filter_map(|ext| {
            let kind = ext.source.kind();
            rollup
                .precedence
                .iter()
                .find(|row| row.kind == kind)
                .map(|row| (row.rank, ext))
        })
        .collect();
    ranked.sort_by_key(|(rank, _)| *rank);

    let mut owners = HashMap::new();
    for (_, ext) in ranked {
        for profile in &ext.descriptor.agent_profiles {
            owners.insert(
                profile.agent.id.clone(),
                (ext.source.clone(), ext.content_hash.clone()),
            );
        }
    }
    owners
}

/// Look up the LOSING `AgentProfile` a `stack_model::ShadowRecord` names, by
/// matching its `(losing_source, losing_tier_hash)` back to the extension
/// that contributed it. `None` when `rollup_inputs` is absent or the id
/// can't be found (defensive — should not happen for a shadow the SAME
/// roll-up produced).
fn find_shadowed_profile<'a>(
    rollup: Option<&'a stack_model::RollupInputs>,
    shadow: &stack_model::ShadowRecord,
) -> Option<&'a agent_schema::AgentProfile> {
    let rollup = rollup?;
    rollup
        .extensions
        .iter()
        .find(|ext| {
            ext.source == shadow.losing_source && ext.content_hash == shadow.losing_tier_hash
        })
        .and_then(|ext| {
            ext.descriptor
                .agent_profiles
                .iter()
                .find(|p| p.agent.id == shadow.id)
        })
}

/// A row's `shadowed_by` column is set (non-empty) — it lost a Phase-3
/// roll-up composition and is not the active def for its id. Goes through
/// [`row_opt_str`], not a direct `Value::String` match — `shadowed_by`
/// stores an `ExtSource` JSON object, so the DataStore's JSON-column
/// reparse (see [`row_opt_str`]) hands back a `Value::Object`, not a
/// `Value::String`, on read-back.
pub(super) fn is_shadow_row(row: &Row) -> bool {
    row_opt_str(row, "shadowed_by").is_some()
}

/// A stored TEXT field, as a `String`. The DataStore eagerly re-parses a
/// TEXT column's value as JSON when the stored bytes happen to BE valid JSON
/// (documented in [`row_doc`]'s "accept both" contract) — `ext_source` and
/// `shadowed_by` store an `ExtSource` JSON object (`{"kind":"space"}`), so a
/// read-back hands us that ALREADY parsed into a `Value::Object`, not the
/// `Value::String` we wrote. Re-serialize that case back to text so the
/// caller always sees the same string it stored. `None` for null/absent/empty.
pub(super) fn row_opt_str(row: &Row, key: &str) -> Option<String> {
    match row.fields.get(key) {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => (!s.is_empty()).then(|| s.clone()),
        Some(other) => {
            Some(serde_json::to_string(other).unwrap_or_default()).filter(|s| !s.is_empty())
        }
    }
}

/// Extract `(agent_id, doc)` from a row. `doc_json` holds the serialized def
/// value; the DataStore may return it as the raw string OR (since it is itself
/// valid JSON) re-parsed into a value — accept both. `None` when the id or doc
/// is absent.
fn row_doc(row: &Row) -> Option<(String, serde_json::Value)> {
    let id = row
        .fields
        .get("agent_id")
        .and_then(|v| v.as_str())?
        .to_string();
    let doc = match row.fields.get("doc_json")? {
        serde_json::Value::Null => return None,
        serde_json::Value::String(s) if s.is_empty() => return None,
        serde_json::Value::String(s) => serde_json::from_str(s).ok()?,
        other => other.clone(),
    };
    Some((id, doc))
}

/// [`row_doc`] plus a merged `_provenance` object (`ext_source` /
/// `tier_content_hash` / `shadowed_by`, each `null` when the row carries no
/// Phase-3 roll-up provenance) — the `list_docs`/`get_doc` read surface's
/// provenance projection. Only merges into an object-shaped doc (agent defs
/// always are); a non-object doc is returned unchanged.
pub(super) fn row_doc_with_provenance(row: &Row) -> Option<(String, serde_json::Value)> {
    let (id, mut doc) = row_doc(row)?;
    if let Some(obj) = doc.as_object_mut() {
        obj.insert(
            "_provenance".to_string(),
            serde_json::json!({
                "ext_source": row_opt_str(row, "ext_source"),
                "tier_content_hash": row_opt_str(row, "tier_content_hash"),
                "shadowed_by": row_opt_str(row, "shadowed_by"),
            }),
        );
    }
    Some((id, doc))
}
