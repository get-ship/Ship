//! L1 impl of the domain-free [`ProjectionReader`] port.
//!
//! Collapses the runner's `crate::projections` reach-ups into primitive
//! results: the union of dispatch-owning slugs (for the coherence sweep's
//! orphan check) and a terminal-status boolean (for the router's
//! terminal-state guard). The domain projection structs (`PipelineRun`,
//! `CanonicalSession`) never cross the port — only `Vec<String>` / `bool` do.

use async_trait::async_trait;

use ship_agents::services::sessions::canonical_projection::load_sessions;
use crate::projections::pipeline_run::{load_pipeline_run_by_id, load_pipeline_runs, PipelineRunStatus};
use ship_code::dispatch_ports::ProjectionReader;
use ship_substrate::projections::actor::active_actor_ids;

/// Reads the live SQL-backed projections (`runtime.db`). Stateless — each
/// call hits the tables fresh so a stale snapshot can't mis-flag a sweep.
pub struct ProjectionsReader;

#[async_trait]
impl ProjectionReader for ProjectionsReader {
    async fn known_dispatch_slugs(&self) -> Vec<String> {
        // Three read models combine into the "known dispatch" set:
        //   - jobs (pipeline composition runs)
        //   - canonical_sessions (the dispatch primitive — bare ship use,
        //     ship dispatch, MCP dispatch_agent, pipeline steps)
        //   - actor_state (every spawned actor, incl. fanout lanes / chat /
        //     shell sessions, which have no jobs/canonical_sessions row)
        // A registered slug absent from this union is a true orphan.
        let mut slugs: Vec<String> = active_actor_ids().unwrap_or_default().into_iter().collect();
        if let Ok(jobs) = load_pipeline_runs() {
            slugs.extend(jobs.into_values().map(|j| j.slug));
        }
        if let Ok(sessions) = load_sessions() {
            slugs.extend(sessions.into_iter().filter_map(|s| s.slug));
        }
        slugs.sort();
        slugs.dedup();
        slugs
    }

    async fn is_run_terminal(&self, entity_id: &str) -> bool {
        matches!(
            load_pipeline_run_by_id(entity_id).ok().flatten().map(|r| r.status),
            Some(
                PipelineRunStatus::Failed
                    | PipelineRunStatus::Merged
                    | PipelineRunStatus::Cancelled
            )
        )
    }
}
