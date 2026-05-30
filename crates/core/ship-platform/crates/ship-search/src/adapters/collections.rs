//! Collections adapter — searches schema id + name.

use async_trait::async_trait;

use crate::adapter::{AdapterResult, EntitySearchAdapter};
use crate::ranking::score_text;
use crate::types::{EntityKind, EntityKindFilter, SearchHit, SearchOpts};

#[derive(Clone, Debug)]
pub struct CollectionSnapshot {
    pub id: String,
    pub name: Option<String>,
}

pub struct CollectionsAdapter {
    pub collections: Vec<CollectionSnapshot>,
}

impl CollectionsAdapter {
    pub fn new(collections: Vec<CollectionSnapshot>) -> Self { Self { collections } }
}

#[async_trait]
impl EntitySearchAdapter for CollectionsAdapter {
    fn name(&self) -> &'static str { "collection" }
    fn kind_filter(&self) -> EntityKindFilter { EntityKindFilter::Collection }

    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
        let mut scored: Vec<(f64, SearchHit)> = self
            .collections
            .iter()
            .filter_map(|c| {
                let primary = c.name.as_deref().unwrap_or(&c.id);
                let score = score_text(q, primary, None)
                    .max(score_text(q, &c.id, None));
                if score <= 0.0 { return None; }
                Some((score, SearchHit {
                    kind: EntityKind::Collection,
                    id: c.id.clone(),
                    label: c.name.clone().unwrap_or_else(|| c.id.clone()),
                    path: format!("/collections/{}", c.id),
                    icon: Some("folder".into()),
                    snippet: None,
                    score,
                }))
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        super::views::page(scored, self.name(), opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn matches_id_and_name() {
        let a = CollectionsAdapter::new(vec![
            CollectionSnapshot { id: "notes".into(), name: Some("Notes".into()) },
            CollectionSnapshot { id: "tasks".into(), name: None },
        ]);
        let r = a.query("note", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert!(r.hits.iter().any(|h| h.id == "notes"));
        let r2 = a.query("tasks", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert!(r2.hits.iter().any(|h| h.id == "tasks"));
    }
}
