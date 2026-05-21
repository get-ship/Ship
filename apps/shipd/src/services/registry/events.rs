//! Single-source `events.*` ops — descriptors + dispatch via `ship_ops!`.

use super::optional_usize;
use super::wire::EventsListResponse;
use crate::services::schema::ArgDescriptor;

ship_ops! {
    EVENTS_OPS;
    {
        name: "events.list",
        desc: "Kernel events with filters, grouped by entity.",
        perm: "ship.events.read",
        args: &[
            ArgDescriptor { name: "limit", kind: "number?", required: false },
            ArgDescriptor { name: "event_type", kind: "string?", required: false },
            ArgDescriptor { name: "actor", kind: "string?", required: false },
            ArgDescriptor { name: "entity", kind: "string?", required: false },
            ArgDescriptor { name: "q", kind: "string?", required: false },
        ],
        ret: EventsListResponse,
        sensitive: true,
        run: |args, state, _files| {
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
            Ok(EventsListResponse { groups, types, actors })
        },
    },
}
