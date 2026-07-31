//! `compose` — the space roll-up law (Phase-3 composition).
//!
//! One pure fn: [`compose_space`] takes [`RollupInputs`] (tier descriptors +
//! precedence table) and emits ONE composed [`ExtensionLibrary`] plus the
//! [`ShadowRecord`]s for every id an outranked tier lost. No IO, no runtime
//! deps — the law is a transformation, callers own where descriptors come
//! from and where the result goes.
//!
//! ## The law
//!
//! - The composed library IS the space tier's descriptor with its noun
//!   collections (`agents`, `agent_profiles`, `skills`, `rules`,
//!   `mcp_servers`) replaced by the tier-composed sets. Every other field —
//!   config knobs, env, plugins, harness settings — passes through from the
//!   space tier verbatim. No space tier in the inputs → an empty base.
//! - Nouns compose id-keyed, low rank to high: a higher-ranked tier's item
//!   replaces a lower-ranked one with the same id IN PLACE (first
//!   occurrence keeps its position; each tier's internal order is
//!   preserved). Every replacement is recorded as a [`ShadowRecord`] —
//!   losers are never silently dropped.
//! - Equal ranks resolve by declared extension order, later wins. An
//!   extension whose source kind has no precedence row is an error, never a
//!   silent default.
//! - **Permissions and hooks are NOT composed.** The space tier's values
//!   pass through untouched; other tiers' permissions/hooks are ignored.
//!   Cross-tier permission/hook merge belongs to the permissions/hooks
//!   format cell (HARD-gated design pass), not this law.
//! - **Views compose by namespace-prefix merge, not id-replace.** Every
//!   other noun collides on a bare id; a view collides only on
//!   `(prefix, route)` together. The same local `route` from two
//!   extensions under different prefixes is not a collision — both survive
//!   side by side. Only a genuine same-prefix collision resolves by rank,
//!   same as any other noun (higher rank wins, loser recorded as a
//!   [`ShadowRecord`]).

use std::collections::HashMap;

use ship_ui::ViewDescriptor;

use crate::resolve::{ExtSource, ExtensionDescriptor, ExtensionLibrary, RollupInputs};

/// The composed result: the winning library + who lost where.
///
/// Route trees are not emitted here — consumers hydrate their own tree via
/// `ship_ui::compose_route_tree(&composed.library.views)` when they consume
/// the view slice (dtsgen, shipd views, tests).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposedLibrary {
    pub library: ExtensionLibrary,
    /// One record per id collision the composition resolved, in resolution
    /// order. Consumers (the def index) persist these as provenance.
    pub shadows: Vec<ShadowRecord>,
}

/// Which noun collection a shadow happened in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NounKind {
    Agent,
    AgentProfile,
    Skill,
    Rule,
    McpServer,
    View,
}

/// An id collision the law resolved: `losing_source`'s item was replaced by
/// `winning_source`'s. `losing_tier_hash` is the losing EXTENSION's
/// content hash (tier-level — the descriptor carries no per-item hash).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShadowRecord {
    pub noun: NounKind,
    pub id: String,
    pub losing_source: ExtSource,
    pub losing_tier_hash: String,
    pub winning_source: ExtSource,
}

/// Apply the composition law. Pure; deterministic for identical inputs.
pub fn compose_space(inputs: &RollupInputs) -> anyhow::Result<ComposedLibrary> {
    // Rank every extension via the kind-level table. No row = no law = error.
    let mut ordered: Vec<(u32, &ExtensionDescriptor)> = Vec::with_capacity(inputs.extensions.len());
    for ext in &inputs.extensions {
        let kind = ext.source.kind();
        let rank = inputs
            .precedence
            .iter()
            .find(|row| row.kind == kind)
            .map(|row| row.rank)
            .ok_or_else(|| {
                anyhow::anyhow!("no precedence row for extension source kind {kind:?}")
            })?;
        ordered.push((rank, ext));
    }
    // Stable sort: equal ranks keep declared order, so later-declared wins.
    ordered.sort_by_key(|(rank, _)| *rank);

    // Base = the space-root tier verbatim (permissions, hooks, and
    // every config field ride through untouched). The multi-extension
    // shape (Wave 4 phase B) declared the space-root descriptor FIRST
    // in declared order, so the first Space in declared order is
    // the space root. Every child-dir Space descriptor is a
    // sibling; the resolver zeroed their permissions + hooks at
    // the per-root identity source (the `roll_up` orchestrator), so
    // the fence is the FIRST Space's permissions/hooks regardless
    // of how many children exist. Absent → empty library.
    let mut library = ordered
        .iter()
        .find(|(_, ext)| ext.source == ExtSource::Space)
        .map(|(_, ext)| ext.descriptor.clone())
        .unwrap_or_default();

    let mut shadows = Vec::new();
    library.agents = compose_noun(
        &ordered,
        &mut shadows,
        NounKind::Agent,
        |d| &d.agents,
        |a| a.agent.id.as_str(),
    );
    library.agent_profiles = compose_noun(
        &ordered,
        &mut shadows,
        NounKind::AgentProfile,
        |d| &d.agent_profiles,
        |a| a.agent.id.as_str(),
    );
    library.skills = compose_noun(
        &ordered,
        &mut shadows,
        NounKind::Skill,
        |d| &d.skills,
        |s| s.id.as_str(),
    );
    library.rules = compose_noun(
        &ordered,
        &mut shadows,
        NounKind::Rule,
        |d| &d.rules,
        |r| r.file_name.as_str(),
    );
    library.mcp_servers = compose_noun(
        &ordered,
        &mut shadows,
        NounKind::McpServer,
        |d| &d.mcp_servers,
        |m| m.id.as_str(),
    );
    library.views = compose_views(&ordered, &mut shadows);

    Ok(ComposedLibrary { library, shadows })
}

/// Compose ONE noun collection across the rank-ordered tiers. First
/// occurrence of an id claims its output position; later (higher) tiers
/// replace in place and push a [`ShadowRecord`].
fn compose_noun<'a, T: Clone + 'a>(
    ordered: &[(u32, &'a ExtensionDescriptor)],
    shadows: &mut Vec<ShadowRecord>,
    noun: NounKind,
    items: impl Fn(&'a ExtensionLibrary) -> &'a [T],
    id_of: impl Fn(&T) -> &str,
) -> Vec<T> {
    let mut out: Vec<T> = Vec::new();
    // Parallel to `out`: the source + tier hash currently occupying the slot.
    let mut owners: Vec<(ExtSource, String)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for (_, ext) in ordered {
        for item in items(&ext.descriptor) {
            let id = id_of(item);
            match index.get(id) {
                Some(&slot) => {
                    let (losing_source, losing_tier_hash) = owners[slot].clone();
                    shadows.push(ShadowRecord {
                        noun,
                        id: id.to_string(),
                        losing_source,
                        losing_tier_hash,
                        winning_source: ext.source.clone(),
                    });
                    out[slot] = item.clone();
                    owners[slot] = (ext.source.clone(), ext.content_hash.clone());
                }
                None => {
                    index.insert(id.to_string(), out.len());
                    out.push(item.clone());
                    owners.push((ext.source.clone(), ext.content_hash.clone()));
                }
            }
        }
    }
    out
}

/// Compose the views noun by namespace-prefix merge. Unlike
/// [`compose_noun`]'s bare-id key, a view's slot is keyed on
/// `(prefix, route)` together: the same `route` under two different
/// prefixes claims two distinct slots and both survive. Only a genuine
/// same-prefix collision replaces in place and records a [`ShadowRecord`],
/// same resolution order (rank, then declared order) as every other noun.
fn compose_views(
    ordered: &[(u32, &ExtensionDescriptor)],
    shadows: &mut Vec<ShadowRecord>,
) -> Vec<ViewDescriptor> {
    let mut out: Vec<ViewDescriptor> = Vec::new();
    // Parallel to `out`: the source + tier hash currently occupying the slot.
    let mut owners: Vec<(ExtSource, String)> = Vec::new();
    let mut index: HashMap<(String, String), usize> = HashMap::new();
    for (_, ext) in ordered {
        for view in &ext.descriptor.views {
            let key = (view.prefix.clone(), view.route.clone());
            match index.get(&key) {
                Some(&slot) => {
                    let (losing_source, losing_tier_hash) = owners[slot].clone();
                    shadows.push(ShadowRecord {
                        noun: NounKind::View,
                        id: format!("{}{}", view.prefix, view.route),
                        losing_source,
                        losing_tier_hash,
                        winning_source: ext.source.clone(),
                    });
                    out[slot] = view.clone();
                    owners[slot] = (ext.source.clone(), ext.content_hash.clone());
                }
                None => {
                    index.insert(key, out.len());
                    out.push(view.clone());
                    owners.push((ext.source.clone(), ext.content_hash.clone()));
                }
            }
        }
    }
    out
}
