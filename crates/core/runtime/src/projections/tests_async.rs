//! Async projection integration tests.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use ship_types::envelope::EventEnvelope;
    use crate::persistence::event_store::{EventStore, SqliteEventStore};
    use crate::events::types::event_types;
    use crate::projections::{EventBus, WorkspaceProjection};

    fn setup() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        tmp
    }

    fn workspace_created(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_CREATED,
            branch,
            &serde_json::json!({
                "workspace_id": branch,
                "workspace_type": "feature",
                "providers": [],
                "mcp_servers": [],
                "skills": [],
                "path": null
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .with_actor_id(branch)
        .elevate()
    }

    // ── Test: projection rebuild from events table produces correct state ──────

    #[test]
    fn projection_rebuild_from_events_table_produces_correct_state() {
        let _tmp = setup();

        let store = SqliteEventStore::new().unwrap();
        let events = vec![
            workspace_created("feature/rebuild-src-a"),
            workspace_created("feature/rebuild-src-b"),
        ];
        for ev in &events {
            store.append(ev).unwrap();
        }

        let mut conn = open_db_at(&db_path().unwrap()).unwrap();
        let mut bus = EventBus::new();
        bus.register(Box::new(WorkspaceProjection::new()));
        let report = bus.rebuild(&events, &mut conn).unwrap();

        assert_eq!(report.events_replayed, 2);
        assert!(
            report.projections_rebuilt.contains(&"workspace_state".to_string()),
            "rebuild report must list workspace_state"
        );

        let exists_a = crate::worktrees::get("feature/rebuild-src-a")
            .unwrap()
            .is_some();
        let exists_b = crate::worktrees::get("feature/rebuild-src-b")
            .unwrap()
            .is_some();

        assert!(exists_a, "workspace-a must exist after rebuild");
        assert!(exists_b, "workspace-b must exist after rebuild");
    }
}
