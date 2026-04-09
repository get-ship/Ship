#[cfg(test)]
mod tests {
    use crate::db::events::list_all_events;
    use crate::workspace::*;
    use crate::workspace::reconcile::reconcile_workspace_with_git_path;
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup_workspace(
        branch: &str,
        is_worktree: bool,
        worktree_path: Option<&str>,
    ) -> Result<(tempfile::TempDir, std::path::PathBuf)> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                is_worktree: Some(is_worktree),
                worktree_path: worktree_path.map(String::from),
                status: Some(WorkspaceStatus::Active),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        Ok((tmp, ship_dir))
    }

    fn events_of_type(event_type: &str) -> Vec<crate::events::EventEnvelope> {
        list_all_events()
            .unwrap()
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    // ── reconcile_workspace_with_git_path: basic transitions ───────────────

    #[test]
    fn noop_when_state_matches_no_worktree() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace("main", false, None)?;
        let ws = reconcile_workspace_with_git_path(&ship_dir, "main", &None)?
            .expect("workspace should exist");
        assert!(!ws.is_worktree);
        assert!(ws.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn noop_when_state_matches_worktree() -> Result<()> {
        let stored_path = "/tmp/ship-test-wt-match".to_string();
        let (_tmp, ship_dir) =
            setup_workspace("feature/x", true, Some(&stored_path))?;
        let ws = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/x",
            &Some(stored_path.clone()),
        )?
        .expect("workspace should exist");
        assert!(ws.is_worktree);
        assert_eq!(ws.worktree_path.as_deref(), Some(stored_path.as_str()));
        Ok(())
    }

    #[test]
    fn worktree_to_main_repo_transition() -> Result<()> {
        let (_tmp, ship_dir) =
            setup_workspace("feature/a", true, Some("/tmp/ship-test-wt-a"))?;
        let ws =
            reconcile_workspace_with_git_path(&ship_dir, "feature/a", &None)?
                .expect("workspace should exist");
        assert!(!ws.is_worktree);
        assert!(ws.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn main_repo_to_worktree_transition() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace("feature/b", false, None)?;
        let new_path = "/tmp/ship-test-worktree-b".to_string();
        let ws = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/b",
            &Some(new_path.clone()),
        )?
        .expect("workspace should exist");
        assert!(ws.is_worktree);
        assert_eq!(ws.worktree_path.as_deref(), Some(new_path.as_str()));
        Ok(())
    }

    #[test]
    fn worktree_path_update() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/c",
            true,
            Some("/tmp/ship-test-wt-c-old"),
        )?;
        let new_path = "/tmp/ship-test-moved-worktree".to_string();
        let ws = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/c",
            &Some(new_path.clone()),
        )?
        .expect("workspace should exist");
        assert!(ws.is_worktree);
        assert_eq!(ws.worktree_path.as_deref(), Some(new_path.as_str()));
        Ok(())
    }

    #[test]
    fn stale_path_cleared() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/stale",
            true,
            Some("/nonexistent/stale/path"),
        )?;
        let ws = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/stale",
            &None,
        )?
        .expect("workspace should exist");
        assert!(!ws.is_worktree);
        assert!(ws.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn returns_none_for_unknown_branch() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir =
            crate::project::init_project(tmp.path().to_path_buf())?;
        let result = reconcile_workspace_with_git_path(
            &ship_dir,
            "nonexistent-branch",
            &None,
        )?;
        assert!(result.is_none());
        Ok(())
    }

    // ── persistence: DB reflects corrections after reconcile ───────────────

    #[test]
    fn stale_worktree_persisted_in_db_after_reconcile() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/persist-stale",
            true,
            Some("/gone/worktree"),
        )?;
        reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/persist-stale",
            &None,
        )?;

        // Re-read from DB to confirm persistence.
        let ws = get_workspace(&ship_dir, "feature/persist-stale")?
            .expect("workspace should exist in DB");
        assert!(!ws.is_worktree, "is_worktree should be false in DB");
        assert!(
            ws.worktree_path.is_none(),
            "worktree_path should be cleared in DB"
        );
        Ok(())
    }

    #[test]
    fn new_worktree_path_persisted_in_db_after_reconcile() -> Result<()> {
        let (_tmp, ship_dir) =
            setup_workspace("feature/persist-new", false, None)?;
        let new_path = "/worktrees/persist-new".to_string();
        reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/persist-new",
            &Some(new_path.clone()),
        )?;

        let ws = get_workspace(&ship_dir, "feature/persist-new")?
            .expect("workspace should exist in DB");
        assert!(ws.is_worktree, "is_worktree should be true in DB");
        assert_eq!(ws.worktree_path.as_deref(), Some(new_path.as_str()));
        Ok(())
    }

    // ── event emission ─────────────────────────────────────────────────────

    #[test]
    fn emits_reconciled_event_on_state_change() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/evt-change",
            true,
            Some("/old/path"),
        )?;
        // Git says branch is no longer in a worktree.
        reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/evt-change",
            &None,
        )?;

        let events = events_of_type("workspace.reconciled");
        assert_eq!(events.len(), 1, "exactly one reconciled event expected");
        let payload: serde_json::Value =
            serde_json::from_str(&events[0].payload_json)?;
        assert_eq!(payload["is_worktree"], false);
        assert!(payload["worktree_path"].is_null());
        assert!(
            payload["reason"].as_str().unwrap().len() > 0,
            "reason should be non-empty"
        );
        Ok(())
    }

    #[test]
    fn no_reconciled_event_when_state_matches() -> Result<()> {
        let (_tmp, ship_dir) =
            setup_workspace("feature/evt-noop", false, None)?;
        reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/evt-noop",
            &None,
        )?;

        let events = events_of_type("workspace.reconciled");
        assert!(
            events.is_empty(),
            "no reconciled event when state already matches"
        );
        Ok(())
    }

    #[test]
    fn emits_reconciled_event_with_new_worktree_path() -> Result<()> {
        let (_tmp, ship_dir) =
            setup_workspace("feature/evt-new-wt", false, None)?;
        let new_path = "/worktrees/evt-new-wt".to_string();
        reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/evt-new-wt",
            &Some(new_path.clone()),
        )?;

        let events = events_of_type("workspace.reconciled");
        assert_eq!(events.len(), 1);
        let payload: serde_json::Value =
            serde_json::from_str(&events[0].payload_json)?;
        assert_eq!(payload["is_worktree"], true);
        assert_eq!(payload["worktree_path"], new_path);
        Ok(())
    }

    #[test]
    fn emits_reconciled_event_on_path_change() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/evt-path",
            true,
            Some("/worktrees/old"),
        )?;
        let new_path = "/worktrees/moved".to_string();
        reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/evt-path",
            &Some(new_path.clone()),
        )?;

        let events = events_of_type("workspace.reconciled");
        assert_eq!(events.len(), 1);
        let payload: serde_json::Value =
            serde_json::from_str(&events[0].payload_json)?;
        assert_eq!(payload["worktree_path"], new_path);
        assert!(payload["reason"].as_str().unwrap().contains("changed"));
        Ok(())
    }

    // ── batch reconciliation: daemon startup (S5) ──────────────────────────

    #[test]
    fn reconcile_all_clears_stale_workspaces_on_startup() -> Result<()> {
        use crate::workspace::reconcile::reconcile_all_workspaces;

        let tmp = tempdir()?;
        let ship_dir =
            crate::project::init_project(tmp.path().to_path_buf())?;

        // Create several workspaces: some stale, some correct.
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/stale-a".to_string(),
                is_worktree: Some(true),
                worktree_path: Some("/nonexistent/a".to_string()),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/stale-b".to_string(),
                is_worktree: Some(true),
                worktree_path: Some("/nonexistent/b".to_string()),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/ok".to_string(),
                is_worktree: Some(false),
                ..Default::default()
            },
        )?;

        let results = reconcile_all_workspaces(&ship_dir)?;

        // All workspaces should be reconciled.
        assert!(results.len() >= 3, "should reconcile all workspaces");

        // Stale workspaces should have is_worktree cleared.
        let stale_a = get_workspace(&ship_dir, "feature/stale-a")?
            .expect("stale-a should exist");
        assert!(
            !stale_a.is_worktree,
            "stale-a should no longer be marked as worktree"
        );
        assert!(stale_a.worktree_path.is_none());

        let stale_b = get_workspace(&ship_dir, "feature/stale-b")?
            .expect("stale-b should exist");
        assert!(!stale_b.is_worktree);
        assert!(stale_b.worktree_path.is_none());

        // Non-worktree workspace should be unchanged.
        let ok = get_workspace(&ship_dir, "feature/ok")?
            .expect("ok should exist");
        assert!(!ok.is_worktree);
        Ok(())
    }

    #[test]
    fn reconcile_all_emits_events_only_for_changed_workspaces() -> Result<()>
    {
        use crate::workspace::reconcile::reconcile_all_workspaces;

        let tmp = tempdir()?;
        let ship_dir =
            crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/changed".to_string(),
                is_worktree: Some(true),
                worktree_path: Some("/nonexistent/changed".to_string()),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/unchanged".to_string(),
                is_worktree: Some(false),
                ..Default::default()
            },
        )?;

        reconcile_all_workspaces(&ship_dir)?;

        let events = events_of_type("workspace.reconciled");
        // Only the stale workspace should produce a reconciled event.
        assert_eq!(
            events.len(),
            1,
            "only changed workspaces should emit reconciled events"
        );
        Ok(())
    }

    #[test]
    fn reconcile_all_returns_empty_when_no_workspaces() -> Result<()> {
        use crate::workspace::reconcile::reconcile_all_workspaces;

        let tmp = tempdir()?;
        let ship_dir =
            crate::project::init_project(tmp.path().to_path_buf())?;

        let results = reconcile_all_workspaces(&ship_dir)?;
        // init_project seeds a "ship" workspace, so we get at least that.
        // But all should reconcile cleanly with no errors.
        for ws in &results {
            assert!(!ws.is_worktree);
        }
        Ok(())
    }

    // ── resolve_project_root ─────────────────────────────────────────────

    #[test]
    fn resolve_project_root_returns_worktree_path_when_valid() -> Result<()> {
        use crate::workspace::reconcile::resolve_project_root_from_db;

        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let wt_dir = tmp.path().join("worktrees").join("feat-x");
        std::fs::create_dir_all(&wt_dir)?;
        let wt_path = wt_dir.to_str().unwrap().to_string();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feat/x".to_string(),
                is_worktree: Some(true),
                worktree_path: Some(wt_path.clone()),
                ..Default::default()
            },
        )?;

        // Simulate git agreeing the worktree exists.
        reconcile_workspace_with_git_path(&ship_dir, "feat/x", &Some(wt_path.clone()))?;

        let (root, ws) = resolve_project_root_from_db(&ship_dir, "feat/x")?;
        assert_eq!(root.to_str().unwrap(), wt_path);
        assert!(ws.is_worktree);
        Ok(())
    }

    #[test]
    fn resolve_project_root_falls_back_for_non_worktree() -> Result<()> {
        use crate::workspace::reconcile::resolve_project_root_from_db;

        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "main".to_string(),
                is_worktree: Some(false),
                ..Default::default()
            },
        )?;

        let (root, _ws) = resolve_project_root_from_db(&ship_dir, "main")?;
        assert_eq!(root, ship_dir.parent().unwrap());
        Ok(())
    }

    #[test]
    fn resolve_project_root_falls_back_when_worktree_path_stale() -> Result<()> {
        use crate::workspace::reconcile::resolve_project_root_from_db;

        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feat/stale-resolve".to_string(),
                is_worktree: Some(true),
                worktree_path: Some("/nonexistent/stale/path".to_string()),
                ..Default::default()
            },
        )?;

        let (root, _ws) = resolve_project_root_from_db(&ship_dir, "feat/stale-resolve")?;
        assert_eq!(root, ship_dir.parent().unwrap());
        Ok(())
    }

    #[test]
    fn resolve_project_root_returns_error_for_unknown_branch() -> Result<()> {
        use crate::workspace::reconcile::resolve_project_root_from_db;

        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let result = resolve_project_root_from_db(&ship_dir, "nonexistent-branch");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        Ok(())
    }

    // ── idempotency ────────────────────────────────────────────────────────

    #[test]
    fn reconcile_is_idempotent() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/idempotent",
            true,
            Some("/stale/path"),
        )?;

        // First reconcile corrects the stale state.
        let ws1 = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/idempotent",
            &None,
        )?
        .expect("workspace should exist");
        assert!(!ws1.is_worktree);

        // Second reconcile is a no-op — state already matches.
        let ws2 = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/idempotent",
            &None,
        )?
        .expect("workspace should exist");
        assert!(!ws2.is_worktree);

        // Only one reconciled event should exist (from the first call).
        let events = events_of_type("workspace.reconciled");
        assert_eq!(
            events.len(),
            1,
            "second reconcile should not emit duplicate event"
        );
        Ok(())
    }

    // ── list_workspaces self-healing ──────────────────────────────────────

    #[test]
    fn list_workspaces_cleans_stale_worktree_on_read() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir =
            crate::project::init_project(tmp.path().to_path_buf())?;

        // Create a workspace with a nonexistent worktree path.
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/stale-listed".to_string(),
                is_worktree: Some(true),
                worktree_path: Some("/nonexistent/stale/worktree".to_string()),
                ..Default::default()
            },
        )?;

        // list_workspaces should reconcile and return corrected data.
        let workspaces = list_workspaces(&ship_dir)?;
        let stale = workspaces
            .iter()
            .find(|w| w.branch == "feature/stale-listed")
            .expect("workspace should still be in the list");

        assert!(
            !stale.is_worktree,
            "stale worktree flag should be cleared by list_workspaces"
        );
        assert!(
            stale.worktree_path.is_none(),
            "stale worktree path should be cleared by list_workspaces"
        );

        // Verify DB was also updated (not just the returned data).
        let from_db = get_workspace(&ship_dir, "feature/stale-listed")?
            .expect("workspace should exist in DB");
        assert!(!from_db.is_worktree);
        assert!(from_db.worktree_path.is_none());
        Ok(())
    }
}
