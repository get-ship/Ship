//! `skills.*` fixtures (list / file / detail / get). `skills.summaries` is a
//! `Value`-shaped rail aggregate and stays `unknown` (no struct fixture).

use serde_json::{json, Value};

use crate::services::registry::wire::{
    SkillFileResponse, SkillsIndexResponse, SkillsSummariesResponse,
};
use ship_platform::services::skills::state::{
    IndexEntry, ScopeTag, SkillDetail as RawSkillDetail,
};
use ship_platform::services::skills::view_state::{SkillDetail, SkillKind, SkillSummary};

impl SkillsIndexResponse {
    pub fn example() -> Self {
        SkillsIndexResponse {
            items: vec![
                IndexEntry {
                    id: "code-review".to_string(),
                    scope: ScopeTag::Project,
                    name: "Code Review".to_string(),
                    description: "Structured review of a diff.".to_string(),
                },
                IndexEntry {
                    id: "tdd".to_string(),
                    scope: ScopeTag::User,
                    name: "TDD".to_string(),
                    description: "Write the failing test first.".to_string(),
                },
            ],
        }
    }
}

impl SkillFileResponse {
    pub fn example() -> Self {
        SkillFileResponse {
            path: "SKILL.md".to_string(),
            name: "SKILL.md".to_string(),
            ext: "md".to_string(),
            content: "# Code Review\n\nReview a unified diff for correctness.".to_string(),
        }
    }
}

// `SkillDetail` (view + state) are ship-platform types — constructed inline
// here (orphan rule forbids an inherent `impl` from this crate).
fn view_detail() -> SkillDetail {
    SkillDetail {
        id: "code-review".to_string(),
        name: "Code Review".to_string(),
        description: "Structured review of a diff.".to_string(),
        kind: SkillKind::Smart,
        source: "PROJECT SKILLS".to_string(),
        tags: vec!["review".to_string()],
        authors: vec!["Ship".to_string()],
        variables: vec!["focus".to_string()],
        artifacts: vec![],
        files: vec!["SKILL.md".to_string()],
        body: "# Code Review\n\nReview a unified diff for correctness.".to_string(),
    }
}

fn raw_detail() -> RawSkillDetail {
    RawSkillDetail {
        id: "code-review".to_string(),
        scope: ScopeTag::Project,
        frontmatter: serde_json::json!({ "name": "Code Review", "kind": "smart" }),
        body: "# Code Review\n\nReview a unified diff for correctness.".to_string(),
        files: vec!["SKILL.md".to_string()],
    }
}

fn skill_summary(id: &str, name: &str, kind: SkillKind, tags: &[&str]) -> SkillSummary {
    SkillSummary {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("{name} skill."),
        kind,
        source: "PROJECT SKILLS".to_string(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        files: vec!["SKILL.md".to_string()],
        has_vars: matches!(kind, SkillKind::Smart),
        has_refs: false,
        has_evals: false,
    }
}

/// `skills.summaries` rail aggregate — `Value`-shaped groups/kind_counts (so it
/// stays `DataHandle<unknown>` in data.ts), but a real `SkillSummary`-backed
/// fixture mirroring the registry handler's shape.
fn summaries() -> SkillsSummariesResponse {
    let items = vec![
        skill_summary("code-review", "Code Review", SkillKind::Smart, &["review"]),
        skill_summary("ship-architecture", "Ship Architecture", SkillKind::Docs, &["docs"]),
    ];
    let tags: std::collections::BTreeSet<String> =
        items.iter().flat_map(|s| s.tags.clone()).collect();
    let total = items.len();
    SkillsSummariesResponse {
        groups: vec![json!({ "source": "PROJECT SKILLS", "skills": items.clone() })],
        kind_counts: vec![
            json!({ "id": "all", "label": "All", "count": total, "selected": true }),
            json!({ "id": "smart", "label": "Smart", "count": 1, "selected": false }),
            json!({ "id": "docs", "label": "Docs", "count": 1, "selected": false }),
        ],
        filtered: items.clone(),
        filtered_count: total,
        items,
        tags,
        total,
        q: String::new(),
        active_kind: "all".to_string(),
        active_tag: String::new(),
    }
}

pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("skills.list", v(SkillsIndexResponse::example())),
        ("skills.summaries", v(summaries())),
        ("skills.file", v(SkillFileResponse::example())),
        ("skills.detail", v(view_detail())),
        ("skills.get", v(raw_detail())),
    ]
}
