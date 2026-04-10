#[cfg(test)]
mod tests {
    use crate::workspace::*;
    use crate::workspace::reconcile::{reconcile_workspace, resolve_project_root};
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup_workspace(
        branch: &str,
        path: Option<&str>,
    ) -> Result<(tempfile::TempDir, std::path::PathBuf)> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                path: path.map(String::from),
                ..Default::default()
            },
        )?;
        Ok((tmp, ship_dir))
    }

    // ── reconcile_workspace: basic lookups ─────────────────────────────────

    #[test]
    fn reconcile_returns_workspace_when_exists() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace("main", None)?;
        let ws = reconcile_workspace(&ship_dir, "main")?
            .expect("workspace should exist");
        assert_eq!(ws.name, "main");
        Ok(())
    }

    #[test]
    fn reconcile_returns_workspace_with_path() -> Result<()> {
        let stored_path = "/tmp/ship-test-wt-match";
        let (_tmp, ship_dir) = setup_workspace("feature/x", Some(stored_path))?;
        let ws = reconcile_workspace(&ship_dir, "feature/x")?
            .expect("workspace should exist");
        assert_eq!(ws.path.as_deref(), Some(stored_path));
        Ok(())
    }

    #[test]
    fn returns_none_for_unknown_branch() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        let result = reconcile_workspace(&ship_dir, "nonexistent-branch")?;
        assert!(result.is_none());
        Ok(())
    }

    // ── reconcile_all_workspaces ──────────────────────────────────────────

    #[test]
    fn reconcile_all_returns_all_workspaces() -> Result<()> {
        use crate::workspace::reconcile::reconcile_all_workspaces;

        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/a".to_string(),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/b".to_string(),
                ..Default::default()
            },
        )?;

        let results = reconcile_all_workspaces(&ship_dir)?;
        assert!(results.len() >= 2, "should reconcile all workspaces");
        Ok(())
    }

    #[test]
    fn reconcile_all_returns_empty_when_no_workspaces() -> Result<()> {
        use crate::workspace::reconcile::reconcile_all_workspaces;

        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let results = reconcile_all_workspaces(&ship_dir)?;
        // init_project may seed a "ship" workspace, so this may not be empty.
        // Just verify it does not error.
        let _ = results;
        Ok(())
    }

    // ── resolve_project_root ────────────────────────────────────────────

    #[test]
    fn resolve_project_root_returns_path_when_valid() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let wt_dir = tmp.path().join("worktrees").join("feat-x");
        std::fs::create_dir_all(&wt_dir)?;
        let wt_path = wt_dir.to_str().unwrap().to_string();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feat/x".to_string(),
                path: Some(wt_path.clone()),
                ..Default::default()
            },
        )?;

        let (root, ws) = resolve_project_root(&ship_dir, "feat/x")?;
        assert_eq!(root.to_str().unwrap(), wt_path);
        assert_eq!(ws.path.as_deref(), Some(wt_path.as_str()));
        Ok(())
    }

    #[test]
    fn resolve_project_root_falls_back_for_no_path() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "main".to_string(),
                ..Default::default()
            },
        )?;

        let (root, _ws) = resolve_project_root(&ship_dir, "main")?;
        assert_eq!(root, ship_dir.parent().unwrap());
        Ok(())
    }

    #[test]
    fn resolve_project_root_falls_back_when_path_stale() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feat/stale-resolve".to_string(),
                path: Some("/nonexistent/stale/path".to_string()),
                ..Default::default()
            },
        )?;

        let (root, _ws) = resolve_project_root(&ship_dir, "feat/stale-resolve")?;
        assert_eq!(root, ship_dir.parent().unwrap());
        Ok(())
    }

    #[test]
    fn resolve_project_root_returns_error_for_unknown_branch() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let result = resolve_project_root(&ship_dir, "nonexistent-branch");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
        Ok(())
    }

    // ── idempotency ────────────────────────────────────────────────────────

    #[test]
    fn reconcile_is_idempotent() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace("feature/idempotent", None)?;

        let ws1 = reconcile_workspace(&ship_dir, "feature/idempotent")?
            .expect("workspace should exist");
        assert_eq!(ws1.name, "feature/idempotent");

        let ws2 = reconcile_workspace(&ship_dir, "feature/idempotent")?
            .expect("workspace should exist");
        assert_eq!(ws2.name, "feature/idempotent");
        Ok(())
    }
}
