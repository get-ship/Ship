#[cfg(test)]
mod tests {
    use crate::db::{block_on, ensure_db, open_db_at};
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

    // ── test 1: start session → session.started event with correct goal ───────

    #[test]
    fn start_session_emits_started_event_with_goal() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-started".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-started",
            Some("implement events".to_string()),
            None,
            None,
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_STARTED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected session.started event");
        let ev = &events[0];
        let payload: crate::events::types::SessionStarted =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(
            payload.goal.as_deref(),
            Some("implement events"),
            "goal must match"
        );
        Ok(())
    }

    // ── test 2: log progress → session.progress event, elevated=0 ─────────────

    #[test]
    fn log_progress_emits_progress_event_not_elevated() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-progress".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-progress",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(
            &ship_dir,
            "feature/se-progress",
            "halfway done",
            None,
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_PROGRESS.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected session.progress event");
        let ev = &events[0];
        let payload: crate::events::types::SessionProgress =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(payload.message, "halfway done", "message must match");
        assert!(!ev.elevated, "session.progress must not be elevated");
        Ok(())
    }

    // ── test 3: end session → session.ended event with summary + duration ──────

    #[test]
    fn end_session_emits_ended_event_with_summary() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-ended".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-ended",
            None,
            None,
            None,
        )?;

        end_workspace_session(
            &ship_dir,
            "feature/se-ended",
            EndWorkspaceSessionRequest {
                summary: Some("all done".to_string()),
                gate_result: Some("pass".to_string()),
                ..Default::default()
            },
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_ENDED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected session.ended event");
        let ev = &events[0];
        let payload: crate::events::types::SessionEnded =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(payload.summary.as_deref(), Some("all done"), "summary must match");
        assert!(payload.duration_secs.is_some(), "duration_secs must be set");
        assert_eq!(payload.gate_result.as_deref(), Some("pass"), "gate_result must match");
        Ok(())
    }

    // ── test 4: all events carry correct entity_id, actor_id, session_id, workspace_id, parent_actor_id ──

    #[test]
    fn session_events_have_correct_metadata() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/se-metadata";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, branch)?
            .ok_or_else(|| anyhow::anyhow!("workspace not found"))?;

        let session = start_workspace_session(&ship_dir, branch, Some("meta test".to_string()), None, None)?;

        record_workspace_session_progress(&ship_dir, branch, "checkpoint", None)?;

        end_workspace_session(
            &ship_dir,
            branch,
            EndWorkspaceSessionRequest {
                summary: Some("done".to_string()),
                ..Default::default()
            },
        )?;

        let store = event_store();
        let all = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            ..Default::default()
        })?;

        let typed: Vec<_> = all
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type.as_str(),
                    event_types::SESSION_STARTED
                        | event_types::SESSION_PROGRESS
                        | event_types::SESSION_ENDED
                )
            })
            .collect();

        assert_eq!(typed.len(), 3, "expected exactly 3 session lifecycle events");

        for ev in &typed {
            assert_eq!(ev.entity_id, session.id, "entity_id must equal session id");
            assert_eq!(
                ev.actor_id.as_deref(),
                Some(session.id.as_str()),
                "actor_id must equal session id, got {:?}",
                ev.actor_id
            );
            assert_eq!(
                ev.session_id.as_deref(),
                Some(session.id.as_str()),
                "session_id must equal session id, got {:?}",
                ev.session_id
            );
            assert_eq!(
                ev.workspace_id.as_deref(),
                Some(workspace.id.as_str()),
                "workspace_id must equal workspace id, got {:?}",
                ev.workspace_id
            );
            assert_eq!(
                ev.parent_actor_id.as_deref(),
                Some(workspace.id.as_str()),
                "parent_actor_id must equal workspace id, got {:?}",
                ev.parent_actor_id
            );
        }
        Ok(())
    }

    // ── test 5: started/ended elevated=1, progress elevated=0 ─────────────────

    #[test]
    fn session_started_and_ended_are_elevated_progress_is_not() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-elevated".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-elevated",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(&ship_dir, "feature/se-elevated", "midpoint", None)?;

        end_workspace_session(
            &ship_dir,
            "feature/se-elevated",
            EndWorkspaceSessionRequest::default(),
        )?;

        let store = event_store();
        let all = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            ..Default::default()
        })?;

        let get = |et: &str| {
            all.iter()
                .find(|e| e.event_type == et)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("event '{}' not found", et))
        };

        let started = get(event_types::SESSION_STARTED)?;
        let progress = get(event_types::SESSION_PROGRESS)?;
        let ended = get(event_types::SESSION_ENDED)?;

        assert!(started.elevated, "session.started must be elevated");
        assert!(!progress.elevated, "session.progress must NOT be elevated");
        assert!(ended.elevated, "session.ended must be elevated");

        Ok(())
    }

    // ── test 6: attach emits session.attached event ────────────────────────────

    #[test]
    fn attach_emits_session_attached_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-attach".to_string(),
                ..Default::default()
            },
        )?;

        let first = start_workspace_session(
            &ship_dir,
            "feature/se-attach",
            Some("original".to_string()),
            None,
            None,
        )?;

        let attached = start_workspace_session(
            &ship_dir,
            "feature/se-attach",
            Some("second call".to_string()),
            None,
            None,
        )?;
        assert_eq!(attached.id, first.id, "attach must return same session");

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(first.id.clone()),
            event_type: Some(event_types::SESSION_ATTACHED.to_string()),
            ..Default::default()
        })?;

        assert_eq!(events.len(), 1, "expected exactly one session.attached event");
        let ev = &events[0];
        assert!(ev.elevated, "session.attached must be elevated");

        let payload: crate::events::types::SessionAttached =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(payload.existing_session_id, first.id);
        Ok(())
    }

    // ── test 8: blocker escalation sets elevated=true and payload.escalation ────

    #[test]
    fn escalated_progress_blocker_sets_elevated_and_payload() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-escalate-blocker".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-escalate-blocker",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(
            &ship_dir,
            "feature/se-escalate-blocker",
            "blocked on missing API key",
            Some("blocker"),
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_PROGRESS.to_string()),
            ..Default::default()
        })?;

        assert_eq!(events.len(), 1, "expected one progress event");
        let ev = &events[0];
        assert!(ev.elevated, "escalated progress must be elevated");

        let payload: crate::events::types::SessionProgress =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(payload.message, "blocked on missing API key");
        assert_eq!(
            payload.escalation.as_deref(),
            Some("blocker"),
            "payload must carry escalation level"
        );
        Ok(())
    }

    // ── test 9: warning escalation sets elevated=true ─────────────────────────

    #[test]
    fn escalated_progress_warning_is_elevated() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-escalate-warning".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-escalate-warning",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(
            &ship_dir,
            "feature/se-escalate-warning",
            "dependency version mismatch",
            Some("warning"),
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_PROGRESS.to_string()),
            ..Default::default()
        })?;

        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert!(ev.elevated, "warning-escalated progress must be elevated");

        let payload: crate::events::types::SessionProgress =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(
            payload.escalation.as_deref(),
            Some("warning"),
        );
        Ok(())
    }

    // ── test 10: question escalation sets elevated=true ───────────────────────

    #[test]
    fn escalated_progress_question_is_elevated() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-escalate-question".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-escalate-question",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(
            &ship_dir,
            "feature/se-escalate-question",
            "should we use gRPC or REST?",
            Some("question"),
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_PROGRESS.to_string()),
            ..Default::default()
        })?;

        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert!(ev.elevated, "question-escalated progress must be elevated");

        let payload: crate::events::types::SessionProgress =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(
            payload.escalation.as_deref(),
            Some("question"),
        );
        Ok(())
    }

    // ── test 11: non-escalated progress remains not elevated ──────────────────

    #[test]
    fn non_escalated_progress_stays_not_elevated() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-no-escalation".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-no-escalation",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(
            &ship_dir,
            "feature/se-no-escalation",
            "routine update",
            None,
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_PROGRESS.to_string()),
            ..Default::default()
        })?;

        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert!(!ev.elevated, "non-escalated progress must NOT be elevated");

        let payload: crate::events::types::SessionProgress =
            serde_json::from_str(&ev.payload_json)?;
        assert!(
            payload.escalation.is_none(),
            "non-escalated progress must have no escalation field"
        );
        Ok(())
    }

    // ── test 12: escalation field absent in payload when None ──────────────────

    #[test]
    fn session_progress_payload_omits_escalation_when_none() -> Result<()> {
        let payload = crate::events::types::SessionProgress {
            message: "just a note".to_string(),
            escalation: None,
        };
        let json = serde_json::to_string(&payload)?;
        let parsed: serde_json::Value = serde_json::from_str(&json)?;
        assert!(
            parsed.get("escalation").is_none()
                || parsed["escalation"].is_null(),
            "escalation must be absent or null when None"
        );
        Ok(())
    }

    // ── test 7: atomicity — if event insert fails, session row rolls back ──────

    #[test]
    fn session_event_insert_failure_rolls_back_session_write() -> Result<()> {
        use crate::db::session_events_testutil::insert_session_started_event;
        use crate::events::types::SessionStarted;

        let (_tmp, _ship_dir) = setup();
        let db = crate::db::db_path()?;

        let mut conn = open_db_at(&db)?;
        block_on(async {
            sqlx::query(
                "CREATE TRIGGER IF NOT EXISTS session_events_test_block \
                 BEFORE INSERT ON events \
                 WHEN NEW.actor_id = 'sess-atomicity-sentinel' \
                 BEGIN \
                   SELECT RAISE(FAIL, 'atomicity test: forced session event insert failure'); \
                 END",
            )
            .execute(&mut conn)
            .await
        })?;
        drop(conn);

        let sentinel_session_id = "sess-atomicity-sentinel";
        let sentinel_workspace_id = "ws-atomicity-sentinel";
        let payload = SessionStarted { goal: None, ..Default::default() };

        let result = insert_session_started_event(
            sentinel_session_id,
            sentinel_workspace_id,
            &payload,
        );

        assert!(
            result.is_err(),
            "transaction must fail when event INSERT is blocked by trigger"
        );

        // The session row must NOT have been committed — verify via direct query.
        let mut conn2 = open_db_at(&db)?;
        let count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE actor_id = ?",
            )
            .bind(sentinel_session_id)
            .fetch_one(&mut conn2)
            .await
        })?;
        assert_eq!(
            count, 0,
            "no events must be present when transaction rolls back; got {count}"
        );

        Ok(())
    }
}
