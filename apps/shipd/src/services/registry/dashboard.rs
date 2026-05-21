//! Single-source `dashboard.*` ops — descriptors + dispatch via `ship_ops!`.

use serde_json::json;

use super::wire::DashboardStateResponse;

ship_ops! {
    DASHBOARD_OPS;
    {
        name: "dashboard.state",
        desc: "Aggregate stats + recent runs + events + agents + collections.",
        perm: "ship.dashboard.read",
        args: &[],
        ret: DashboardStateResponse,
        run: |_args, state, _files| {
            let ship_dir = state.ship_dir();
            // Jobs: split into active / history for stats + recent feed.
            let runs = crate::views::state::list_pipeline_runs(
                ship_dir,
                &crate::views::state::Filter::default(),
            );
            let (active, history) = crate::views::state::partition_runs(runs);
            let stats = json!({ "active": active.len(), "history": history.len() });
            let recent_runs = if !active.is_empty() {
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
            let project_dir = ship_substrate::project::get_project_dir(None).ok();
            let mut schemas = ship_platform::collections::builtin::all_schemas();
            schemas.extend(
                ship_platform::collections::schema::load_schemas_merged(ship_dir, project_dir.as_deref())
                    .unwrap_or_default(),
            );
            let collections: Vec<_> = schemas
                .iter()
                .map(|s| json!({ "id": s.id, "name": s.name, "field_count": s.fields.len() }))
                .collect();
            Ok(DashboardStateResponse {
                stats,
                recent_runs,
                events,
                total_events,
                agents,
                collections,
            })
        },
    },
}
