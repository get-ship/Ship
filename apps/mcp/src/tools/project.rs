#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ship_substrate::project::{get_active_project_global, get_project_dir, set_active_project_global};
use ship_substrate::project::{get_project_name, list_registered_projects};
use ship_platform::config::get_config;
use ship_platform::sessions::get_active_session as runtime_get_active_session;
use ship_substrate::events::read_recent_events;
use ship_substrate::log::read_log;
use tokio::sync::Mutex;

/// Set the active project; returns (response_message, resolved_project_root).
#[allow(dead_code)] // MCP tool implementation not yet wired to server router
pub async fn open_project(
    path: &str,
    active_project: &Arc<Mutex<Option<PathBuf>>>,
) -> (String, Option<PathBuf>) {
    let path = PathBuf::from(path);
    match get_project_dir(Some(path.clone())) {
        Ok(ship_dir) => {
            let resolved = resolve_project_root(ship_dir.clone());
            {
                let mut lock = active_project.lock().await;
                *lock = Some(resolved.clone());
            }
            if let Err(e) = set_active_project_global(ship_dir.clone()) {
                return (
                    format!(
                        "Opened project at {} (warning: failed to persist global active project: {})",
                        ship_dir.display(),
                        e
                    ),
                    Some(resolved),
                );
            }
            (
                format!("Opened project at {}", ship_dir.display()),
                Some(resolved),
            )
        }
        Err(e) => (format!("Error: {}", e), None),
    }
}

/// Resolve the effective project directory (parent of `.ship/`).
///
/// `actor_id` is the bridge-resolved actor for this MCP connection
/// (see `ShipServer::resolve_actor_id`). When set and the actor has a
/// `project_id`, that's the source of truth — per-actor scope replaces
/// the legacy `active_project` singleton. Falls back to the singleton
/// + cwd + global + single-registered chain for connections without an
/// associated actor (external IDE clients etc).
pub async fn get_effective_project_dir(
    actor_id: &str,
    active_project: &Arc<Mutex<Option<PathBuf>>>,
) -> Result<PathBuf, String> {
    if !actor_id.is_empty()
        && let Ok(Some(actor)) = ship_substrate::projections::actor::get_actor(actor_id)
        && let Some(project_id) = actor.project_id.as_deref()
        && let Ok(Some(project)) = ship_platform::projects::get(project_id)
        && let Ok(dir) = get_project_dir(Some(PathBuf::from(&project.path)))
    {
        return Ok(resolve_project_root(dir));
    }

    {
        let active = active_project.lock().await;
        if let Some(ref path) = *active {
            return Ok(path.clone());
        }
    }

    if let Ok(dir) = get_project_dir(None) {
        return Ok(resolve_project_root(dir));
    }

    if let Ok(Some(global_active)) = get_active_project_global()
        && let Ok(dir) = get_project_dir(Some(global_active.clone()))
    {
        return Ok(resolve_project_root(dir));
    }

    if let Ok(registry) = list_registered_projects()
        && registry.len() == 1
        && let Ok(dir) = get_project_dir(Some(registry[0].path.clone()))
    {
        return Ok(resolve_project_root(dir));
    }

    get_project_dir(None)
        .map(resolve_project_root)
        .map_err(|e| {
            format!(
                "No active project and auto-detection failed: {}. Checked actor scope, process cwd, global active project, and registered projects.",
                e
            )
        })
}

fn resolve_project_root(ship_dir: PathBuf) -> PathBuf {
    if ship_dir.file_name().and_then(|n| n.to_str()) == Some(".ship") {
        ship_dir.parent().unwrap_or(&ship_dir).to_path_buf()
    } else {
        ship_dir
    }
}

/// Resolve the active agent for a project from the compile-state projection.
/// Returns None when the project has never been compiled. The argument is
/// the project root (with `.ship/` appended internally to find ship.jsonc).
fn active_agent_for_project(project_dir: &Path) -> Option<String> {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(project_dir);
    let project_id = ship_substrate::project::ensure_project_id(&ship_dir).ok()?;
    ship_platform::projections::compile_state::load(&project_id)
        .ok()
        .flatten()
        .and_then(|s| s.active_agent)
}

/// Build the full project context snapshot used by resources.
pub async fn get_project_info(project_dir: &Path) -> String {
    let name = get_project_name(project_dir);
    let config = get_config(Some(project_dir.to_path_buf())).unwrap_or_default();

    let mut out = format!("# Project: {}\n\n", name);
    out.push_str("## Current Context\n");

    // Resolve current branch via git; show its session (if any).
    let current_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(branch) = current_branch {
        out.push_str(&format!("- Branch: {}\n", branch));
        match runtime_get_active_session(project_dir, &branch) {
            Ok(Some(session)) => {
                out.push_str(&format!("- Session: ACTIVE (id: {})", session.id));
                if let Some(ref goal) = session.goal {
                    out.push_str(&format!(" — goal: {}", goal));
                }
                if let Some(ref harness) = session.primary_harness {
                    out.push_str(&format!(" [{}]", harness));
                }
                out.push('\n');
            }
            Ok(None) => out.push_str("- Session: none (call start_session to begin)\n"),
            Err(_) => {}
        }
    } else {
        out.push_str("- Session: none (call start_session to begin)\n");
    }

    let active_agent_id = active_agent_for_project(project_dir);
    if let Some(active_id) = active_agent_id.as_deref() {
        if let Some(agent) = config.agents.iter().find(|a| a.id == active_id) {
            out.push_str(&format!("- Agent: {} ({})\n", agent.name, agent.id));
        }
    } else if !config.agents.is_empty() {
        out.push_str("- Agent: none (available: ");
        let names: Vec<_> = config.agents.iter().map(|a| a.id.as_str()).collect();
        out.push_str(&names.join(", "));
        out.push_str(")\n");
    }

    if let Ok(log) = read_log(project_dir) {
        let recent: Vec<&str> = log
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if !recent.is_empty() {
            out.push_str("\n## Recent Activity\n");
            for line in recent {
                out.push_str(&format!("{}\n", line));
            }
        }
    }

    let _ = project_dir;
    if let Ok(events) = read_recent_events(10)
        && !events.is_empty()
    {
        out.push_str("\n## Recent Events\n");
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
    }

    out
}
