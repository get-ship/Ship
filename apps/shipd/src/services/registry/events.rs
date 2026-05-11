//! Registry dispatcher for `events.*` ops. Split off from
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
        "events.list" => {
            let limit = optional_usize(args, "limit", 200);
            let filter = crate::views::events_state::EventFilter {
                event_type: args.get("event_type").and_then(|v| v.as_str()).map(String::from),
                actor: args.get("actor").and_then(|v| v.as_str()).map(String::from),
                entity: args.get("entity").and_then(|v| v.as_str()).map(String::from),
                q: args.get("q").and_then(|v| v.as_str()).map(String::from),
            };
            let rows = crate::views::events_state::list_events(state.ship_dir(), &filter, limit);
            let types = crate::views::events_state::distinct_types(&rows);
            let actors = crate::views::events_state::distinct_actors(&rows);
            let groups = crate::views::events_state::group_by_entity(rows);
            Ok(json!({
                "groups": groups,
                "types": types,
                "actors": actors,
            }))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
