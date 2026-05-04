//! View-shaped builders for the Studio specs view. Take pre-loaded
//! `Spec` records and shape them into rail + detail payloads that
//! templates consume directly (minijinja 2.x can't mutate maps, so all
//! shaping must land here).
//!
//! Pure functions. Callers (HTTP routes, `specs.*` registry) own the
//! query against the `specs` collection.

use crate::services::specs::shape::Spec;
use serde::Serialize;

use super::parse::{
    glue_other_sections, human_since, js_escape, launch_hint, launch_label,
    parse_acceptance, pipeline_step_count, split_sections, status_label, tone_for,
    verb_for, AcceptanceItem,
};

/// Row in the left rail.
#[derive(Debug, Clone, Serialize)]
pub struct SpecSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub pipeline: Option<String>,
    pub tone: &'static str,
    pub when: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Counts {
    pub draft: usize,
    pub ready: usize,
    pub launched: usize,
    pub completed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RailGroup {
    pub key: &'static str,
    pub label: &'static str,
    pub count: usize,
    pub items: Vec<SpecSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummariesCtx {
    pub selected_id: Option<String>,
    pub q: String,
    pub counts: Counts,
    pub groups: Vec<RailGroup>,
    pub total: usize,
    pub pipelines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecDetail {
    pub id: String,
    pub title: String,
    pub body: String,
    pub status: String,
    pub status_label: String,
    pub pipeline: Option<String>,
    pub pipeline_label: String,
    pub agent: String,
    pub vars_display: String,
    pub branch: String,
    pub tone: &'static str,
    pub goal_md: String,
    pub scope_md: String,
    pub linked_md: String,
    pub acceptance: Vec<AcceptanceItem>,
    pub other_md: String,
    pub launch_label: String,
    pub launch_hint: String,
    pub title_js: String,
    pub body_js: String,
    pub pipeline_js: String,
}

fn matches_query(s: &Spec, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let needle = q.to_lowercase();
    s.title.to_lowercase().contains(&needle) || s.id.to_lowercase().contains(&needle)
}

fn to_summary(s: &Spec) -> SpecSummary {
    let when = format!("{} · {}", verb_for(&s.status), human_since(s.updated_at));
    SpecSummary {
        id: s.id.clone(),
        title: if s.title.is_empty() {
            s.id.clone()
        } else {
            s.title.clone()
        },
        status: s.status.clone(),
        pipeline: s.pipeline.clone(),
        tone: tone_for(&s.status),
        when,
    }
}

fn counts_of(all: &[Spec]) -> Counts {
    let mut c = Counts::default();
    for s in all {
        match s.status.as_str() {
            "draft" => c.draft += 1,
            "ready" => c.ready += 1,
            "launched" => c.launched += 1,
            "completed" => c.completed += 1,
            _ => {}
        }
    }
    c
}

fn groups_of(all: &[Spec], q: &str) -> Vec<RailGroup> {
    let order: [(&'static str, &'static str); 4] = [
        ("draft", "Draft"),
        ("ready", "Ready"),
        ("launched", "Launched"),
        ("completed", "Completed"),
    ];
    order
        .iter()
        .map(|(key, label)| {
            let items: Vec<SpecSummary> = all
                .iter()
                .filter(|s| s.status == *key && matches_query(s, q))
                .map(to_summary)
                .collect();
            let count = all.iter().filter(|s| s.status == *key).count();
            RailGroup {
                key,
                label,
                count,
                items,
            }
        })
        .collect()
}

/// Enumerate pipeline names available to the edit dropdown.
fn list_pipelines() -> Vec<String> {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let dir = project_dir.join("pipelines");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("yaml") {
                return None;
            }
            p.file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
        .collect();
    out.sort();
    out
}

/// Build the rail context for `ship.specs.summaries(q?, selected?)`.
pub fn build_summaries(specs: &[Spec], q: &str, selected: Option<&str>) -> SummariesCtx {
    SummariesCtx {
        selected_id: selected.map(|s| s.to_string()),
        q: q.to_string(),
        counts: counts_of(specs),
        groups: groups_of(specs, q),
        total: specs.len(),
        pipelines: list_pipelines(),
    }
}

/// Build the editor-pane context for `ship.specs.detail(id)`.
pub fn build_detail(spec: &Spec, body: &str) -> SpecDetail {
    let body = body.to_string();
    let (sections, prelude) = split_sections(&body);

    let goal_md = sections.get("goal").cloned().unwrap_or(prelude);
    let acceptance_md = sections.get("acceptance").cloned().unwrap_or_default();
    let scope_md = sections.get("scope").cloned().unwrap_or_default();
    let linked_md = sections.get("linked").cloned().unwrap_or_default();
    let acceptance = parse_acceptance(&acceptance_md);
    let other_md = glue_other_sections(&sections, &["goal", "acceptance", "scope", "linked"]);

    let pipeline_label = spec.pipeline.clone().unwrap_or_else(|| "—".to_string());
    let (_steps, first_agent) = match spec.pipeline.as_deref() {
        Some(name) => pipeline_step_count(name),
        None => (0, None),
    };
    let agent = first_agent.clone().unwrap_or_else(|| "—".to_string());

    let vars_display = if spec.vars.is_empty() {
        "—".to_string()
    } else {
        spec.vars
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    };

    SpecDetail {
        id: spec.id.clone(),
        title: spec.title.clone(),
        body: body.clone(),
        status: spec.status.clone(),
        status_label: status_label(&spec.status),
        pipeline: spec.pipeline.clone(),
        pipeline_label,
        agent,
        vars_display,
        branch: "auto".into(),
        tone: tone_for(&spec.status),
        goal_md,
        scope_md,
        linked_md,
        acceptance,
        other_md,
        launch_label: launch_label(spec.pipeline.as_deref(), first_agent.as_deref()),
        launch_hint: launch_hint(spec.pipeline.as_deref(), first_agent.as_deref()),
        title_js: js_escape(&spec.title),
        body_js: js_escape(&body),
        pipeline_js: js_escape(spec.pipeline.as_deref().unwrap_or("")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(id: &str, status: &str, title: &str) -> Spec {
        Spec {
            id: id.into(),
            title: title.into(),
            body_path: String::new(),
            status: status.into(),
            pipeline: None,
            vars: Default::default(),
            criteria: vec![],
            scope: vec![],
            priority: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn matches_query_is_case_insensitive() {
        let s = sample("abc-123", "draft", "Auth pair flow");
        assert!(matches_query(&s, ""));
        assert!(matches_query(&s, "auth"));
        assert!(matches_query(&s, "AUTH"));
        assert!(matches_query(&s, "abc"));
        assert!(!matches_query(&s, "xyz"));
    }

    #[test]
    fn groups_of_emits_all_four_keys_in_order() {
        let all = vec![sample("s1", "draft", "One"), sample("s2", "ready", "Two")];
        let g = groups_of(&all, "");
        assert_eq!(g.len(), 4);
        assert_eq!(g[0].key, "draft");
        assert_eq!(g[1].key, "ready");
        assert_eq!(g[2].key, "launched");
        assert_eq!(g[3].key, "completed");
        assert_eq!(g[0].items.len(), 1);
        assert_eq!(g[1].items.len(), 1);
    }

    #[test]
    fn counts_of_tallies_each_status() {
        let all = vec![
            sample("s1", "draft", "a"),
            sample("s2", "draft", "b"),
            sample("s3", "ready", "c"),
        ];
        let c = counts_of(&all);
        assert_eq!(c.draft, 2);
        assert_eq!(c.ready, 1);
        assert_eq!(c.launched, 0);
    }
}
