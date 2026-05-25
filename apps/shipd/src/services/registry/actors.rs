//! Single-source `actors.*` ops — descriptors + dispatch via `ship_ops!`.
//!
//! `actors.list` is dispatchable but intentionally absent from `schema::OPS`
//! (not advertised in the public catalog); `ACTORS_OPS` is therefore not folded
//! into `OPS`. The declaration still single-sources its descriptor + dispatch.

use super::wire::ActorsListResponse;
use super::RegistryError;

ship_ops! {
    ACTORS_OPS;
    {
        name: "actors.list",
        desc: "Live actors from the projection, filterable by workspace/kind/status.",
        perm: "ship.actors.read",
        args: &[],
        ret: ActorsListResponse,
        run: |args, _state, _files| {
            let filter = ship_substrate::projections::actor::ListActorsFilter {
                workspace_id: args.get("workspace_id").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(String::from),
                kind: args.get("kind").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(String::from),
                status: args.get("status").and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()).map(String::from),
                limit: args.get("limit").and_then(|v| v.as_i64()).unwrap_or(0),
            };
            let actors = ship_substrate::projections::actor::list_actors(&filter)
                .map_err(|e| RegistryError::service(format!("actors.list: {e}")))?;
            Ok(ActorsListResponse { items: actors })
        },
    },
}
