//! Body-parsing + formatting helpers for specs. Pure functions.
//!
//! Split-out concerns:
//!  - section splitter over `## Heading` H2s in the spec body.
//!  - acceptance checklist parser (GFM `- [x]` / `- [ ]`).
//!  - status → tone mapping, "when" relative-time formatter, verb map.
//!  - minimal JS-string escaping for inline signal inputs.

use chrono::{DateTime, Utc};
use serde::Serialize;

/// One acceptance checklist item from the `## Acceptance` section.
#[derive(Debug, Clone, Serialize)]
pub struct AcceptanceItem {
    pub text: String,
    pub done: bool,
}

/// Map a spec status to a `tone` attribute accepted by `<ship-status-dot>`.
pub fn tone_for(status: &str) -> &'static str {
    match status {
        "completed" => "success",
        "ready" => "accent",
        "launched" => "warning",
        "failed" => "destructive",
        _ => "muted",
    }
}

/// "last <verb>" label used in the rail row's when line.
pub fn verb_for(status: &str) -> &'static str {
    match status {
        "ready" => "ready",
        "launched" => "launched",
        "completed" => "merged",
        _ => "updated",
    }
}

/// Relative-time formatter — "just now", "3m ago", "2h ago", "yesterday",
/// "4d ago".
pub fn human_since(ts: DateTime<Utc>) -> String {
    let now = Utc::now();
    let secs = now.signed_duration_since(ts).num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3_600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3_600)
    } else if secs < 86_400 * 2 {
        "yesterday".to_string()
    } else {
        format!("{}d ago", secs / 86_400)
    }
}

/// Minimal JS-string escape for inline signal defaults in templates.
/// The template uses single-quoted JS literals, so we escape `\`, `'`, `\n`
/// and strip `\r`.
pub fn js_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "")
}

/// Split a markdown body into (sections_by_lowercase_heading, prelude).
///
/// `prelude` captures anything above the first H2. Sections are keyed by
/// the lowercased H2 text (e.g. `goal`, `acceptance`, `scope`, `linked`).
pub fn split_sections(
    body: &str,
) -> (std::collections::HashMap<String, String>, String) {
    let mut sections: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut prelude = String::new();
    let mut current_key: Option<String> = None;
    let mut buf = String::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(k) = current_key.take() {
                sections.insert(k, std::mem::take(&mut buf).trim().to_string());
            } else if !buf.is_empty() {
                prelude = std::mem::take(&mut buf).trim().to_string();
            }
            current_key = Some(rest.trim().to_lowercase());
        } else {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if let Some(k) = current_key.take() {
        sections.insert(k, buf.trim().to_string());
    } else if !buf.is_empty() && prelude.is_empty() {
        prelude = buf.trim().to_string();
    }
    (sections, prelude)
}

/// Parse an `## Acceptance` section body into checklist items. Non-check
/// lines are ignored. Standard GFM: `- [x] done`, `- [ ] open`.
pub fn parse_acceptance(section: &str) -> Vec<AcceptanceItem> {
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix("- ")?;
            let (done, text) = if let Some(t) = rest.strip_prefix("[x] ") {
                (true, t)
            } else if let Some(t) = rest.strip_prefix("[X] ") {
                (true, t)
            } else if let Some(t) = rest.strip_prefix("[ ] ") {
                (false, t)
            } else {
                return None;
            };
            Some(AcceptanceItem {
                text: text.trim().to_string(),
                done,
            })
        })
        .collect()
}

/// Glue "extra" sections (anything not goal/acceptance/scope/linked)
/// back into one markdown blob for raw rendering.
pub fn glue_other_sections(
    sections: &std::collections::HashMap<String, String>,
    known: &[&str],
) -> String {
    let mut other = String::new();
    for (k, v) in sections {
        if known.contains(&k.as_str()) {
            continue;
        }
        other.push_str("## ");
        let mut chars = k.chars();
        if let Some(c) = chars.next() {
            other.extend(c.to_uppercase());
            other.push_str(chars.as_str());
        }
        other.push('\n');
        other.push_str(v);
        other.push_str("\n\n");
    }
    other.trim().to_string()
}

/// Resolve a pipeline's step count + first-agent string. Returns `(0, None)`
/// when the pipeline isn't loadable (missing or malformed) — view is
/// expected to render `—` in that case.
pub fn pipeline_step_count(name: &str) -> (usize, Option<String>) {
    use ship_primitives::services::orchestrator::pipeline_loader;
    let payload = ship_substrate::events::job::JobCreatedPayload {
        job_id: String::new(),
        slug: String::new(),
        agent: String::new(),
        branch: String::new(),
        spec_path: String::new(),
        source_spec_path: None,
        plan_id: None,
        model: None,
        provider: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: Some(name.to_string()),
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
        baseline_sha: None,
    };
    let resolved = match pipeline_loader::ensure_pipeline(payload) {
        Ok(p) => p,
        Err(_) => return (0, None),
    };
    let steps = match resolved.steps.as_ref() {
        Some(p) => &p.steps,
        None => return (0, None),
    };
    let first_agent = steps
        .iter()
        .find(|s| s.agent.is_some())
        .and_then(|s| s.agent.clone());
    (steps.len(), first_agent)
}

/// Launch-bar label. Combines pipeline + first agent when both present.
pub fn launch_label(pipeline: Option<&str>, first_agent: Option<&str>) -> String {
    match (pipeline, first_agent) {
        (Some(p), Some(a)) => format!("Launch {p} → {a}"),
        (Some(p), None) => format!("Launch {p}"),
        _ => "Launch".to_string(),
    }
}

/// Launch-bar hint — describes what clicking `Launch` will do. Empty
/// pipeline nudges the user toward the dropdown.
pub fn launch_hint(pipeline: Option<&str>, first_agent: Option<&str>) -> String {
    match first_agent {
        Some(agent) => format!(
            "Will create worktree job/{}-… and dispatch {} with this spec.",
            pipeline.unwrap_or("spec"),
            agent
        ),
        None => "Select a pipeline above to enable launch.".to_string(),
    }
}

/// Human-readable status label: capitalize first letter.
pub fn status_label(status: &str) -> String {
    if status.is_empty() {
        return String::new();
    }
    let mut c = status.chars();
    let first = c.next().unwrap().to_uppercase().to_string();
    first + c.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_sections_captures_goal_and_acceptance() {
        let body = "## Goal\n\nThe goal.\n\n## Acceptance\n- [x] first\n- [ ] second\n";
        let (sections, _) = split_sections(body);
        assert_eq!(sections.get("goal").map(String::as_str), Some("The goal."));
        assert!(sections.contains_key("acceptance"));
    }

    #[test]
    fn parse_acceptance_reads_gfm_checkboxes() {
        let body = "- [x] first\n- [X] second\n- [ ] third\n- not a check";
        let items = parse_acceptance(body);
        assert_eq!(items.len(), 3);
        assert!(items[0].done);
        assert!(items[1].done);
        assert!(!items[2].done);
    }

    #[test]
    fn tone_and_verb_map() {
        assert_eq!(tone_for("completed"), "success");
        assert_eq!(tone_for("ready"), "accent");
        assert_eq!(tone_for("launched"), "warning");
        assert_eq!(tone_for("draft"), "muted");
        assert_eq!(verb_for("completed"), "merged");
        assert_eq!(verb_for("draft"), "updated");
    }

    #[test]
    fn launch_label_variants() {
        assert_eq!(launch_label(Some("tdd"), Some("writer")), "Launch tdd → writer");
        assert_eq!(launch_label(Some("tdd"), None), "Launch tdd");
        assert_eq!(launch_label(None, None), "Launch");
    }

    #[test]
    fn status_label_capitalizes() {
        assert_eq!(status_label("draft"), "Draft");
        assert_eq!(status_label(""), "");
    }
}
