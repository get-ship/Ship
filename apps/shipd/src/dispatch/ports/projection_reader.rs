//! L1 impl of the domain-free [`ProjectionReader`] port.
//!
//! Relocated from `orchestrator/projection_reader.rs` (deleted with the
//! engine cut, 2026-07-10). The pipeline-run projection is gone, so
//! `is_run_terminal` is reduced to a `false` default — the bare-agent
//! terminal-guard no longer blocks events for "terminal pipeline runs"
//! (no pipeline runs to be terminal of); the canonical-sessions projection
//! still owns session-terminal semantics elsewhere.
//!
//! `known_dispatch_slugs` is the bare-agent orphan check's input: it
//! unions the in-process actor registry with the canonical-sessions
//! projection's slugs. A registered slug absent from this union is a
//! true orphan process the runtime can reap.

use async_trait::async_trait;

use bridge::dispatch_ports::ProjectionReader;
use ship_agents::services::sessions::canonical_projection::load_sessions;
use ship_substrate::projections::actor::active_actor_ids;

/// Reads the live SQL-backed projections (`runtime.db`). Stateless — each
/// call hits the tables fresh so a stale snapshot can't mis-flag a sweep.
pub struct ProjectionsReader;

#[async_trait]
impl ProjectionReader for ProjectionsReader {
    async fn known_dispatch_slugs(&self) -> Vec<String> {
        // Two read models combine into the "known dispatch" set:
        //   - actor_state (every spawned actor)
        //   - canonical_sessions (the dispatch primitive — bare ship use,
        //     ship dispatch, MCP dispatch_agent, pipeline steps; each
        //     session has an optional slug)
        // A registered slug absent from this union is a true orphan.
        let mut slugs: Vec<String> = active_actor_ids().unwrap_or_default().into_iter().collect();
        if let Ok(sessions) = load_sessions() {
            slugs.extend(sessions.into_iter().filter_map(|s| s.slug));
        }
        slugs.sort();
        slugs.dedup();
        slugs
    }

    // The pipeline-run projection is retired with the engine. The bare-agent
    // terminal-guard sees nothing to be terminal of; the canonical-sessions
    // projection owns session-terminal semantics out-of-band (consumers key
    // on `session.{completed,failed,cancelled}` directly). Returning `false`
    // admits every event the actor sees — the handlers themselves are
    // idempotent on already-terminal sessions, so this is safe.
    async fn is_run_terminal(&self, _entity_id: &str) -> bool {
        false
    }

    async fn slug_for_session_id(&self, session_id: &str) -> Option<String> {
        load_sessions()
            .ok()?
            .into_iter()
            .find(|s| s.session_id == session_id)
            .and_then(|s| s.slug)
    }
}
