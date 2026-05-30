//! Skills adapter — searches a snapshot of skill ids/names/descriptions.
//! Optionally matches against SKILL.md content when the snapshot
//! includes it.

use async_trait::async_trait;

use crate::adapter::{AdapterResult, EntitySearchAdapter};
use crate::ranking::score_text;
use crate::types::{EntityKind, EntityKindFilter, SearchHit, SearchOpts};

#[derive(Clone, Debug)]
pub struct SkillSnapshot {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    /// Body of SKILL.md (caller-provided, optional). Searched at lower
    /// weight than name/description.
    pub body: Option<String>,
}

pub struct SkillsAdapter {
    pub skills: Vec<SkillSnapshot>,
}

impl SkillsAdapter {
    pub fn new(skills: Vec<SkillSnapshot>) -> Self { Self { skills } }
}

#[async_trait]
impl EntitySearchAdapter for SkillsAdapter {
    fn name(&self) -> &'static str { "skill" }
    fn kind_filter(&self) -> EntityKindFilter { EntityKindFilter::Skill }

    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult {
        let mut scored: Vec<(f64, SearchHit)> = self
            .skills
            .iter()
            .filter_map(|s| {
                let primary = s.name.as_deref().unwrap_or(&s.id);
                let mut score = score_text(q, primary, s.description.as_deref())
                    .max(score_text(q, &s.id, None));
                // Body match contributes the description-tier score at most.
                if score <= 0.0
                    && let Some(body) = s.body.as_deref()
                    && !q.trim().is_empty()
                    && body.to_lowercase().contains(&q.trim().to_lowercase())
                {
                    score = 100.0;
                }
                if score <= 0.0 { return None; }
                Some((score, SearchHit {
                    kind: EntityKind::Skill,
                    id: s.id.clone(),
                    label: s.name.clone().unwrap_or_else(|| s.id.clone()),
                    path: format!("/skills/{}", s.id),
                    icon: Some("sparkles".into()),
                    snippet: s.description.clone(),
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
    async fn body_match_when_no_name_or_desc_match() {
        let a = SkillsAdapter::new(vec![SkillSnapshot {
            id: "no-slop".into(),
            name: Some("No slop".into()),
            description: Some("rules".into()),
            body: Some("Ten commandments for shipping software with AI agents.".into()),
        }]);
        let r = a.query("commandments", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(r.hits.len(), 1);
        assert_eq!(r.hits[0].id, "no-slop");
    }

    #[tokio::test]
    async fn description_match() {
        let a = SkillsAdapter::new(vec![SkillSnapshot {
            id: "review".into(),
            name: Some("Review".into()),
            description: Some("code review skill".into()),
            body: None,
        }]);
        let r = a.query("code", &SearchOpts { limit: 10, ..Default::default() }).await;
        assert_eq!(r.hits.len(), 1);
    }
}
