//! Registry dispatcher for `dashboard.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, html_escape, optional_usize, parse_scope, required_str};
use crate::routes::rest_api::ApiState;
use crate::services::files::{FilesService, Scope};

#[allow(unused_imports, unused_variables)]
pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "dashboard.state" => {
            let ship_dir = state.ship_dir();
            // Jobs: split into active / history for stats + recent feed.
            let runs = crate::views::state::list_pipeline_runs(
                ship_dir,
                &crate::views::state::Filter::default(),
            );
            let (active, history) = crate::views::state::partition_runs(runs);
            let stats = json!({ "active": active.len(), "history": history.len() });
            let recent_runs: Vec<_> = if !active.is_empty() {
                active.iter().take(5).cloned().collect()
            } else {
                history.iter().take(5).cloned().collect()
            };
            // Events feed.
            let filter = crate::views::events_state::EventFilter::default();
            let events = crate::views::events_state::list_events(ship_dir, &filter, 10);
            let total_events = events.len();
            // Agents + collections lists.
            let agents = crate::views::state::list_agents(ship_dir);
            let project_dir = ship_application::project::get_project_dir(None).ok();
            let mut schemas = ship_platform::collections::builtin::all_schemas();
            schemas.extend(
                ship_platform::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
                    .unwrap_or_default(),
            );
            let collections: Vec<_> = schemas.iter().map(|s| json!({
                "id": s.id, "name": s.name, "field_count": s.fields.len(),
            })).collect();
            Ok(json!({
                "stats": stats,
                "recent_runs": recent_runs,
                "events": events,
                "total_events": total_events,
                "agents": agents,
                "collections": collections,
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
