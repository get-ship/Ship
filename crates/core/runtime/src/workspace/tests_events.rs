#[cfg(test)]
mod tests {
    use crate::db::{ensure_db, open_db_at};
    use crate::db::block_on;
    use crate::events::filter::EventFilter;
    use crate::persistence::event_store::{EventStore, SqliteEventStore};
    use crate::events::types::event_types;
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    fn event_store() -> SqliteEventStore {
        SqliteEventStore::new().unwrap()
    }

    // ── test 1: create emits workspace.created ───────────────────────────────

    #[test]
    fn create_workspace_emits_created_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/evt-create".to_string(),
                ..Default::default()
            },
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some("feature/evt-create".to_string()),
            event_type: Some(event_types::WORKSPACE_CREATED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected at least one workspace.created event");
        let ev = &events[0];
        assert_eq!(ev.event_type, event_types::WORKSPACE_CREATED);
        Ok(())
    }

    // ── test 2: compile emits workspace.compiled with correct payload ─────────

    #[test]
    fn set_agent_emits_compiled_event_with_payload() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/evt-compiled".to_string(),
                ..Default::default()
            },
        )?;
        let _ = set_workspace_active_agent(&ship_dir, "feature/evt-compiled", None)?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some("feature/evt-compiled".to_string()),
            event_type: Some(event_types::WORKSPACE_COMPILED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected workspace.compiled event after set_agent");
        let ev = &events[0];
        let payload: crate::events::types::WorkspaceCompiled =
            serde_json::from_str(&ev.payload_json)?;
        assert!(payload.duration_ms < 60_000, "duration_ms should be reasonable");
        Ok(())
    }

    // ── test 3: compile failure emits workspace.compile_failed ────────────────

    #[test]
    #[ignore = "provider resolution always falls back to claude; need a different compile failure trigger"]
    fn compile_failure_emits_compile_failed_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/evt-fail".to_string(),
                ..Default::default()
            },
        )?;
        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.active_agent = Some("nonexistent-agent-id".to_string());
        crate::config::save_config(&config, Some(ship_dir.clone()))?;
        let result = set_workspace_active_agent(&ship_dir, "feature/evt-fail", Some("nonexistent-agent-id"));
        assert!(result.is_err(), "expected compile failure");

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some("feature/evt-fail".to_string()),
            event_type: Some(event_types::WORKSPACE_COMPILE_FAILED.to_string()),
            ..Default::default()
        })?;

        assert!(
            !events.is_empty(),
            "expected workspace.compile_failed event after compile error"
        );
        let ev = &events[0];
        let payload: crate::events::types::WorkspaceCompileFailed =
            serde_json::from_str(&ev.payload_json)?;
        assert!(!payload.error.is_empty(), "error string must be non-empty");
        Ok(())
    }

    // ── test 4: workspace events have correct metadata ───────────────────────

    #[test]
    fn workspace_events_have_correct_metadata() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/evt-metadata";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        let store = event_store();
        let all_ws_events = store.query(&EventFilter {
            entity_id: Some(branch.to_string()),
            ..Default::default()
        })?;

        let typed: Vec<_> = all_ws_events
            .iter()
            .filter(|e| e.event_type == event_types::WORKSPACE_CREATED)
            .collect();

        assert!(
            !typed.is_empty(),
            "expected workspace.created event for branch '{}'",
            branch
        );

        for ev in &typed {
            assert_eq!(
                ev.entity_id, branch,
                "entity_id must equal workspace branch"
            );
            assert!(ev.elevated, "workspace lifecycle events must have elevated=true");
        }
        Ok(())
    }

    // ── test 5: atomicity — event insert failure prevents projection write ───

    #[test]
    fn event_insert_failure_prevents_workspace_write() -> Result<()> {
        use crate::db::workspace_events::emit_workspace_created;
        use crate::events::types::WorkspaceCreated;

        let (_tmp, _ship_dir) = setup();
        let db = crate::db::db_path()?;

        let mut conn = open_db_at(&db)?;
        block_on(async {
            sqlx::query(
                "CREATE TRIGGER IF NOT EXISTS events_test_block \
                 BEFORE INSERT ON events \
                 WHEN NEW.actor_id = 'atomicity-sentinel' \
                 BEGIN \
                   SELECT RAISE(FAIL, 'atomicity test: forced event insert failure'); \
                 END",
            )
            .execute(&mut conn)
            .await
        })?;
        drop(conn);

        let sentinel_branch = "atomicity-sentinel";
        let payload = WorkspaceCreated {
            workspace_id: sentinel_branch.to_string(),
            workspace_type: "feature".to_string(),
            active_agent: None,
            providers: vec![],
            mcp_servers: vec![],
            skills: vec![],
            path: None,
        };
        let result = emit_workspace_created(sentinel_branch, &payload);

        assert!(
            result.is_err(),
            "transaction must fail when event INSERT is blocked by trigger"
        );

        let mut conn2 = open_db_at(&db)?;
        let count: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM workspace WHERE branch = ?")
                .bind(sentinel_branch)
                .fetch_one(&mut conn2)
                .await
        })?;
        assert_eq!(
            count, 0,
            "workspace row must not exist when event INSERT fails; got {count} rows"
        );

        Ok(())
    }
}
