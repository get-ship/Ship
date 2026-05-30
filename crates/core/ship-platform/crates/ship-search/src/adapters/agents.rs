//! Agents adapter — searches a snapshot of agent ids/names/descriptions.

use async_trait::async_trait;

use crate::adapter::{AdapterResult, EntitySearchAdapter};
use crate::ranking::score_text;
use crate::types::{EntityKind, EntityKindFilter, SearchHit, SearchOpts};

#[derive(Clone, Debug)]
pub struct AgentSnapshot {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct AgentsAdapter {
    pub agents: Vec<AgentSnapshot>,
}

impl AgentsAdapter {
    pub fn new(agents: Vec<AgentSnapshot>) -> Self { Self { agents } }
}

#[async_trait]
impl EntitySearchAdapter for AgentsAdapter {
    fn name(&self) -> &'static str { "agent" }
    fn kind_filter(&self) -> EntityKindFilter { EntityKindFilter::Agent }

    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
        let mut scored: Vec<(f64, SearchHit)> = self
            .agents
            .iter()
            .filter_map(|a| {
                let primary = a.name.as_deref().unwrap_or(&a.id);
                let score = score_text(q, primary, a.description.as_deref())
                    .max(score_text(q, &a.id, None));
                if score <= 0.0 { return None; }
                Some((score, SearchHit {
                    kind: EntityKind::Agent,
                    id: a.id.clone(),
                    label: a.name.clone().unwrap_or_else(|| a.id.clone()),
                    path: format!("/agents/{}", a.id),
                    icon: Some("bot".into()),
                    snippet: a.description.clone(),
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

    fn fixture() -> AgentsAdapter {
        AgentsAdapter::new(vec![
            AgentSnapshot { id: "writer".into(), name: Some("Writer".into()), description: Some("drafts prose".into()) },
            AgentSnapshot { id: "reviewer".into(), name: None, description: None },
        ])
    }

    #[tokio::test]
    async fn id_substring_match() {
        let a = fixture();
        let r = a.query("review", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(r.hits.len(), 1);
        assert_eq!(r.hits[0].id, "reviewer");
        assert!(matches!(r.hits[0].kind, EntityKind::Agent));
    }

    #[tokio::test]
    async fn description_match() {
        let a = fixture();
        let r = a.query("prose", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert!(r.hits.iter().any(|h| h.id == "writer"));
    }
}
