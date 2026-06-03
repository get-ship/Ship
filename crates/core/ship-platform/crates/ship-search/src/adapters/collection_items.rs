//! Collection items adapter — searches a flat snapshot of items
//! contributed by every collection, returning hits tagged with the
//! parent collection id.
//!
//! Searches all string-valued top-level fields in `data`; if `label`
//! is provided by the snapshot it ranks against that first, otherwise
//! the item id is the fallback display label.

use async_trait::async_trait;

use crate::adapter::{AdapterResult, EntitySearchAdapter};
use crate::ranking::score_text;
use crate::types::{EntityKind, EntityKindFilter, SearchHit, SearchOpts};

#[derive(Clone, Debug)]
pub struct CollectionItemSnapshot {
    pub collection_id: String,
    pub item_id: String,
    /// Display label — typically `data.title` or `data.name` when
    /// present; falls back to `item_id` if `None`.
    pub label: Option<String>,
    /// Concatenated string fields from `data`, used as the searchable
    /// haystack and a snippet preview.
    pub searchable_text: String,
}

pub struct CollectionItemsAdapter {
    pub items: Vec<CollectionItemSnapshot>,
}

impl CollectionItemsAdapter {
    pub fn new(items: Vec<CollectionItemSnapshot>) -> Self { Self { items } }
}

#[async_trait]
impl EntitySearchAdapter for CollectionItemsAdapter {
    fn name(&self) -> &'static str { "collection_item" }
    fn kind_filter(&self) -> EntityKindFilter { EntityKindFilter::CollectionItem }

    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
        let mut scored: Vec<(f64, SearchHit)> = self
            .items
            .iter()
            .filter_map(|it| {
                let primary = it.label.as_deref().unwrap_or(&it.item_id);
                let score = score_text(q, primary, Some(&it.searchable_text));
                if score <= 0.0 { return None; }
                let snippet = if it.searchable_text.len() > 120 {
                    Some(format!("{}…", &it.searchable_text[..120]))
                } else if it.searchable_text.is_empty() {
                    None
                } else {
                    Some(it.searchable_text.clone())
                };
                Some((score, SearchHit {
                    kind: EntityKind::CollectionItem { collection_id: it.collection_id.clone() },
                    id: it.item_id.clone(),
                    label: it.label.clone().unwrap_or_else(|| it.item_id.clone()),
                    path: format!("/collections/{}/items/{}", it.collection_id, it.item_id),
                    icon: Some("file".into()),
                    snippet,
                    score,
                    space_id: String::new(),
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
    async fn item_search_returns_parent_collection_id() {
        let a = CollectionItemsAdapter::new(vec![
            CollectionItemSnapshot {
                collection_id: "notes".into(),
                item_id: "n1".into(),
                label: Some("Hello world".into()),
                searchable_text: "Hello world some body content".into(),
            },
        ]);
        let r = a.query("hello", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(r.hits.len(), 1);
        match &r.hits[0].kind {
            EntityKind::CollectionItem { collection_id } => assert_eq!(collection_id, "notes"),
            other => panic!("unexpected kind: {other:?}"),
        }
        assert_eq!(r.hits[0].path, "/collections/notes/items/n1");
    }

    #[tokio::test]
    async fn body_substring_matches() {
        let a = CollectionItemsAdapter::new(vec![
            CollectionItemSnapshot {
                collection_id: "notes".into(),
                item_id: "n1".into(),
                label: Some("Untitled".into()),
                searchable_text: "deeply nested keyword".into(),
            },
        ]);
        let r = a.query("nested", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(r.hits.len(), 1);
    }
}
