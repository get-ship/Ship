#[cfg(test)]
mod tests {
    use crate::db::branch_context::get_branch_link;
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn create_two_workspaces_both_exist() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/alpha".to_string(),
                ..Default::default()
            },
        )?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/beta".to_string(),
                ..Default::default()
            },
        )?;

        let first_after = get_workspace(&ship_dir, "feature/alpha")?
            .ok_or_else(|| anyhow::anyhow!("feature/alpha workspace missing"))?;
        let second_after = get_workspace(&ship_dir, "feature/beta")?
            .ok_or_else(|| anyhow::anyhow!("feature/beta workspace missing"))?;
        assert_eq!(first_after.name, "feature/alpha");
        assert_eq!(second_after.name, "feature/beta");
        Ok(())
    }

    #[test]
    fn create_workspace_main_branch_stays_unlinked() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "main".into(),
                ..Default::default()
            },
        )?;
        assert_eq!(workspace.name, "main");
        assert!(get_branch_link("main")?.is_none());
        Ok(())
    }

    #[test]
    fn delete_workspace_removes_workspace_links_and_sessions() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/delete-me".to_string(),
                ..Default::default()
            },
        )?;

        let started = crate::events::types::SessionStarted {
            goal: None,
            workspace_id: workspace.id.clone(),
            workspace_branch: workspace.name.clone(),
            ..Default::default()
        };
        let _start_envelope = crate::db::session_events::insert_session_with_started_event(
            "session-delete-me",
            &workspace.id,
            &started,
        )?;
        let ended = crate::events::types::SessionEnded {
            summary: Some("done".to_string()),
            duration_secs: Some(0),
            gate_result: None,
            updated_workspace_ids: Vec::new(),
            compile_error: None,
            ..Default::default()
        };
        let _end_envelope = crate::db::session_events::update_session_with_ended_event(
            "session-delete-me",
            &workspace.id,
            &ended,
        )?;
        assert_eq!(list_workspace_sessions(&ship_dir, None, 10)?.len(), 1);

        delete_workspace(&ship_dir, "feature/delete-me")?;

        assert!(get_workspace(&ship_dir, "feature/delete-me")?.is_none());
        assert!(get_branch_link("feature/delete-me")?.is_none());
        assert!(list_workspace_sessions(&ship_dir, None, 10)?.is_empty());
        Ok(())
    }

    #[test]
    fn create_workspace_with_path_stores_and_retrieves() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let branch = "feature/path-test";
        let ws = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                path: Some("/worktrees/path-test".to_string()),
                ..Default::default()
            },
        )?;
        assert_eq!(ws.path.as_deref(), Some("/worktrees/path-test"));

        let stored = get_workspace(&ship_dir, branch)?
            .ok_or_else(|| anyhow::anyhow!("workspace missing after create"))?;
        assert_eq!(stored.path.as_deref(), Some("/worktrees/path-test"));
        Ok(())
    }

    #[test]
    fn workspace_compile_exports_to_path_root() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["claude".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        let worktree_root = tmp
            .path()
            .join(".worktrees")
            .join("feature-worktree-export");
        std::fs::create_dir_all(&worktree_root)?;
        let worktree_path = worktree_root.to_string_lossy().to_string();
        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/worktree-export".to_string(),
                path: Some(worktree_path),
                ..Default::default()
            },
        )?;

        crate::workspace::compile::compile_workspace_context(
            &ship_dir,
            &workspace,
            workspace.active_agent.as_deref(),
        )?;
        assert!(
            worktree_root.join(".mcp.json").exists(),
            "expected provider config to be written to worktree root"
        );
        assert!(
            !tmp.path().join(".mcp.json").exists(),
            "main checkout root should not receive worktree provider config"
        );
        Ok(())
    }

    #[test]
    fn create_workspace_rejects_unknown_active_agent() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let err = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/no-mode".to_string(),
                active_agent: Some("ghost".to_string()),
                ..Default::default()
            },
        )
        .expect_err("expected invalid agent to be rejected");

        assert!(err.to_string().contains("Agent 'ghost' not found"));
        Ok(())
    }

    #[test]
    fn set_workspace_tmux_session_write_and_read_back() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feat/tmux-test".to_string(),
                ..Default::default()
            },
        )?;

        let _updated = set_workspace_tmux_session(
            &ship_dir,
            "feat/tmux-test",
            Some("my-tmux-session"),
        )?;

        let _cleared = set_workspace_tmux_session(&ship_dir, "feat/tmux-test", None)?;

        Ok(())
    }

    #[test]
    fn set_workspace_tmux_session_errors_for_missing_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let err = set_workspace_tmux_session(&ship_dir, "nonexistent", Some("session"))
            .expect_err("expected error for nonexistent workspace");
        assert!(
            err.to_string().contains("Workspace not found"),
            "unexpected error: {err}"
        );
        Ok(())
    }
}
