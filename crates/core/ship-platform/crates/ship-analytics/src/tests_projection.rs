//! AnalyticsProjection behavioral tests — verify event-fold into rollup tables.
//!
//! #25b: hook.* events replaced by canonical op.* (tool-use) and
//! session.turn.ended (turns). Session lifecycle driven by actor.created/stopped.
//! Two test shapes:
//!   hook-POST path  — op.* emitted by the http_ingress (third-party harness config)
//!   synth path      — op.tool_use_started + session.turn.ended from process.rs
//!                     (opencode / cursor / harnesses without hooks)

#[cfg(test)]
mod analytics_tests {
    use agent_def::canonical_session::TurnEnded;
    use agent_def::op::{OpToolUseCompleted, OpToolUseStarted};
    use ship_substrate::db::{block_on, open_runtime_db};
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::projections::EventBus;
    use ship_types::event_types::canonical_session as op_event_types;
    use crate::AnalyticsProjection;

    fn setup() -> (std::path::PathBuf, sqlx::SqliteConnection) {
        let ship_dir = ship_substrate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        ship_substrate::testutil::init_project_minimal(base).unwrap();
        let conn = open_runtime_db(&ship_dir).unwrap();
        (ship_dir, conn)
    }

    fn bus() -> EventBus {
        let mut bus = EventBus::new();
        bus.register(Box::new(AnalyticsProjection::new()));
        bus
    }

    // ── op.* event helpers ───────────────────────────────────────────

    fn op_tool_use_started_event(actor_id: &str, tool: &str, started_at: &str) -> EventEnvelope {
        let payload = OpToolUseStarted {
            session_id: actor_id.to_string(),
            harness_session_id: None,
            tool: tool.to_string(),
            input: Some(serde_json::json!({"command": "echo test"})),
            blast: None,
            started_at: started_at.to_string(),
        };
        EventEnvelope::new(op_event_types::OP_TOOL_USE_STARTED, actor_id, &payload)
            .unwrap()
            .with_actor_id(actor_id)
            .with_target(actor_id)
    }

    fn op_tool_use_completed_event(
        actor_id: &str,
        tool: &str,
        status: &str,
        ended_at: &str,
    ) -> EventEnvelope {
        let payload = OpToolUseCompleted {
            session_id: actor_id.to_string(),
            harness_session_id: None,
            tool: tool.to_string(),
            status: status.to_string(),
            error: None,
            response: None,
            ended_at: ended_at.to_string(),
        };
        EventEnvelope::new(op_event_types::OP_TOOL_USE_COMPLETED, actor_id, &payload)
            .unwrap()
            .with_actor_id(actor_id)
            .with_target(actor_id)
    }

    fn turn_ended_event(
        actor_id: &str,
        model: Option<&str>,
        input: u64,
        output: u64,
        cache_w: u64,
        cache_r: u64,
    ) -> EventEnvelope {
        let payload = TurnEnded {
            session_id: actor_id.to_string(),
            idx: 0,
            stop_reason: Some("end_turn".to_string()),
            harness_session_id: None,
            model: model.map(String::from),
            input_tokens: Some(input),
            output_tokens: Some(output),
            reasoning_tokens: None,
            cache_creation_tokens: if cache_w > 0 { Some(cache_w) } else { None },
            cache_read_tokens: if cache_r > 0 { Some(cache_r) } else { None },
            harness_reported_cost_usd: None,
            observed_at: Some("2026-05-19T10:00:00+00:00".to_string()),
        };
        EventEnvelope::new(op_event_types::TURN_ENDED, actor_id, &payload)
            .unwrap()
            .with_actor_id(actor_id)
    }

    // ── actor.* event helpers ────────────────────────────────────────

    fn actor_created_event(
        actor_id: &str,
        harness: Option<&str>,
        model: Option<&str>,
    ) -> EventEnvelope {
        let payload = ship_types::payload::ActorCreated {
            kind: "worker".to_string(),
            environment_type: "cli".to_string(),
            harness: harness.map(String::from),
            model: model.map(String::from),
            ..Default::default()
        };
        EventEnvelope::new(ship_types::event_types::ACTOR_CREATED, actor_id, &payload)
            .unwrap()
            .with_actor_id(actor_id)
    }

    fn actor_stopped_event(actor_id: &str, reason: &str) -> EventEnvelope {
        let payload = ship_types::payload::ActorStopped {
            reason: reason.to_string(),
            exit_code: Some(0),
            duration_ms: None,
        };
        EventEnvelope::new(ship_types::event_types::ACTOR_STOPPED, actor_id, &payload)
            .unwrap()
            .with_actor_id(actor_id)
    }

    // ── Row query helpers ────────────────────────────────────────────

    #[derive(Debug)]
    struct SessionRow {
        actor_id: String,
        harness: Option<String>,
        model: Option<String>,
        started_at: String,
        ended_at: Option<String>,
        duration_ms: Option<i64>,
        tool_use_count: i64,
        tool_error_count: i64,
        reason: Option<String>,
    }

    fn query_session(conn: &mut sqlx::SqliteConnection, key: &str) -> Option<SessionRow> {
        let row: Option<(String, Option<String>, Option<String>, String, Option<String>, Option<i64>, i64, i64, Option<String>)> =
            block_on(async {
                sqlx::query_as(
                    "SELECT actor_id, harness, model, started_at, ended_at, duration_ms, \
                     tool_use_count, tool_error_count, reason \
                     FROM analytics_sessions WHERE session_key = ?",
                )
                .bind(key)
                .fetch_optional(&mut *conn)
                .await
            })
            .ok()
            .flatten();
        row.map(|r| SessionRow {
            actor_id: r.0,
            harness: r.1,
            model: r.2,
            started_at: r.3,
            ended_at: r.4,
            duration_ms: r.5,
            tool_use_count: r.6,
            tool_error_count: r.7,
            reason: r.8,
        })
    }

    #[derive(Debug)]
    struct ToolRow {
        tool: String,
        status: Option<String>,
        started_at: String,
        ended_at: Option<String>,
        duration_ms: Option<i64>,
    }

    fn query_tool_uses(conn: &mut sqlx::SqliteConnection, key: &str) -> Vec<ToolRow> {
        let rows: Vec<(String, Option<String>, String, Option<String>, Option<i64>)> =
            block_on(async {
                sqlx::query_as(
                    "SELECT tool, status, started_at, ended_at, duration_ms \
                     FROM analytics_tool_uses \
                     WHERE session_key = ? ORDER BY started_at ASC",
                )
                .bind(key)
                .fetch_all(&mut *conn)
                .await
            })
            .unwrap();
        rows.into_iter()
            .map(|r| ToolRow {
                tool: r.0,
                status: r.1,
                started_at: r.2,
                ended_at: r.3,
                duration_ms: r.4,
            })
            .collect()
    }

    // ── Hook-POST path tests (op.* emitted by http_ingress) ─────────

    #[test]
    fn op_tool_use_started_inserts_open_row_and_bumps_session_count() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-tu-1", Some("claude"), None),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_started_event("actor-tu-1", "Bash", "2026-05-18T10:00:01+00:00"),
            &mut conn,
        ).unwrap();
        let s = query_session(&mut conn, "actor-tu-1").expect("session row");
        assert_eq!(s.tool_use_count, 1);
        let tu = query_tool_uses(&mut conn, "actor-tu-1");
        assert_eq!(tu.len(), 1);
        assert_eq!(tu[0].tool, "Bash");
        assert!(tu[0].ended_at.is_none());
        assert!(tu[0].duration_ms.is_none());
    }

    #[test]
    fn op_tool_use_completed_closes_matching_open_row_with_duration() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &op_tool_use_started_event("actor-tu-2", "Bash", "2026-05-18T10:00:01+00:00"),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_completed_event("actor-tu-2", "Bash", "ok", "2026-05-18T10:00:01.750+00:00"),
            &mut conn,
        ).unwrap();
        let tu = query_tool_uses(&mut conn, "actor-tu-2");
        assert_eq!(tu.len(), 1);
        assert_eq!(tu[0].status.as_deref(), Some("ok"));
        assert_eq!(tu[0].duration_ms, Some(750));
    }

    #[test]
    fn op_tool_use_error_increments_session_error_count() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-err", None, None),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_started_event("actor-err", "Write", "2026-05-18T10:00:01+00:00"),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_completed_event("actor-err", "Write", "error", "2026-05-18T10:00:01.100+00:00"),
            &mut conn,
        ).unwrap();
        let s = query_session(&mut conn, "actor-err").expect("session row");
        assert_eq!(s.tool_error_count, 1);
        assert_eq!(s.tool_use_count, 1);
    }

    #[test]
    fn op_pairing_uses_oldest_open_row_per_tool() {
        // Two concurrent Bash calls: complete closes the OLDER one
        // first. Mis-pairing is documented; this test pins the rule.
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &op_tool_use_started_event("actor-pair-2", "Bash", "2026-05-18T10:00:01+00:00"),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_started_event("actor-pair-2", "Bash", "2026-05-18T10:00:02+00:00"),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_completed_event("actor-pair-2", "Bash", "ok", "2026-05-18T10:00:05+00:00"),
            &mut conn,
        ).unwrap();
        let tu = query_tool_uses(&mut conn, "actor-pair-2");
        assert_eq!(tu.len(), 2);
        // First (older start) should be closed.
        assert!(tu[0].ended_at.is_some());
        assert_eq!(tu[0].duration_ms, Some(4000));
        // Second is still open.
        assert!(tu[1].ended_at.is_none());
    }

    // ── Synth-path tests (opencode/cursor — no hook config) ──────────

    #[test]
    fn synth_path_tool_use_and_turn_fold_correctly() {
        // Simulates opencode: actor.created (no hook session-start),
        // op.tool_use_started from the supervisor stream parser,
        // session.turn.ended with token usage.
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-synth-1", Some("opencode"), Some("claude-3-5-sonnet")),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_started_event("actor-synth-1", "Bash", "2026-05-18T10:00:01+00:00"),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &turn_ended_event("actor-synth-1", Some("claude-3-5-sonnet"), 250, 75, 10, 1000),
            &mut conn,
        ).unwrap();
        // session row from actor.created
        let s = query_session(&mut conn, "actor-synth-1").expect("session row from actor.created");
        assert_eq!(s.harness.as_deref(), Some("opencode"));
        assert_eq!(s.tool_use_count, 1);
        // turn inserted
        let summary = crate::queries::session_token_summary(&mut conn, "actor-synth-1")
            .unwrap()
            .expect("turn summary");
        assert_eq!(summary.turn_count, 1);
        assert_eq!(summary.input_tokens, 250);
        assert_eq!(summary.output_tokens, 75);
        assert_eq!(summary.cache_creation_tokens, 10);
        assert_eq!(summary.cache_read_tokens, 1000);
    }

    // ── Actor-lifecycle fallback ─────────────────────────────────────

    #[test]
    fn actor_created_populates_session_when_no_hook_fires() {
        // Simulates dispatched keepalive claude: no hook events arrive.
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-keepalive-1", Some("claude"), Some("opus")),
            &mut conn,
        ).unwrap();
        let r = query_session(&mut conn, "actor-keepalive-1")
            .expect("actor.created must seed a session row");
        assert_eq!(r.actor_id, "actor-keepalive-1");
        assert_eq!(r.harness.as_deref(), Some("claude"));
        assert_eq!(r.model.as_deref(), Some("opus"));
        assert!(r.ended_at.is_none());
    }

    #[test]
    fn actor_created_without_harness_is_ignored() {
        // Service actors (mesh, user-inbox, supervisor) have no harness.
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(&actor_created_event("service-mesh", None, None), &mut conn).unwrap();
        assert!(query_session(&mut conn, "service-mesh").is_none());
    }

    #[test]
    fn actor_stopped_closes_session_row() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-stop-1", Some("claude"), None),
            &mut conn,
        ).unwrap();
        bus.dispatch(&actor_stopped_event("actor-stop-1", "process-exited"), &mut conn).unwrap();
        let r = query_session(&mut conn, "actor-stop-1").expect("row");
        assert!(r.ended_at.is_some(), "actor.stopped must record ended_at");
        assert_eq!(r.reason.as_deref(), Some("process-exited"));
    }

    #[test]
    fn actor_created_harness_not_overwritten_by_subsequent_op_insert() {
        // When actor.created fires first (harness=claude), then op.tool_use_started
        // triggers a stub session insert — the COALESCE must keep claude's value.
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-both", Some("claude"), Some("sonnet")),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_started_event("actor-both", "Bash", "2026-05-18T10:00:01+00:00"),
            &mut conn,
        ).unwrap();
        let r = query_session(&mut conn, "actor-both").expect("row");
        assert_eq!(r.harness.as_deref(), Some("claude"), "actor.created harness preserved");
        assert_eq!(r.model.as_deref(), Some("sonnet"), "actor.created model preserved");
    }

    #[test]
    fn actor_stopped_not_applied_after_session_already_closed() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-end-first", Some("claude"), None),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &actor_stopped_event("actor-end-first", "normal-exit"),
            &mut conn,
        ).unwrap();
        // A second actor.stopped should not re-update (ended_at IS NULL guard).
        // Simulate by dispatching again — ended_at is already set, WHERE filters it.
        bus.dispatch(
            &actor_stopped_event("actor-end-first", "force-killed"),
            &mut conn,
        ).unwrap();
        let r = query_session(&mut conn, "actor-end-first").expect("row");
        assert_eq!(r.reason.as_deref(), Some("normal-exit"), "first stop reason preserved");
    }

    // ── Token-usage analytics_turns ─────────────────────────────────

    #[test]
    fn turn_ended_inserts_analytics_turns_row() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &turn_ended_event("actor-tu-tokens", Some("claude-opus"), 250, 75, 10, 1000),
            &mut conn,
        ).unwrap();
        let summary =
            crate::queries::session_token_summary(&mut conn, "actor-tu-tokens")
                .unwrap()
                .expect("summary row");
        assert_eq!(summary.turn_count, 1);
        assert_eq!(summary.input_tokens, 250);
        assert_eq!(summary.output_tokens, 75);
        assert_eq!(summary.cache_creation_tokens, 10);
        assert_eq!(summary.cache_read_tokens, 1000);
    }

    #[test]
    fn turn_ended_backfills_session_model_when_absent() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &turn_ended_event("actor-bf", Some("claude-sonnet"), 100, 50, 0, 0),
            &mut conn,
        ).unwrap();
        let row = query_session(&mut conn, "actor-bf").expect("session stub created");
        assert_eq!(row.model.as_deref(), Some("claude-sonnet"));
    }

    #[test]
    fn turn_ended_does_not_overwrite_existing_session_model() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-noov", Some("claude"), Some("opus")),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &turn_ended_event("actor-noov", Some("sonnet"), 100, 50, 0, 0),
            &mut conn,
        ).unwrap();
        let row = query_session(&mut conn, "actor-noov").expect("session row");
        assert_eq!(row.model.as_deref(), Some("opus"), "actor.created model preserved");
    }

    #[test]
    fn truncate_clears_both_tables() {
        let (_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(
            &actor_created_event("actor-trunc", Some("claude"), None),
            &mut conn,
        ).unwrap();
        bus.dispatch(
            &op_tool_use_started_event("actor-trunc", "Bash", "2026-05-18T10:00:01+00:00"),
            &mut conn,
        ).unwrap();
        bus.rebuild(&[], &mut conn).unwrap();
        assert!(query_session(&mut conn, "actor-trunc").is_none());
        assert!(query_tool_uses(&mut conn, "actor-trunc").is_empty());
    }
}
