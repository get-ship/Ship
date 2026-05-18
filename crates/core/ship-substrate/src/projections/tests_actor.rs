//! Actor projection tests — prove actor state is derived from events.

#[cfg(test)]
mod actor_tests {
    use crate::db::{block_on, open_db};
    use crate::events::types::event_types;
    use crate::events::EventEnvelope;
    use crate::projections::{ActorProjection, EventBus};

    fn setup() -> (std::path::PathBuf, sqlx::SqliteConnection) {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::testutil::init_project_minimal(base).unwrap();
        crate::db::ensure_db().unwrap();
        // actors table lives in platform.db.
        let conn = open_db().unwrap();
        (ship_dir, conn)
    }

    fn bus() -> EventBus {
        let mut bus = EventBus::new();
        bus.register(Box::new(ActorProjection::new()));
        bus
    }

    fn actor_created_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_CREATED,
            actor_id,
            &serde_json::json!({
                "kind": "test-worker",
                "environment_type": "local",
                "workspace_id": ws_id,
                "parent_actor_id": null,
                "restart_count": 0
            }),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn actor_woke_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(event_types::ACTOR_WOKE, actor_id, &serde_json::json!({}))
            .unwrap()
            .with_context(Some(ws_id), None)
            .with_actor_id(actor_id)
            .elevate()
    }

    fn actor_slept_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_SLEPT,
            actor_id,
            &serde_json::json!({"idle_secs": 30}),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn actor_stopped_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_STOPPED,
            actor_id,
            &serde_json::json!({"reason": "done"}),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn actor_crashed_event(actor_id: &str, ws_id: &str, restart_count: u32) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_CRASHED,
            actor_id,
            &serde_json::json!({"error": "OOM", "restart_count": restart_count}),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn query_actor_status(conn: &mut sqlx::SqliteConnection, actor_id: &str) -> Option<String> {
        let rows: Vec<(String,)> = block_on(async {
            sqlx::query_as("SELECT status FROM actors WHERE id = ?")
                .bind(actor_id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().map(|r| r.0.clone())
    }

    fn query_actor_restart_count(conn: &mut sqlx::SqliteConnection, actor_id: &str) -> Option<i64> {
        let rows: Vec<(i64,)> = block_on(async {
            sqlx::query_as("SELECT restart_count FROM actors WHERE id = ?")
                .bind(actor_id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().map(|r| r.0)
    }

    fn query_actor_pid(conn: &mut sqlx::SqliteConnection, actor_id: &str) -> Option<i64> {
        let rows: Vec<(Option<i64>,)> = block_on(async {
            sqlx::query_as("SELECT pid FROM actors WHERE id = ?")
                .bind(actor_id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().and_then(|r| r.0)
    }

    fn process_spawned_event(slug: &str, pid: u32) -> EventEnvelope {
        EventEnvelope::new(
            crate::events::process::event_types::PROCESS_SPAWNED,
            slug,
            &serde_json::json!({
                "slug": slug,
                "job_id": format!("job-{slug}"),
                "pid": pid,
                "cmd": "test-cmd",
                "argv": [],
                "env_keys": [],
                "cwd": "/tmp",
                "executor_kind": "pty",
                "step_id": null,
            }),
        )
        .unwrap()
        .with_actor_id("service.process-lifecycle")
        .elevate()
    }

    // ── Tests ──────────────────────────────────────────────────────────

    #[test]
    fn dispatch_created_inserts_actor_row() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        let event = actor_created_event("actor-proj-1", "ws-proj-actor");

        bus.dispatch(&event, &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-1");
        assert_eq!(status.as_deref(), Some("created"), "created event must insert row");
    }

    #[test]
    fn dispatch_woke_updates_status() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-2", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_woke_event("actor-proj-2", "ws-proj-actor"), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-2");
        assert_eq!(status.as_deref(), Some("active"));
    }

    #[test]
    fn dispatch_stopped_updates_status() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-3", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_stopped_event("actor-proj-3", "ws-proj-actor"), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-3");
        assert_eq!(status.as_deref(), Some("stopped"));
    }

    #[test]
    fn dispatch_crashed_updates_status_and_restart_count() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-4", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_crashed_event("actor-proj-4", "ws-proj-actor", 3), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-4");
        assert_eq!(status.as_deref(), Some("crashed"));
        let count = query_actor_restart_count(&mut conn, "actor-proj-4");
        assert_eq!(count, Some(3));
    }

    #[test]
    fn rebuild_from_actor_events_produces_identical_state() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        let ws = "ws-proj-actor";

        let events = vec![
            actor_created_event("actor-rebuild-a", ws),
            actor_woke_event("actor-rebuild-a", ws),
            actor_created_event("actor-rebuild-b", ws),
            actor_slept_event("actor-rebuild-b", ws),
            actor_crashed_event("actor-rebuild-a", ws, 2),
        ];

        for event in &events {
            bus.dispatch(event, &mut conn);
        }

        let status_a = query_actor_status(&mut conn, "actor-rebuild-a");
        let status_b = query_actor_status(&mut conn, "actor-rebuild-b");
        let count_a = query_actor_restart_count(&mut conn, "actor-rebuild-a");

        let report = bus.rebuild(&events, &mut conn).unwrap();
        assert_eq!(report.events_replayed, 5);

        let rebuilt_a = query_actor_status(&mut conn, "actor-rebuild-a");
        let rebuilt_b = query_actor_status(&mut conn, "actor-rebuild-b");
        let rebuilt_count = query_actor_restart_count(&mut conn, "actor-rebuild-a");
        assert_eq!(status_a, rebuilt_a, "rebuild must produce identical status for actor A");
        assert_eq!(status_b, rebuilt_b, "rebuild must produce identical status for actor B");
        assert_eq!(count_a, rebuilt_count, "rebuild must produce identical restart_count");
    }

    #[test]
    fn dispatch_slept_updates_status() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-5", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_slept_event("actor-proj-5", "ws-proj-actor"), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-5");
        assert_eq!(status.as_deref(), Some("sleeping"));
    }

    #[test]
    fn dispatch_created_persists_cwd_and_project_id() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        let event = EventEnvelope::new(
            event_types::ACTOR_CREATED,
            "actor-cwd-test",
            &serde_json::json!({
                "kind": "agent",
                "environment_type": "local",
                "cwd": "/tmp/agent-cwd",
                "project_id": "proj-abc123",
                "restart_count": 0
            }),
        )
        .unwrap()
        .with_actor_id("actor-cwd-test")
        .elevate();

        bus.dispatch(&event, &mut conn);

        let row: Option<(Option<String>, Option<String>)> = block_on(async {
            sqlx::query_as("SELECT cwd, project_id FROM actors WHERE id = ?")
                .bind("actor-cwd-test")
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        let (cwd, project_id) = row.expect("actor row missing");
        assert_eq!(cwd.as_deref(), Some("/tmp/agent-cwd"));
        assert_eq!(project_id.as_deref(), Some("proj-abc123"));
    }

    #[test]
    fn dispatch_created_without_cwd_or_project_leaves_columns_null() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        // Use the legacy event helper which omits cwd / project_id.
        let event = actor_created_event("actor-legacy-shape", "ws-legacy");
        bus.dispatch(&event, &mut conn);

        let row: Option<(Option<String>, Option<String>)> = block_on(async {
            sqlx::query_as("SELECT cwd, project_id FROM actors WHERE id = ?")
                .bind("actor-legacy-shape")
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        let (cwd, project_id) = row.expect("actor row missing");
        assert!(cwd.is_none(), "legacy events leave cwd null");
        assert!(project_id.is_none(), "legacy events leave project_id null");
    }

    // ── Cursor tracking ────────────────────────────────────────────────

    #[test]
    fn dispatch_writes_projection_cursor() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        let event = actor_created_event("actor-cursor-1", "ws-cursor");

        // Pre-state: no cursor.
        assert!(bus.cursor_for("actor_state").unwrap().is_none());

        bus.dispatch(&event, &mut conn);

        // Post-state: cursor matches the event's id.
        let cursor = bus.cursor_for("actor_state").unwrap();
        assert_eq!(cursor.as_deref(), Some(event.id.as_str()));
    }

    #[test]
    fn dispatch_advances_cursor_to_latest_event() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        let e1 = actor_created_event("actor-cursor-2", "ws-cursor");
        let e2 = actor_woke_event("actor-cursor-2", "ws-cursor");

        bus.dispatch(&e1, &mut conn);
        bus.dispatch(&e2, &mut conn);

        let cursor = bus.cursor_for("actor_state").unwrap();
        assert_eq!(
            cursor.as_deref(),
            Some(e2.id.as_str()),
            "cursor must point at the latest applied event"
        );
    }

    #[test]
    fn truncate_clears_projection_cursor() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        let event = actor_created_event("actor-cursor-3", "ws-cursor");

        bus.dispatch(&event, &mut conn);
        assert!(bus.cursor_for("actor_state").unwrap().is_some());

        bus.rebuild(&[], &mut conn).unwrap();
        assert!(
            bus.cursor_for("actor_state").unwrap().is_none(),
            "rebuild must clear the projection cursor"
        );
    }

    // ── process.spawned → pid wiring ─────────────────────────────────

    #[test]
    fn process_spawned_after_created_sets_pid() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(&actor_created_event("pid-ord-1", "ws-pid"), &mut conn);
        bus.dispatch(&process_spawned_event("pid-ord-1", 12345), &mut conn);
        assert_eq!(query_actor_pid(&mut conn, "pid-ord-1"), Some(12345));
    }

    #[test]
    fn process_spawned_before_created_upserts_stub_then_preserves_pid() {
        // Race: process.spawned can land in the projection BEFORE the
        // fire-and-forget actor.created thread completes (routing.rs
        // spawns it on a std::thread). The UPSERT in apply_process_spawned
        // must insert a stub row with the pid; the later actor.created
        // must NOT trample the pid (ON CONFLICT preserves it).
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(&process_spawned_event("pid-race-1", 54321), &mut conn);
        assert_eq!(
            query_actor_pid(&mut conn, "pid-race-1"),
            Some(54321),
            "process.spawned must INSERT a stub row carrying pid"
        );
        bus.dispatch(&actor_created_event("pid-race-1", "ws-pid"), &mut conn);
        assert_eq!(
            query_actor_pid(&mut conn, "pid-race-1"),
            Some(54321),
            "actor.created landing after process.spawned must preserve pid",
        );
        // And the actor_created metadata still applied — status moved
        // from default 'created' (insert) to 'created' (created event)
        // is a no-op, but the row should now have kind = 'test-worker'
        // (not the 'pipeline-step' the stub used).
        let kind: Option<String> = block_on(async {
            sqlx::query_as::<_, (String,)>("SELECT kind FROM actors WHERE id = ?")
                .bind("pid-race-1")
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap()
        .map(|r| r.0);
        assert_eq!(kind.as_deref(), Some("test-worker"));
    }

    #[test]
    fn process_spawned_with_no_pid_is_noop() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(&actor_created_event("pid-none-1", "ws-pid"), &mut conn);
        // pid: 0 (the kernel-swapper sentinel) → guard rejects.
        bus.dispatch(&process_spawned_event("pid-none-1", 0), &mut conn);
        assert_eq!(
            query_actor_pid(&mut conn, "pid-none-1"),
            None,
            "pid=0 must be treated as no pid recorded"
        );
    }

    #[test]
    fn duplicate_actor_created_preserves_pid_set_by_process_spawned() {
        // Regression: INSERT OR REPLACE used to wipe pid on every
        // duplicate actor.created (reconcile path can fire twice).
        // The ON CONFLICT DO UPDATE in apply_created must preserve pid.
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        bus.dispatch(&actor_created_event("pid-dup-1", "ws-pid"), &mut conn);
        bus.dispatch(&process_spawned_event("pid-dup-1", 99999), &mut conn);
        assert_eq!(query_actor_pid(&mut conn, "pid-dup-1"), Some(99999));
        // Second actor.created (race with reconcile, etc.) must NOT
        // clobber pid.
        bus.dispatch(&actor_created_event("pid-dup-1", "ws-pid"), &mut conn);
        assert_eq!(
            query_actor_pid(&mut conn, "pid-dup-1"),
            Some(99999),
            "duplicate actor.created must not trample pid set by process.spawned"
        );
    }
}
