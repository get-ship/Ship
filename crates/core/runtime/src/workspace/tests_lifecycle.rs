//! Lifecycle tests — transition matrix, tmux/started events, auto-creation,
//! and rebuild-from-events for tmux/worktree fields.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::workspace::helpers::validate_workspace_transition;
    use crate::workspace::*;
    use anyhow::Result;

    fn setup() -> std::path::PathBuf {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        ensure_db().unwrap();
        ship_dir
    }

    fn count_events(event_type: &str, entity_id: &str) -> i64 {
        let mut conn = open_db_at(&db_path().unwrap()).unwrap();
        block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE event_type = ? AND entity_id = ?",
            )
            .bind(event_type)
            .bind(entity_id)
            .fetch_one(&mut conn)
            .await
        })
        .unwrap()
    }

    // ── Transition matrix ──────────────────────────────────────────────────

    #[test]
    fn transition_active_to_active_is_valid() {
        assert!(validate_workspace_transition(
            WorkspaceStatus::Active,
            WorkspaceStatus::Active,
        )
        .is_ok());
    }

    #[test]
    fn transition_active_to_archived_is_valid() {
        assert!(validate_workspace_transition(
            WorkspaceStatus::Active,
            WorkspaceStatus::Archived,
        )
        .is_ok());
    }

    #[test]
    fn transition_archived_to_active_is_valid() {
        assert!(validate_workspace_transition(
            WorkspaceStatus::Archived,
            WorkspaceStatus::Active,
        )
        .is_ok());
    }

    #[test]
    fn transition_archived_to_archived_is_noop() {
        assert!(validate_workspace_transition(
            WorkspaceStatus::Archived,
            WorkspaceStatus::Archived,
        )
        .is_ok());
    }

    // ── Activate emits events ──────────────────────────────────────────────

    #[test]
    fn activate_emits_activated_event() -> Result<()> {
        let ship_dir = setup();
        let branch = "feature/lc-activate-event";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        activate_workspace(&ship_dir, branch)?;

        let count = count_events("workspace.activated", branch);
        assert!(count >= 1, "activate must emit workspace.activated");
        Ok(())
    }

    #[test]
    fn activate_auto_creates_emits_created_event() -> Result<()> {
        let ship_dir = setup();
        let branch = "feature/lc-auto-create";

        // No workspace exists — activate should auto-create.
        activate_workspace(&ship_dir, branch)?;

        let created = count_events("workspace.created", branch);
        let activated = count_events("workspace.activated", branch);
        assert_eq!(created, 1, "auto-creation must emit workspace.created");
        assert_eq!(activated, 1, "must also emit workspace.activated");
        Ok(())
    }

    // ── Re-activate (archived → active) ────────────────────────────────────

    #[test]
    fn reactivate_archived_workspace() -> Result<()> {
        let ship_dir = setup();
        let branch = "feature/lc-reactivate";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Archived)?;

        let ws = activate_workspace(&ship_dir, branch)?;
        assert_eq!(ws.status, WorkspaceStatus::Active);
        assert!(ws.last_activated_at.is_some());
        Ok(())
    }

    // ── Tmux assign emits event ────────────────────────────────────────────

    #[test]
    fn tmux_assign_emits_event() -> Result<()> {
        let ship_dir = setup();
        let branch = "feature/lc-tmux";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        set_workspace_tmux_session(&ship_dir, branch, Some("ship-tmux"))?;

        let count = count_events("workspace.tmux_assigned", branch);
        assert_eq!(count, 1, "tmux assign must emit workspace.tmux_assigned");
        Ok(())
    }

    #[test]
    fn tmux_clear_emits_event() -> Result<()> {
        let ship_dir = setup();
        let branch = "feature/lc-tmux-clear";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        set_workspace_tmux_session(&ship_dir, branch, Some("ship-t"))?;
        set_workspace_tmux_session(&ship_dir, branch, None)?;

        let count = count_events("workspace.tmux_assigned", branch);
        assert_eq!(count, 2, "assign + clear = 2 tmux_assigned events");
        Ok(())
    }

    // ── Workspace started emits event ──────────────────────────────────────

    #[test]
    fn workspace_started_emits_event() -> Result<()> {
        let ship_dir = setup();
        let branch = "feature/lc-started";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        let worktree = std::path::Path::new("/tmp/test-worktree");
        set_workspace_started(&ship_dir, branch, worktree, "ship-started")?;

        let count = count_events("workspace.started", branch);
        assert_eq!(count, 1, "must emit workspace.started event");
        Ok(())
    }

    // ── Started event sets worktree fields correctly ──────────────────────

    #[test]
    fn started_sets_worktree_fields_in_projection() -> Result<()> {
        let ship_dir = setup();
        let branch = "feature/lc-started-proj";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        let worktree = std::path::Path::new("/tmp/started-proj-wt");
        set_workspace_started(&ship_dir, branch, worktree, "ship-proj")?;

        let ws = get_workspace(&ship_dir, branch)?.unwrap();
        assert!(ws.is_worktree, "started must set is_worktree=true");
        assert_eq!(
            ws.worktree_path.as_deref(),
            Some("/tmp/started-proj-wt"),
            "started must set worktree_path"
        );
        Ok(())
    }
}
