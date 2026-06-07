//! Session projection tests — prove session state is derived from events.

#[cfg(test)]
mod session_tests {
    use crate::db::{block_on, open_runtime_db};
    use crate::events::types::event_types;
    use crate::events::EventEnvelope;
    use crate::projections::{EventBus, SessionProjection};

    fn setup() -> sqlx::SqliteConnection {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::testutil::init_project_minimal(base).unwrap();
        open_runtime_db(&ship_dir).unwrap()
    }

    fn bus() -> EventBus {
        let mut bus = EventBus::new();
        bus.register(Box::new(SessionProjection::new()));
        bus
    }

    fn session_started_event(session_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::SESSION_STARTED,
            session_id,
            &serde_json::json!({
                "goal": "test goal",
                "workspace_id": ws_id,
                "branch": "feature/proj-test",
                "agent_id": "test-agent",
                "primary_harness": "claude",
                "config_generation_at_start": 1
            }),
        )
        .unwrap()
        .with_context(Some(ws_id), Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(ws_id)
        .elevate()
    }

    fn session_ended_event(session_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::SESSION_ENDED,
            session_id,
            &serde_json::json!({
                "summary": "did things",
                "duration_secs": 120,
                "gate_result": "pass",
                "updated_workspace_ids": ["ws-other"],
                "compile_error": null
            }),
        )
        .unwrap()
        .with_context(Some(ws_id), Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(ws_id)
        .elevate()
    }

    fn session_progress_event(session_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::SESSION_PROGRESS,
            session_id,
            &serde_json::json!({"message": "working on it"}),
        )
        .unwrap()
        .with_context(Some(ws_id), Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(ws_id)
    }

    fn query_session_status(conn: &mut sqlx::SqliteConnection, id: &str) -> Option<String> {
        let rows: Vec<(String,)> = block_on(async {
            sqlx::query_as("SELECT status FROM sessions WHERE id = ?")
                .bind(id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().map(|r| r.0.clone())
    }

    fn query_session_summary(conn: &mut sqlx::SqliteConnection, id: &str) -> Option<String> {
        let rows: Vec<(Option<String>,)> = block_on(async {
            sqlx::query_as("SELECT summary FROM sessions WHERE id = ?")
                .bind(id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().and_then(|r| r.0.clone())
    }

    fn query_session_goal(conn: &mut sqlx::SqliteConnection, id: &str) -> Option<String> {
        let rows: Vec<(Option<String>,)> = block_on(async {
            sqlx::query_as("SELECT goal FROM sessions WHERE id = ?")
                .bind(id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().and_then(|r| r.0.clone())
    }

    // ── Tests ──────────────────────────────────────────────────────────

    #[test]
    fn dispatch_started_inserts_session_row() {
        let mut conn = setup();
        let bus = bus();
        let event = session_started_event("sess-proj-1", "ws-proj-session");

        bus.dispatch(&event, &mut conn).unwrap();

        let status = query_session_status(&mut conn, "sess-proj-1");
        assert_eq!(status.as_deref(), Some("active"), "started event must insert row");

        let goal = query_session_goal(&mut conn, "sess-proj-1");
        assert_eq!(goal.as_deref(), Some("test goal"));
    }

    #[test]
    fn started_payload_persists_space_id_and_kind_dimensions() {
        let mut conn = setup();
        let bus = bus();
        let event = EventEnvelope::new(
            event_types::SESSION_STARTED,
            "sess-dim-1",
            &serde_json::json!({
                "goal": "pipeline step",
                "workspace_id": "ws-dim",
                "branch": "feat/dim",
                "space_id": "proj-aaa",
                "kind": "pipeline_step",
                "pipeline_run_id": "run-1",
                "agent_id": "rust-runtime",
                "primary_harness": "codex"
            }),
        )
        .unwrap()
        .with_context(Some("ws-dim"), Some("sess-dim-1"))
        .with_actor_id("sess-dim-1")
        .elevate();

        bus.dispatch(&event, &mut conn).unwrap();

        type SessionDimRow = (Option<String>, String, Option<String>, Option<String>);
        let row: Option<SessionDimRow> =
            block_on(async {
                sqlx::query_as(
                    "SELECT space_id, kind, pipeline_run_id, chat_thread_id \
                     FROM sessions WHERE id = ?",
                )
                .bind("sess-dim-1")
                .fetch_optional(&mut conn)
                .await
            })
            .unwrap();
        let (space_id, kind, pipeline_run_id, chat_thread_id) =
            row.expect("session row missing");
        assert_eq!(space_id.as_deref(), Some("proj-aaa"));
        assert_eq!(kind, "pipeline_step");
        assert_eq!(pipeline_run_id.as_deref(), Some("run-1"));
        assert!(chat_thread_id.is_none());
    }

    #[test]
    fn started_payload_defaults_kind_to_direct_attach() {
        let mut conn = setup();
        let bus = bus();
        let event = session_started_event("sess-default-kind", "ws-dim2");
        bus.dispatch(&event, &mut conn).unwrap();

        let kind: Option<String> = block_on(async {
            sqlx::query_scalar("SELECT kind FROM sessions WHERE id = ?")
                .bind("sess-default-kind")
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(kind.as_deref(), Some("direct_attach"));
    }

    #[test]
    fn dispatch_ended_updates_session() {
        let mut conn = setup();
        let bus = bus();

        bus.dispatch(
            &session_started_event("sess-proj-2", "ws-proj-session"),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &session_ended_event("sess-proj-2", "ws-proj-session"),
            &mut conn,
        ).unwrap();

        let status = query_session_status(&mut conn, "sess-proj-2");
        assert_eq!(status.as_deref(), Some("ended"));

        let summary = query_session_summary(&mut conn, "sess-proj-2");
        assert_eq!(summary.as_deref(), Some("did things"));
    }

    #[test]
    fn dispatch_progress_is_noop() {
        let mut conn = setup();
        let bus = bus();

        bus.dispatch(
            &session_started_event("sess-proj-3", "ws-proj-session"),
            &mut conn,
        ).unwrap();

        let status_before = query_session_status(&mut conn, "sess-proj-3");
        bus.dispatch(
            &session_progress_event("sess-proj-3", "ws-proj-session"),
            &mut conn,
        ).unwrap();
        let status_after = query_session_status(&mut conn, "sess-proj-3");

        assert_eq!(status_before, status_after, "progress must not change session row");
    }

    // ── Drain event tests ──────────────────────────────────────────────

    fn drain_started_event(session_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::SESSION_DRAIN_STARTED,
            session_id,
            &serde_json::json!({ "drained_at": "2026-04-05T00:00:00Z" }),
        )
        .unwrap()
        .with_context(Some(ws_id), Some(session_id))
        .elevate()
    }

    fn drain_completed_event(session_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::SESSION_DRAIN_COMPLETED,
            session_id,
            &serde_json::json!({ "ended_at": "2026-04-05T00:01:00Z" }),
        )
        .unwrap()
        .with_context(Some(ws_id), Some(session_id))
        .elevate()
    }

    fn drain_aborted_event(session_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::SESSION_DRAIN_ABORTED,
            session_id,
            &serde_json::json!({ "resumed_at": "2026-04-05T00:00:30Z" }),
        )
        .unwrap()
        .with_context(Some(ws_id), Some(session_id))
        .elevate()
    }

    fn tool_count_event(session_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::SESSION_TOOL_COUNT_INCREMENTED,
            session_id,
            &serde_json::json!({}),
        )
        .unwrap()
        .with_context(Some(ws_id), Some(session_id))
        .elevate()
    }

    fn query_session_drain_status(
        conn: &mut sqlx::SqliteConnection,
        id: &str,
    ) -> Option<(String, Option<String>)> {
        let rows: Vec<(String, Option<String>)> = block_on(async {
            sqlx::query_as("SELECT status, drained_at FROM sessions WHERE id = ?")
                .bind(id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.into_iter().next()
    }

    fn query_tool_count(conn: &mut sqlx::SqliteConnection, id: &str) -> i64 {
        block_on(async {
            sqlx::query_scalar::<_, i64>(
                "SELECT tool_call_count FROM sessions WHERE id = ?",
            )
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
        })
        .unwrap()
        .unwrap_or(0)
    }

    #[test]
    fn drain_started_transitions_to_draining() {
        let mut conn = setup();
        let bus = bus();
        let ws = "ws-drain";

        bus.dispatch(&session_started_event("sess-drain-1", ws), &mut conn).unwrap();
        bus.dispatch(&drain_started_event("sess-drain-1", ws), &mut conn).unwrap();

        let (status, drained_at) = query_session_drain_status(&mut conn, "sess-drain-1").unwrap();
        assert_eq!(status, "draining");
        assert!(drained_at.is_some());
    }

    #[test]
    fn drain_completed_transitions_to_ended() {
        let mut conn = setup();
        let bus = bus();
        let ws = "ws-drain";

        bus.dispatch(&session_started_event("sess-drain-2", ws), &mut conn).unwrap();
        bus.dispatch(&drain_started_event("sess-drain-2", ws), &mut conn).unwrap();
        bus.dispatch(&drain_completed_event("sess-drain-2", ws), &mut conn).unwrap();

        let status = query_session_status(&mut conn, "sess-drain-2");
        assert_eq!(status.as_deref(), Some("ended"));
    }

    #[test]
    fn drain_aborted_resumes_to_active() {
        let mut conn = setup();
        let bus = bus();
        let ws = "ws-drain";

        bus.dispatch(&session_started_event("sess-drain-3", ws), &mut conn).unwrap();
        bus.dispatch(&drain_started_event("sess-drain-3", ws), &mut conn).unwrap();
        bus.dispatch(&drain_aborted_event("sess-drain-3", ws), &mut conn).unwrap();

        let (status, drained_at) = query_session_drain_status(&mut conn, "sess-drain-3").unwrap();
        assert_eq!(status, "active");
        assert!(drained_at.is_none(), "drained_at must be cleared on abort");
    }

    #[test]
    fn tool_count_increments() {
        let mut conn = setup();
        let bus = bus();
        let ws = "ws-drain";

        bus.dispatch(&session_started_event("sess-tool-1", ws), &mut conn).unwrap();
        bus.dispatch(&tool_count_event("sess-tool-1", ws), &mut conn).unwrap();
        bus.dispatch(&tool_count_event("sess-tool-1", ws), &mut conn).unwrap();
        bus.dispatch(&tool_count_event("sess-tool-1", ws), &mut conn).unwrap();

        let count = query_tool_count(&mut conn, "sess-tool-1");
        assert_eq!(count, 3);
    }

    #[test]
    fn drain_events_survive_rebuild() {
        let mut conn = setup();
        let bus = bus();
        let ws = "ws-drain-rebuild";

        let events = vec![
            session_started_event("sess-drain-rb", ws),
            drain_started_event("sess-drain-rb", ws),
            tool_count_event("sess-drain-rb", ws),
            tool_count_event("sess-drain-rb", ws),
            drain_completed_event("sess-drain-rb", ws),
        ];

        for e in &events {
            bus.dispatch(e, &mut conn).unwrap();
        }

        let status_before = query_session_status(&mut conn, "sess-drain-rb");
        let count_before = query_tool_count(&mut conn, "sess-drain-rb");

        bus.rebuild(&events, &mut conn).unwrap();

        let status_after = query_session_status(&mut conn, "sess-drain-rb");
        let count_after = query_tool_count(&mut conn, "sess-drain-rb");

        assert_eq!(status_before, status_after);
        assert_eq!(count_before, count_after);
    }

    #[test]
    fn rebuild_from_session_events_produces_identical_state() {
        let mut conn = setup();
        let bus = bus();
        let ws = "ws-proj-session";

        let events = vec![
            session_started_event("sess-rebuild-a", ws),
            session_started_event("sess-rebuild-b", ws),
            session_ended_event("sess-rebuild-a", ws),
        ];

        for event in &events {
            bus.dispatch(event, &mut conn).unwrap();
        }

        let status_a = query_session_status(&mut conn, "sess-rebuild-a");
        let status_b = query_session_status(&mut conn, "sess-rebuild-b");
        let summary_a = query_session_summary(&mut conn, "sess-rebuild-a");

        let report = bus.rebuild(&events, &mut conn).unwrap();
        assert_eq!(report.events_replayed, 3);

        let rebuilt_a = query_session_status(&mut conn, "sess-rebuild-a");
        let rebuilt_b = query_session_status(&mut conn, "sess-rebuild-b");
        let rebuilt_summary = query_session_summary(&mut conn, "sess-rebuild-a");
        assert_eq!(status_a, rebuilt_a, "rebuild must produce identical status for session A");
        assert_eq!(status_b, rebuilt_b, "rebuild must produce identical status for session B");
        assert_eq!(summary_a, rebuilt_summary, "rebuild must produce identical summary");
    }
}
