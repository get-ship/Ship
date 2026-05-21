//! `skills.*` fixtures (list / file / detail / get). `skills.summaries` is a
//! `Value`-shaped rail aggregate and stays `unknown` (no struct fixture).

use serde_json::Value;

use crate::services::registry::wire::{SkillFileResponse, SkillsIndexResponse};
use ship_platform::services::skills::state::{
    IndexEntry, ScopeTag, SkillDetail as RawSkillDetail,
};
use ship_platform::services::skills::view_state::{SkillDetail, SkillKind};

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

pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("skills.list", v(SkillsIndexResponse::example())),
        ("skills.file", v(SkillFileResponse::example())),
        ("skills.detail", v(view_detail())),
        ("skills.get", v(raw_detail())),
    ]
}
