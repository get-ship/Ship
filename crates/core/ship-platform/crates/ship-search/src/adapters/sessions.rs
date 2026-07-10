//! Sessions adapter — searches active PTY sessions by agent id +
//! workspace + status. Ended sessions are skipped by the caller when
//! populating the snapshot.

use async_trait::async_trait;

use crate::adapter::{AdapterResult, EntitySearchAdapter};
use crate::ranking::score_text;
use crate::types::{EntityKind, EntityKindFilter, SearchHit, SearchOpts};

#[derive(Clone, Debug)]
pub struct SessionSnapshot {
    pub id: String,
    pub agent_id: Option<String>,
    pub status: String,
}

pub struct SessionsAdapter {
    pub sessions: Vec<SessionSnapshot>,
}

impl SessionsAdapter {
    pub fn new(sessions: Vec<SessionSnapshot>) -> Self { Self { sessions } }
}

#[async_trait]
impl EntitySearchAdapter for SessionsAdapter {
    fn name(&self) -> &'static str { "session" }
    fn kind_filter(&self) -> EntityKindFilter { EntityKindFilter::Session }

    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
        let mut scored: Vec<(f64, SearchHit)> = self
            .sessions
            .iter()
            .filter_map(|s| {
                let primary = s.agent_id.as_deref().unwrap_or(&s.id);
                let score = score_text(q, primary, None);
                if score <= 0.0 { return None; }
                let label = match &s.agent_id {
                    Some(aid) => format!("{aid} ({})", s.status),
                    None => format!("session {}", &s.id[..s.id.len().min(8)]),
                };
                Some((score, SearchHit {
                    kind: EntityKind::Session,
                    id: s.id.clone(),
                    label,
                    path: format!("/chats?session={}", s.id),
                    icon: Some("message-square-plus".into()),
                    snippet: Some(s.status.clone()),
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
    async fn matches_agent_id() {
        let a = SessionsAdapter::new(vec![
            SessionSnapshot {
                id: "s1".into(),
                agent_id: Some("writer".into()),
                status: "idle".into(),
            },
        ]);
        let r = a.query("writer", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(r.hits.len(), 1);
        assert!(matches!(r.hits[0].kind, EntityKind::Session));
        assert_eq!(r.hits[0].id, "s1");
    }
}
