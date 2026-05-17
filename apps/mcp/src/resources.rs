use std::path::Path;

use rmcp::model::{AnnotateAble, RawResource, RawResourceTemplate};
use ship_substrate::events::read_recent_events;

pub fn static_resource_list() -> Vec<rmcp::model::Annotated<RawResource>> {
    vec![
        RawResource::new("ship://project_info", "Project Info").no_annotation(),
        RawResource::new("ship://adrs", "ADRs").no_annotation(),
        RawResource::new("ship://skills", "Skills").no_annotation(),
        RawResource::new("ship://workspaces", "Workspaces").no_annotation(),
        RawResource::new("ship://sessions", "Workspace Sessions").no_annotation(),
        RawResource::new("ship://modes", "Modes").no_annotation(),
        RawResource::new("ship://harnesses", "Harnesses").no_annotation(),
        RawResource::new("ship://jobs", "Jobs").no_annotation(),
        RawResource::new("ship://log", "Project Log").no_annotation(),
        RawResource::new("ship://events", "Event Stream").no_annotation(),
        RawResource::new("ship://jobs", "Jobs").no_annotation(),
        RawResource::new("ship://mesh", "Mesh Agents").no_annotation(),
        components_resource(),
    ]
}

fn components_resource() -> rmcp::model::Annotated<RawResource> {
    RawResource {
        uri: "ship://components".to_string(),
        name: "Components".to_string(),
        title: Some("SUI Component Catalog".to_string()),
        description: Some(
            "Aggregated Custom Elements Manifest for every <sui-*> primitive — \
             tagName, attributes, slots, parts, events, CSS custom properties. \
             The canonical agent-side catalog source."
                .to_string(),
        ),
        mime_type: Some("application/json".to_string()),
        size: None,
        icons: None,
        meta: None,
    }
    .no_annotation()
}

fn tmpl(uri: &str, name: &str, mime: &str) -> rmcp::model::Annotated<RawResourceTemplate> {
    RawResourceTemplate {
        uri_template: uri.to_string(),
        name: name.to_string(),
        title: None,
        description: None,
        mime_type: Some(mime.to_string()),
        icons: None,
    }
    .no_annotation()
}

pub fn static_resource_template_list() -> Vec<rmcp::model::Annotated<RawResourceTemplate>> {
    vec![
        tmpl(
            "ship://session/{path}",
            "Session File",
            "application/octet-stream",
        ),
        tmpl("ship://adrs/{id}", "ADR", "text/markdown"),
        tmpl(
            "ship://workspaces/{branch}",
            "Workspace",
            "application/json",
        ),
        tmpl(
            "ship://workspaces/{branch}/harness-matrix",
            "Workspace Harness Matrix",
            "application/json",
        ),
        tmpl(
            "ship://workspaces/{branch}/session",
            "Workspace Active Session",
            "application/json",
        ),
        tmpl(
            "ship://sessions/{workspace}",
            "Workspace Sessions",
            "application/json",
        ),
        tmpl(
            "ship://harnesses/{id}/models",
            "Harness Models",
            "application/json",
        ),
        tmpl("ship://jobs/{id}", "Job", "application/json"),
        tmpl("ship://events/{limit}", "Recent Events", "text/plain"),
        tmpl("ship://skills/{id}", "Skill", "text/markdown"),
        tmpl("ship://jobs/{id}", "Job", "application/json"),
        tmpl(
            "ship://components/{tag}",
            "Component CEM",
            "application/json",
        ),
    ]
}

pub fn render_events_resource(project_dir: &Path, limit: usize) -> Option<String> {
    let _ = project_dir;
    match read_recent_events(limit) {
        Ok(events) => {
            if events.is_empty() {
                return Some("No events found.".to_string());
            }
            let mut out = String::from("Events:\n");
            for e in events {
                out.push_str(&format!(
                    "- {} {} [{}] {} {}\n",
                    e.id,
                    e.created_at.format("%Y-%m-%d %H:%M:%S"),
                    e.actor,
                    e.event_type,
                    e.entity_id,
                ));
            }
            Some(out)
        }
        Err(err) => Some(format!("Error: {}", err)),
    }
}
