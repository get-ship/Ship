//! S4 typed-boundary proof for `agent.dispatch`: the dispatch core's structured
//! data decodes onto the typed `DispatchResult`, and its `session_id` is the
//! entity id of the `agent.session.started` event the core persisted. Kept in a
//! sibling file so `tests.rs` stays under the file cap.

use super::DispatchRequest;
use crate::agent_event_ports::AgentDispatchAdapter;
use ship_agents::services::agents::dispatch_core::{DispatchCoreError, dispatch_agent_session_core};
use ship_substrate::db::events::list_recent_events_at;
use ship_substrate::events::agent_session::InvocationMode;
use std::sync::Arc;
use tempfile::TempDir;

fn bare_req(agent: &str, workspace: &str) -> DispatchRequest {
    DispatchRequest {
        agent: agent.into(),
        spec: None,
        prompt: None,
        resume: false,
        harness: None,
        model: None,
        open_tab: true,
        invocation: InvocationMode::Pty,
        keepalive: true,
        workspace: Some(workspace.into()),
        slug: None,
        branch: None,
        source_spec_path: None,
        depends_on: None,
        vars: None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_result_session_id_matches_emitted_started_event() {
    let dir = TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = ship_substrate::events::new_isolated_for_test(ship_dir.clone()).unwrap();
    let state = Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir.clone(), kernel));

    let port = AgentDispatchAdapter::new(state);
    let outcome =
        dispatch_agent_session_core(&port, bare_req("rust-lane", "match-test"))
            .await
            .expect("dispatch must succeed");

    assert!(!outcome.session_id.is_empty(), "DispatchOutcome carries a session_id");

    let events = list_recent_events_at(&ship_dir, 50).unwrap();
    let started = events
        .iter()
        .find(|e| e.event_type == "agent.session.started")
        .expect("agent.session.started was emitted");
    assert_eq!(
        started.entity_id, outcome.session_id,
        "DispatchOutcome.session_id is the started event's entity id"
    );
}
