use anyhow::Result;
use ship_config::project::{AgentProfile, get_config};
use std::path::PathBuf;

/// Resolve the `AgentProfile` for the project's currently-active agent.
/// "Active" is defined as the agent recorded by the latest `compile.completed`
/// event materialized in `project_compile_state`. Returns `None` when the
/// project has never been compiled or when the recorded agent id no longer
/// exists in the project's agents list. Callers pass the ship_dir
/// (`<project>/.ship/`).
pub fn get_active_agent(project_dir: Option<PathBuf>) -> Result<Option<AgentProfile>> {
    let Some(ship_dir) = project_dir.clone() else {
        return Ok(None);
    };
    let space_id = ship_substrate::project::ensure_space_id(&ship_dir)?;
    let Some(state) = crate::services::compile::projection::load(&space_id)? else {
        return Ok(None);
    };
    let Some(active_id) = state.active_agent else {
        return Ok(None);
    };
    let config = get_config(project_dir)?;
    Ok(config.agents.into_iter().find(|a| a.id == active_id))
}
