//! Global (cross-space) search fan-out.
//!
//! `SearchService` stays scope-blind: it runs one `SearchContext` and knows
//! nothing about spaces. Global reach is built ON TOP of it here — the
//! orchestrator runs the SAME scope-blind service once per reachable space,
//! stamps each space's hits with its `space_id`, then [`merge_global`] folds
//! every space's results into one ranked, capped list.
//!
//! Why the merge is a plain stable sort: every space's hits are scored by the
//! identical heuristic ([`crate::ranking`]), so scores are directly comparable
//! across spaces. A global ranking is therefore just "concatenate, sort by
//! score desc, cap to `limit`" — the same shape the in-space aggregator uses,
//! lifted one level up.
//!
//! ## Opt-out
//!
//! A space can opt OUT of global search (per-space `search.global = false`
//! setting). The opt-out is enforced at the ENUMERATION boundary: the caller
//! decides which spaces to run before any search happens, via the
//! `should_include` predicate on [`run_global`]. An excluded space is never
//! searched, so its hits can never reach the merge. Default is included.

use crate::service::{SearchContext, SearchService};
use crate::types::{SearchHit, SearchOpts};

/// One reachable space, paired with a factory for its scope-blind
/// `SearchContext`. The factory is BOXED so a `Vec<SpaceTarget<C>>` can hold
/// targets whose factory closures differ (every closure has a distinct type).
/// The factory is called only if the space is included (not opted out), so a
/// per-space context for an excluded space is never built.
pub struct SpaceTarget<C> {
    pub space_id: String,
    pub make_ctx: Box<dyn Fn() -> C + Send + Sync>,
}

impl<C> SpaceTarget<C> {
    pub fn new(
        space_id: impl Into<String>,
        make_ctx: impl Fn() -> C + Send + Sync + 'static,
    ) -> Self {
        Self { space_id: space_id.into(), make_ctx: Box::new(make_ctx) }
    }
}

/// Merge per-space hit lists into one ranked, capped list.
///
/// Each `(space_id, hits)` pair has its hits stamped with `space_id` (the
/// space they came from), then all hits are merged and ranked by the shared
/// heuristic score (desc). On ties the original space order is preserved
/// (stable sort). The result is capped to `limit`.
///
/// This is the pure fan-out core — no I/O, no async. The orchestrator
/// ([`run_global`]) and shipd's wire both reduce to a call to this.
pub fn merge_global(
    per_space: Vec<(String, Vec<SearchHit>)>,
    limit: usize,
) -> Vec<SearchHit> {
    let mut merged: Vec<SearchHit> = Vec::new();
    for (space_id, hits) in per_space {
        for mut hit in hits {
            hit.space_id = space_id.clone();
            merged.push(hit);
        }
    }

    // Stable sort by score desc — scores are comparable across spaces
    // (same scorer), and stability keeps per-space input order on ties.
    merged.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let cap = if limit == 0 { 20 } else { limit.min(200) };
    merged.truncate(cap);
    merged
}

/// Run a GLOBAL search across the supplied space targets.
///
/// `should_include(space_id) -> bool` is the opt-out gate, evaluated per
/// space BEFORE its context is built or searched (default callers pass a
/// closure returning `true`; opted-out spaces return `false`). Included
/// spaces are searched with the same scope-blind `SearchService::run`; their
/// hits are stamped and merged via [`merge_global`].
///
/// `opts.limit` caps the merged set. Per-space cursor/pagination is NOT
/// threaded across the fan-out (a global cursor is a follow); each space is
/// searched from offset 0 and the merged top-`limit` is returned.
pub async fn run_global<C, P>(
    service: &SearchService,
    targets: Vec<SpaceTarget<C>>,
    query: &str,
    opts: SearchOpts,
    should_include: P,
) -> Vec<SearchHit>
where
    C: SearchContext,
    P: Fn(&str) -> bool,
{
    let limit = opts.limit_or_default();
    let mut per_space: Vec<(String, Vec<SearchHit>)> = Vec::new();

    for target in targets {
        if !should_include(&target.space_id) {
            continue;
        }
        let ctx = (target.make_ctx)();
        // Each space runs the same scope-blind service. Pull `limit` hits per
        // space so the merge has enough candidates to fill the global cap.
        let per_space_opts = SearchOpts {
            types: opts.types.clone(),
            limit,
            cursor: None,
        };
        let resp = service.run(&ctx, query, per_space_opts).await;
        per_space.push((target.space_id, resp.hits));
    }

    merge_global(per_space, limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EntityKind;

    fn hit(id: &str, score: f64) -> SearchHit {
        SearchHit {
            kind: EntityKind::View,
            id: id.into(),
            label: id.into(),
            path: format!("/{id}"),
            icon: None,
            snippet: None,
            score,
            space_id: String::new(),
        }
    }

    // (a) global merge tags + orders hits across >=2 spaces by score.
    #[test]
    fn merge_tags_and_orders_across_spaces() {
        let merged = merge_global(
            vec![
                ("space-a".into(), vec![hit("alpha", 50.0), hit("beta", 90.0)]),
                ("space-b".into(), vec![hit("gamma", 70.0)]),
            ],
            10,
        );
        assert_eq!(merged.len(), 3);
        // Ranked by score desc, regardless of which space they came from.
        assert_eq!(merged[0].id, "beta");
        assert_eq!(merged[0].space_id, "space-a");
        assert_eq!(merged[1].id, "gamma");
        assert_eq!(merged[1].space_id, "space-b");
        assert_eq!(merged[2].id, "alpha");
        assert_eq!(merged[2].space_id, "space-a");
    }

    // (b) every merged hit carries the space it came from.
    #[test]
    fn merge_stamps_space_id_on_every_hit() {
        let merged = merge_global(
            vec![
                ("home".into(), vec![hit("x", 10.0)]),
                ("work".into(), vec![hit("y", 20.0)]),
            ],
            10,
        );
        assert!(merged.iter().all(|h| !h.space_id.is_empty()));
        assert_eq!(merged.iter().find(|h| h.id == "x").unwrap().space_id, "home");
        assert_eq!(merged.iter().find(|h| h.id == "y").unwrap().space_id, "work");
    }

    // Merge caps the global set to `limit`.
    #[test]
    fn merge_caps_to_limit() {
        let merged = merge_global(
            vec![(
                "s".into(),
                vec![hit("a", 1.0), hit("b", 2.0), hit("c", 3.0)],
            )],
            2,
        );
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].id, "c"); // top scores survive the cap
        assert_eq!(merged[1].id, "b");
    }

    // (d) the deeplink URI composes as ship://space/<id><path>.
    #[test]
    fn deeplink_composes_with_space_prefix() {
        let merged = merge_global(vec![("space-7".into(), vec![hit("dash", 1.0)])], 10);
        assert_eq!(merged[0].deeplink(), "ship://space/space-7/dash");
    }

    // This-space hit (no space_id) deeplinks to the bare intra-space path.
    #[test]
    fn deeplink_this_space_is_bare_path() {
        let h = hit("dash", 1.0);
        assert!(h.space_id.is_empty());
        assert_eq!(h.deeplink(), "/dash");
    }
}
