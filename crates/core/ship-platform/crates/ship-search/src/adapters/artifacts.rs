//! Artifacts adapter — artifacts are themselves a collection in the
//! data store, so this adapter is a thin wrapper around the
//! collection-items snapshot scoped to the artifacts collection. The
//! caller passes a pre-filtered snapshot.

use async_trait::async_trait;

use crate::adapter::{AdapterResult, EntitySearchAdapter};
use crate::ranking::score_text;
use crate::types::{EntityKind, EntityKindFilter, SearchHit, SearchOpts};

#[derive(Clone, Debug)]
pub struct ArtifactSnapshot {
    pub id: String,
    pub name: String,
    pub mime: Option<String>,
    pub scope: Option<String>,
}

pub struct ArtifactsAdapter {
    pub artifacts: Vec<ArtifactSnapshot>,
}

impl ArtifactsAdapter {
    pub fn new(artifacts: Vec<ArtifactSnapshot>) -> Self { Self { artifacts } }
}

#[async_trait]
impl EntitySearchAdapter for ArtifactsAdapter {
    fn name(&self) -> &'static str { "artifact" }
    fn kind_filter(&self) -> EntityKindFilter { EntityKindFilter::Artifact }

    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
        let mut scored: Vec<(f64, SearchHit)> = self
            .artifacts
            .iter()
            .filter_map(|a| {
                let score = score_text(q, &a.name, a.mime.as_deref())
                    .max(score_text(q, &a.id, None));
                if score <= 0.0 { return None; }
                Some((score, SearchHit {
                    kind: EntityKind::Artifact,
                    id: a.id.clone(),
                    label: a.name.clone(),
                    // The artifact id is the op-shaped reference — consumers
                    // call `GetBlob` with it. (The previous shape synthesized
                    // `/api/v1/artifacts/<id>` but no route serves that.)
                    path: a.id.clone(),
                    icon: Some("file".into()),
                    snippet: a.mime.clone(),
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
    async fn matches_name_and_mime() {
        let a = ArtifactsAdapter::new(vec![
            ArtifactSnapshot {
                id: "art_1".into(),
                name: "screenshot.png".into(),
                mime: Some("image/png".into()),
                scope: Some("session".into()),
            },
        ]);
        let r = a.query("screenshot", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(r.hits.len(), 1);
        assert!(matches!(r.hits[0].kind, EntityKind::Artifact));
    }
}
