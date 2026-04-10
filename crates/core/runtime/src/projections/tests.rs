//! Projection system tests — prove that events are the source of truth.

#[cfg(test)]
mod workspace_tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::events::types::event_types;
    use crate::events::EventEnvelope;
    use crate::projections::{EventBus, WorkspaceProjection};

    fn setup() -> sqlx::SqliteConnection {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        ensure_db().unwrap();
        open_db_at(&db_path().unwrap()).unwrap()
    }

    fn bus() -> EventBus {
        let mut bus = EventBus::new();
        bus.register(Box::new(WorkspaceProjection::new()));
        bus
    }

    fn workspace_created_event(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_CREATED,
            branch,
            &serde_json::json!({
                "workspace_id": branch,
                "workspace_type": "feature",
                "active_agent": null,
                "providers": [],
                "mcp_servers": [],
                "skills": [],
                "path": null
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    fn workspace_compiled_event(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_COMPILED,
            branch,
            &serde_json::json!({
                "config_generation": 1,
                "duration_ms": 42
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    fn workspace_deleted_event(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_DELETED,
            branch,
            &serde_json::json!({"branch": branch}),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    fn query_workspace_exists(conn: &mut sqlx::SqliteConnection, branch: &str) -> bool {
        let count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_one(&mut *conn)
            .await
        })
        .unwrap();
        count > 0
    }

    // ── Tests ──────────────────────────────────────────────────────────

    #[test]
    fn dispatch_created_inserts_workspace_row() {
        let mut conn = setup();
        let bus = bus();
        let event = workspace_created_event("feature/proj-test-1");

        bus.dispatch(&event, &mut conn);

        assert!(query_workspace_exists(&mut conn, "feature/proj-test-1"), "created event must insert row");
    }

    #[test]
    fn dispatch_deleted_removes_row() {
        let mut conn = setup();
        let bus = bus();

        bus.dispatch(&workspace_created_event("feature/proj-test-3"), &mut conn);
        assert!(query_workspace_exists(&mut conn, "feature/proj-test-3"));

        bus.dispatch(&workspace_deleted_event("feature/proj-test-3"), &mut conn);
        assert!(!query_workspace_exists(&mut conn, "feature/proj-test-3"));
    }

    #[test]
    fn rebuild_from_events_produces_identical_state() {
        let mut conn = setup();
        let bus = bus();

        let events = vec![
            workspace_created_event("feature/rebuild-a"),
            workspace_compiled_event("feature/rebuild-a"),
            workspace_created_event("feature/rebuild-b"),
        ];

        for event in &events {
            bus.dispatch(event, &mut conn);
        }

        let exists_a = query_workspace_exists(&mut conn, "feature/rebuild-a");
        let exists_b = query_workspace_exists(&mut conn, "feature/rebuild-b");

        let report = bus.rebuild(&events, &mut conn).unwrap();
        assert_eq!(report.events_replayed, 3);

        let rebuilt_a = query_workspace_exists(&mut conn, "feature/rebuild-a");
        let rebuilt_b = query_workspace_exists(&mut conn, "feature/rebuild-b");
        assert_eq!(exists_a, rebuilt_a, "rebuild must produce identical state for workspace A");
        assert_eq!(exists_b, rebuilt_b, "rebuild must produce identical state for workspace B");
    }

    #[test]
    fn rebuild_report_lists_projections() {
        let mut conn = setup();
        let bus = bus();
        let report = bus.rebuild(&[], &mut conn).unwrap();
        assert!(
            report.projections_rebuilt.contains(&"workspace_state".to_string()),
            "report must list rebuilt projections"
        );
    }

    // ── Idempotency tests ─────────────────────────────────────────────

    #[test]
    fn compiled_projection_is_idempotent() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/idem-compiled";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        let event = workspace_compiled_event(branch);
        bus.dispatch(&event, &mut conn);
        bus.dispatch(&event, &mut conn); // apply same event twice

        let config_gen: Option<i64> = block_on(async {
            sqlx::query_scalar("SELECT config_generation FROM workspace WHERE branch = ?")
                .bind(branch)
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(config_gen, Some(1), "applying same compiled event twice must not double-increment");
    }

    #[test]
    fn tmux_assigned_projection_applies() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/tmux-test";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        let event = EventEnvelope::new(
            event_types::WORKSPACE_TMUX_ASSIGNED,
            branch,
            &serde_json::json!({ "tmux_session_name": "ship-tmux" }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate();
        bus.dispatch(&event, &mut conn);

        let name: Option<String> = block_on(async {
            sqlx::query_scalar("SELECT tmux_session_name FROM workspace WHERE branch = ?")
                .bind(branch)
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap()
        .flatten();
        assert_eq!(name.as_deref(), Some("ship-tmux"));
    }

    #[test]
    fn started_projection_sets_path_field() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/started-test";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        let event = EventEnvelope::new(
            event_types::WORKSPACE_STARTED,
            branch,
            &serde_json::json!({
                "worktree_path": "/tmp/worktrees/started-test",
                "tmux_session_name": "ship-started"
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate();
        bus.dispatch(&event, &mut conn);

        let row: Option<(Option<String>, Option<String>)> = block_on(async {
            sqlx::query_as(
                "SELECT path, tmux_session_name \
                 FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
        })
        .unwrap();
        let (path, tmux) = row.unwrap();
        assert_eq!(path.as_deref(), Some("/tmp/worktrees/started-test"));
        assert_eq!(tmux.as_deref(), Some("ship-started"));
    }

    #[test]
    fn event_versioning_v1_compiled_without_new_fields() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/versioning-test";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        let v1_event = EventEnvelope::new(
            event_types::WORKSPACE_COMPILED,
            branch,
            &serde_json::json!({
                "config_generation": 5,
                "duration_ms": 100
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate();

        bus.dispatch(&v1_event, &mut conn);

        let config_gen: Option<i64> = block_on(async {
            sqlx::query_scalar("SELECT config_generation FROM workspace WHERE branch = ?")
                .bind(branch)
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(config_gen, Some(5), "v1 event must set config_generation from payload");
    }

    #[test]
    fn rebuild_preserves_tmux_state() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/rebuild-tmux";

        let events = vec![
            workspace_created_event(branch),
            EventEnvelope::new(
                event_types::WORKSPACE_STARTED,
                branch,
                &serde_json::json!({
                    "worktree_path": "/tmp/wt/rebuild",
                    "tmux_session_name": "ship-rebuild"
                }),
            )
            .unwrap()
            .with_context(Some(branch), None)
            .elevate(),
            workspace_compiled_event(branch),
        ];

        for e in &events {
            bus.dispatch(e, &mut conn);
        }

        let before: Option<(Option<String>, Option<String>, i64)> = block_on(async {
            sqlx::query_as(
                "SELECT tmux_session_name, path, config_generation \
                 FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
        })
        .unwrap();

        bus.rebuild(&events, &mut conn).unwrap();

        let after: Option<(Option<String>, Option<String>, i64)> = block_on(async {
            sqlx::query_as(
                "SELECT tmux_session_name, path, config_generation \
                 FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
        })
        .unwrap();

        assert_eq!(before, after, "rebuild must reproduce identical state");
    }

    #[test]
    fn transactional_rollback_on_projection_failure() {
        use crate::db::{ensure_db, open_db};
        use crate::persistence::event_store::append_event_with_conn;
        use crate::projections::{Projection, WorkspaceProjection};

        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        ensure_db().unwrap();

        let bad_envelope = EventEnvelope::new(
            event_types::WORKSPACE_CREATED,
            "feature/txn-rollback-test",
            &serde_json::json!("not-a-valid-payload"),
        )
        .unwrap()
        .elevate();

        let event_id = bad_envelope.id.clone();

        let mut conn = open_db().unwrap();
        block_on(async { sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await }).unwrap();

        append_event_with_conn(&bad_envelope, &mut conn).unwrap();

        let proj = WorkspaceProjection::new();
        let apply_result = proj.apply(&bad_envelope, &mut conn);
        assert!(apply_result.is_err(), "projection must fail on bad payload");

        block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await }).unwrap();
        drop(conn);

        let mut check_conn = open_db().unwrap();
        let count: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
                .bind(&event_id)
                .fetch_one(&mut check_conn)
                .await
        })
        .unwrap();
        assert_eq!(count, 0, "event must be rolled back when projection fails");
    }
}
