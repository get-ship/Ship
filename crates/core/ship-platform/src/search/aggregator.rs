//! Adapter fan-out + result merging + cursor management.
//!
//! Adapters run in parallel via `futures_util::future::join_all`; the
//! aggregator merges their hits, sorts by score (desc), takes the top
//! `limit`, and builds the next-cursor from each adapter's
//! `next_offset`. An empty cursor means there are no more results.

use std::sync::Arc;

use futures_util::future::join_all;
use serde::Serialize;

use super::adapter::EntitySearchAdapter;
use super::types::{Cursor, EntityKindFilter, SearchHit, SearchOpts};

#[derive(Serialize, Debug)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Fan out `q` to every adapter (filtered by `opts.types` when set),
/// merge their hits, rank by score (desc), and page to `opts.limit`.
pub async fn run_search(
    adapters: &[Arc<dyn EntitySearchAdapter>],
    q: &str,
    opts: SearchOpts,
) -> SearchResponse {
    let enabled: Vec<&Arc<dyn EntitySearchAdapter>> = adapters
        .iter()
        .filter(|a| adapter_enabled(a.kind_filter(), opts.types.as_deref()))
        .collect();

    let q_ref = q;
    let opts_ref = &opts;
    let futures = enabled.iter().map(|a| async move {
        let result = a.query(q_ref, opts_ref).await;
        (a.name(), result)
    });
    let results = join_all(futures).await;

    let mut next_cursor = Cursor::default();
    let mut merged: Vec<SearchHit> = Vec::new();
    for (name, result) in results {
        merged.extend(result.hits);
        if let Some(off) = result.next_offset {
            next_cursor.set(name, off);
        }
    }

    // Sort by score desc; stable order on ties preserves adapter order.
    merged.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let limit = opts.limit_or_default();
    let total = merged.len();
    let truncated = if merged.len() > limit {
        merged.truncate(limit);
        true
    } else {
        false
    };

    let cursor = if !next_cursor.is_empty() || truncated {
        Some(next_cursor.encode())
    } else {
        None
    };

    SearchResponse { hits: merged, total, cursor }
}

fn adapter_enabled(kind: EntityKindFilter, allow: Option<&[EntityKindFilter]>) -> bool {
    match allow {
        None => true,
        Some(list) => list.contains(&kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::adapter::AdapterResult;
    use crate::search::types::EntityKind;
    use async_trait::async_trait;

    struct StaticAdapter {
        name: &'static str,
        kind: EntityKindFilter,
        items: Vec<(String, f64, EntityKind)>,
    }

    #[async_trait]
    impl EntitySearchAdapter for StaticAdapter {
        fn name(&self) -> &'static str { self.name }
        fn kind_filter(&self) -> EntityKindFilter { self.kind }
        async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
            let start = opts.cursor.as_ref().map(|c| c.get(self.name)).unwrap_or(0);
            let limit = opts.limit_or_default();
            let mut hits: Vec<SearchHit> = self
                .items
                .iter()
                .filter(|(label, _, _)| q.is_empty() || label.to_lowercase().contains(&q.to_lowercase()))
                .skip(start)
                .take(limit)
                .map(|(label, score, kind)| SearchHit {
                    kind: kind.clone(),
                    id: label.clone(),
                    label: label.clone(),
                    path: format!("/{label}"),
                    icon: None,
                    snippet: None,
                    score: *score,
                })
                .collect();
            let consumed = start + hits.len();
            let total_matching = self
                .items
                .iter()
                .filter(|(label, _, _)| q.is_empty() || label.to_lowercase().contains(&q.to_lowercase()))
                .count();
            let next_offset = if consumed < total_matching { Some(consumed) } else { None };
            // Keep things stable for assertions.
            hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            AdapterResult { hits, next_offset }
        }
    }

    fn make_adapters() -> Vec<Arc<dyn EntitySearchAdapter>> {
        vec![
            Arc::new(StaticAdapter {
                name: "view",
                kind: EntityKindFilter::View,
                items: vec![
                    ("alpha".into(), 100.0, EntityKind::View),
                    ("beta".into(), 50.0, EntityKind::View),
                ],
            }),
            Arc::new(StaticAdapter {
                name: "agent",
                kind: EntityKindFilter::Agent,
                items: vec![
                    ("alpha-bot".into(), 200.0, EntityKind::Agent),
                ],
            }),
        ]
    }

    #[tokio::test]
    async fn fans_out_and_ranks_by_score() {
        let adapters = make_adapters();
        let resp = run_search(&adapters, "alpha", SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(resp.hits.len(), 2);
        assert_eq!(resp.hits[0].label, "alpha-bot"); // higher score
        assert_eq!(resp.hits[1].label, "alpha");
    }

    #[tokio::test]
    async fn type_filter_skips_disabled_adapters() {
        let adapters = make_adapters();
        let resp = run_search(
            &adapters,
            "alpha",
            SearchOpts {
                types: Some(vec![EntityKindFilter::View]),
                limit: 10,
                ..Default::default()
            },
        ).await;
        assert_eq!(resp.hits.len(), 1);
        assert_eq!(resp.hits[0].label, "alpha");
    }

    #[tokio::test]
    async fn cursor_encodes_when_truncated() {
        let adapters = make_adapters();
        let resp = run_search(&adapters, "", SearchOpts { limit: 1, ..Default::default() }).await;
        assert_eq!(resp.hits.len(), 1);
        assert!(resp.cursor.is_some());
    }

    #[tokio::test]
    async fn cursor_decode_and_resume() {
        let adapters = make_adapters();
        // First page — limit 1 forces a cursor.
        let first = run_search(&adapters, "", SearchOpts { limit: 1, ..Default::default() }).await;
        let cursor_str = first.cursor.expect("cursor expected");
        let cursor = Cursor::decode(&cursor_str).expect("decodable cursor");
        // Resume — every adapter's offset advances past what was returned.
        let second = run_search(
            &adapters,
            "",
            SearchOpts { limit: 10, cursor: Some(cursor), ..Default::default() },
        ).await;
        // The "alpha-bot" hit was on the first page (highest score);
        // resume yields the remaining items.
        assert!(!second.hits.is_empty());
    }
}
