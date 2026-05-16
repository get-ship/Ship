//! Views adapter — searches a snapshot of the view registry by name
//! and manifest description.

use async_trait::async_trait;

use crate::search::adapter::{AdapterResult, EntitySearchAdapter};
use crate::search::ranking::score_text;
use crate::search::types::{EntityKind, EntityKindFilter, SearchHit, SearchOpts};

/// One view available to the daemon. Built by the route handler from
/// the shipd-side view discovery (`apps/shipd/src/views/discovery.rs`)
/// — we don't pull that into ship-platform.
#[derive(Clone, Debug)]
pub struct ViewSnapshot {
    pub name: String,
    pub description: Option<String>,
}

pub struct ViewsAdapter {
    pub views: Vec<ViewSnapshot>,
}

impl ViewsAdapter {
    pub fn new(views: Vec<ViewSnapshot>) -> Self { Self { views } }
}

#[async_trait]
impl EntitySearchAdapter for ViewsAdapter {
    fn name(&self) -> &'static str { "view" }
    fn kind_filter(&self) -> EntityKindFilter { EntityKindFilter::View }

    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
        let mut scored: Vec<(f64, SearchHit)> = self
            .views
            .iter()
            .filter_map(|v| {
                let score = score_text(q, &v.name, v.description.as_deref());
                if score <= 0.0 { return None; }
                Some((score, SearchHit {
                    kind: EntityKind::View,
                    id: v.name.clone(),
                    label: v.name.clone(),
                    path: format!("/{}", v.name),
                    icon: Some("app-window".into()),
                    snippet: v.description.clone(),
                    score,
                }))
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        page(scored, self.name(), opts)
    }
}

pub(crate) fn page(
    scored: Vec<(f64, SearchHit)>,
    adapter: &str,
    opts: &SearchOpts,
) -> AdapterResult {
    let start = opts.cursor.as_ref().map(|c| c.get(adapter)).unwrap_or(0);
    let limit = opts.limit_or_default();
    let total = scored.len();
    let hits: Vec<SearchHit> = scored.into_iter().skip(start).take(limit).map(|(_, h)| h).collect();
    let consumed = start + hits.len();
    let next_offset = if consumed < total { Some(consumed) } else { None };
    AdapterResult { hits, next_offset }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ViewsAdapter {
        ViewsAdapter::new(vec![
            ViewSnapshot { name: "dashboard".into(), description: Some("overview home".into()) },
            ViewSnapshot { name: "settings".into(), description: Some("config provider theme".into()) },
            ViewSnapshot { name: "counter".into(), description: None },
        ])
    }

    #[tokio::test]
    async fn exact_name_match_ranks_top() {
        let a = fixture();
        let r = a.query("settings", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert!(!r.hits.is_empty());
        assert_eq!(r.hits[0].id, "settings");
        assert!(matches!(r.hits[0].kind, EntityKind::View));
        assert_eq!(r.hits[0].path, "/settings");
    }

    #[tokio::test]
    async fn description_match_finds_view() {
        let a = fixture();
        let r = a.query("provider", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert!(r.hits.iter().any(|h| h.id == "settings"));
    }

    #[tokio::test]
    async fn no_match_returns_empty() {
        let a = fixture();
        let r = a.query("xyzzy", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert!(r.hits.is_empty());
        assert!(r.next_offset.is_none());
    }
}
