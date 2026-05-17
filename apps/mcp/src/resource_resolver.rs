use std::path::Path;

use ship_platform::agents::harnesses::{list_models, list_harnesses};
use ship_platform::agents::skill::{get_effective_skill, list_effective_skills};
use ship_platform::agents::skill_vars::get_skill_vars;
use ship_platform::config::get_config;
use ship_platform::sessions::{
    get_active_session as runtime_get_active_session, list_sessions as runtime_list_sessions,
};
use ship_substrate::log::read_log;
use ship_platform::projections::job::load_jobs_at;

use ship_platform::projections::job::load_jobs;

use crate::resources::render_events_resource;

/// Resolve a `ship://` URI to its text content, or `None` if not found.
///
/// `dir` is the project root (parent of `.ship/`).
pub async fn resolve_resource_uri(
    uri: &str,
    dir: &Path,
    project_info: impl std::future::Future<Output = String>,
) -> Option<String> {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir);

    if uri == "ship://project_info" {
        return Some(project_info.await);
    }

    // MCP Apps bridge: resolve `ui://ship/view-session/{id}` by fetching
    // the daemon's iframe shell. Parsing is delegated to shipd to keep the
    // URI format in one place. Only called when an MCP Apps host requests
    // the session resource — standard MCP clients never hit this path.
    if let Some(rest) = uri.strip_prefix("ui://ship/view-session/")
        && !rest.is_empty()
        && rest.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return crate::network_client::view_session_raw(rest).await.ok();
    }
    if uri == "ship://skills" {
        let entries = list_effective_skills(&ship_dir).ok()?;
        if entries.is_empty() {
            return Some("No skills found.".to_string());
        }
        let mut out = String::from("Skills:\n");
        for s in &entries {
            out.push_str(&format!("- {} ({})\n", s.id, s.name));
        }
        return Some(out);
    }
    if let Some(id) = uri.strip_prefix("ship://skills/") {
        return get_effective_skill(dir, id).ok().map(|s| {
            let mut out = s.content;
            if let Ok(Some(vars)) = get_skill_vars(dir, id) {
                out.push_str("\n\n## Variables\n\n");
                let mut keys: Vec<&String> = vars.keys().collect();
                keys.sort();
                for key in keys {
                    out.push_str(&format!("- {}: {}\n", key, vars[key]));
                }
            }
            out
        });
    }
    if uri == "ship://jobs" {
        let jobs = load_jobs_at(dir).unwrap_or_default();
        let mut records: Vec<_> = jobs.into_values().collect();
        records.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        return serde_json::to_string_pretty(&records).ok();
    }
    if let Some(id) = uri.strip_prefix("ship://jobs/") {
        let jobs = load_jobs_at(dir).unwrap_or_default();
        return jobs.get(id).and_then(|j| serde_json::to_string_pretty(j).ok());
    }
    // ship://mesh deferred to v0.3 — needs SharedMeshRegistry wired into resolver
    if uri == "ship://log" {
        return match read_log(&ship_dir) {
            Ok(content) if content.trim().is_empty() || content.trim() == "# Project Log" => {
                Some("No log entries yet.".to_string())
            }
            Ok(content) => Some(content),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://events" {
        return render_events_resource(&ship_dir, 100);
    }
    if let Some(limit_str) = uri.strip_prefix("ship://events/") {
        let Ok(limit) = limit_str.parse::<usize>() else {
            return Some(format!("Error: invalid limit '{}'", limit_str));
        };
        return render_events_resource(&ship_dir, limit);
    }
    // Jobs resources
    if uri == "ship://jobs" {
        return resolve_jobs_list();
    }
    if let Some(id) = uri.strip_prefix("ship://jobs/") {
        return resolve_job_by_id(id);
    }
    // Mesh resource
    if uri == "ship://mesh" {
        return resolve_mesh_agents().await;
    }
    if let Some(result) = resolve_workspace_uri(uri, &ship_dir) {
        return Some(result);
    }
    if let Some(result) = resolve_components_uri(uri) {
        return Some(result);
    }
    None
}

fn resolve_components_uri(uri: &str) -> Option<String> {
    if uri == "ship://components" {
        return Some(crate::components_catalog::full_catalog().to_string());
    }
    if let Some(tag) = uri.strip_prefix("ship://components/") {
        return crate::components_catalog::component_catalog(tag);
    }
    None
}

fn resolve_jobs_list() -> Option<String> {
    match load_jobs() {
        Ok(jobs) if jobs.is_empty() => Some("[]".to_string()),
        Ok(jobs) => {
            let records: Vec<_> = jobs.into_values().collect();
            serde_json::to_string_pretty(&records).ok()
        }
        Err(err) => Some(format!("Error: {}", err)),
    }
}

fn resolve_job_by_id(id: &str) -> Option<String> {
    match ship_platform::projections::job::load_job_by_id(id) {
        Ok(Some(job)) => serde_json::to_string_pretty(&job).ok(),
        Ok(None) => Some(format!("Job '{}' not found", id)),
        Err(err) => Some(format!("Error: {}", err)),
    }
}

async fn resolve_mesh_agents() -> Option<String> {
    match crate::network_client::mesh_discover().await {
        Ok(raw) => Some(raw),
        Err(err) => Some(format!("Error: {}", err)),
    }
}

fn resolve_workspace_uri(uri: &str, ship_dir: &Path) -> Option<String> {
    if let Some(rest) = uri.strip_prefix("ship://workspaces/")
        && let Some(branch) = rest.strip_suffix("/session")
    {
        return match runtime_get_active_session(ship_dir, branch) {
            Ok(Some(session)) => serde_json::to_string_pretty(&session).ok(),
            Ok(None) => Some(format!("No active session on branch '{}'", branch)),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://sessions" {
        return match runtime_list_sessions(ship_dir, None, 50) {
            Ok(sessions) => serde_json::to_string_pretty(&sessions).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if let Some(workspace) = uri.strip_prefix("ship://sessions/") {
        return match runtime_list_sessions(ship_dir, Some(workspace), 50) {
            Ok(sessions) => serde_json::to_string_pretty(&sessions).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://agents" {
        return match get_config(Some(ship_dir.to_path_buf())) {
            Ok(config) => {
                let active_agent = ship_substrate::project::ensure_project_id(ship_dir)
                    .ok()
                    .and_then(|pid| {
                        ship_platform::projections::compile_state::load(&pid)
                            .ok()
                            .flatten()
                    })
                    .and_then(|s| s.active_agent);
                let payload = serde_json::json!({
                    "active_agent": active_agent,
                    "agents": config.agents,
                });
                serde_json::to_string_pretty(&payload).ok()
            }
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if uri == "ship://harnesses" {
        return match list_harnesses(ship_dir) {
            Ok(harnesses) => serde_json::to_string_pretty(&harnesses).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    if let Some(rest) = uri.strip_prefix("ship://harnesses/")
        && let Some(harness_id) = rest.strip_suffix("/models")
    {
        return match list_models(harness_id) {
            Ok(models) => serde_json::to_string_pretty(&models).ok(),
            Err(err) => Some(format!("Error: {}", err)),
        };
    }
    None
}
