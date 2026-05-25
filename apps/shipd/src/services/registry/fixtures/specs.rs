//! `specs.*` fixtures (list / get / summaries / detail).

use std::collections::HashMap;

use serde_json::Value;

use crate::services::registry::wire::{SpecGetResponse, SpecsListResponse};
use crate::services::specs::parse::AcceptanceItem;
use crate::services::specs::shape::Spec;
use crate::services::specs::view_state::{Counts, RailGroup, SpecDetail, SpecSummary, SummariesCtx};

/// Fixed instant so committed fixtures diff cleanly (2026-01-01T00:00:00Z).
fn fixed_dt() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_767_225_600, 0).expect("valid timestamp")
}

fn spec() -> Spec {
    // Single-entry map keeps serialization deterministic (HashMap order).
    let mut vars = HashMap::new();
    vars.insert("area".to_string(), "daemon".to_string());
    Spec {
        id: "typed-ops".to_string(),
        title: "Typed-op single source".to_string(),
        body_path: ".ship/specs/typed-ops.md".to_string(),
        status: "completed".to_string(),
        pipeline: Some("ship-feature".to_string()),
        agent: Some("rust-lane".to_string()),
        vars,
        criteria: vec!["Every op declared once".to_string()],
        scope: vec!["apps/shipd".to_string()],
        priority: Some(1),
        created_at: fixed_dt(),
        updated_at: fixed_dt(),
    }
}

fn spec_summary() -> SpecSummary {
    SpecSummary {
        id: "typed-ops".to_string(),
        title: "Typed-op single source".to_string(),
        status: "completed".to_string(),
        pipeline: Some("ship-feature".to_string()),
        tone: "success",
        when: "2026-01-01".to_string(),
    }
}

impl SummariesCtx {
    pub fn example() -> Self {
        SummariesCtx {
            selected_id: Some("typed-ops".to_string()),
            q: String::new(),
            counts: Counts { draft: 0, ready: 0, launched: 0, completed: 1 },
            groups: vec![RailGroup {
                key: "completed",
                label: "Completed",
                count: 1,
                items: vec![spec_summary()],
            }],
            total: 1,
            pipelines: vec!["ship-feature".to_string()],
        }
    }
}

impl SpecDetail {
    pub fn example() -> Self {
        SpecDetail {
            id: "typed-ops".to_string(),
            title: "Typed-op single source".to_string(),
            body: "# Typed-op single source\n\nKill the shadow registry.".to_string(),
            status: "completed".to_string(),
            status_label: "Completed".to_string(),
            pipeline: Some("ship-feature".to_string()),
            pipeline_label: "ship-feature".to_string(),
            agent: "Rust Lane".to_string(),
            agent_id: "rust-lane".to_string(),
            vars_display: "area=daemon".to_string(),
            branch: "job/typed-ops".to_string(),
            tone: "success",
            goal_md: "Make the typed operation the single source.".to_string(),
            scope_md: "apps/shipd, ship-platform services, xtask".to_string(),
            linked_md: String::new(),
            acceptance: vec![
                AcceptanceItem { text: "Every op declared once".to_string(), done: true },
                AcceptanceItem { text: "No anonymous json!".to_string(), done: true },
            ],
            other_md: String::new(),
            launch_label: "Re-run".to_string(),
            launch_hint: "Dispatch ship-feature for this spec".to_string(),
            title_js: "Typed-op single source".to_string(),
            body_js: "# Typed-op single source".to_string(),
            pipeline_js: "ship-feature".to_string(),
        }
    }
}

pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    let get = SpecGetResponse {
        spec: spec(),
        body: "# Typed-op single source\n\nKill the shadow registry.".to_string(),
    };
    vec![
        ("specs.list", v(SpecsListResponse { items: vec![spec()] })),
        ("specs.get", v(get)),
        ("specs.summaries", v(SummariesCtx::example())),
        ("specs.detail", v(SpecDetail::example())),
    ]
}
