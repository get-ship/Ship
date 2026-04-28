#[cfg(test)]
mod tests {
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn workspace_session_start_and_end_happy_path() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/session-flow".to_string(),
                ..Default::default()
            },
        )?;

        let started = start_workspace_session(
            &ship_dir,
            "feature/session-flow",
            Some("Implement parser".to_string()),
            None,
            None,
        )?;
        assert_eq!(started.status, WorkspaceSessionStatus::Active);
        assert_eq!(started.goal.as_deref(), Some("Implement parser"));
        assert_eq!(started.primary_provider.as_deref(), Some("claude"));
        assert!(started.ended_at.is_none());

        let active = get_active_workspace_session(&ship_dir, "feature/session-flow")?
            .ok_or_else(|| anyhow::anyhow!("active session not found"))?;
        assert_eq!(active.id, started.id);

        let ended = end_workspace_session(
            &ship_dir,
            "feature/session-flow",
            EndWorkspaceSessionRequest {
                summary: Some("Implemented parser + tests".to_string()),
                updated_workspace_ids: vec!["feat-parser".to_string()],
                ..Default::default()
            },
        )?;
        assert_eq!(ended.status, WorkspaceSessionStatus::Ended);
        assert!(ended.ended_at.is_some());
        assert_eq!(ended.summary.as_deref(), Some("Implemented parser + tests"));
        assert_eq!(ended.updated_workspace_ids, vec!["feat-parser".to_string()]);
        assert!(get_active_workspace_session(&ship_dir, "feature/session-flow")?.is_none());

        Ok(())
    }

    #[test]
    fn session_record_captures_metrics_on_end() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/metrics".to_string(),
                ..Default::default()
            },
        )?;

        let started = start_workspace_session(
            &ship_dir,
            "feature/metrics",
            Some("test metrics".to_string()),
            None,
            None,
        )?;

        let ended = end_workspace_session(
            &ship_dir,
            "feature/metrics",
            EndWorkspaceSessionRequest {
                summary: Some("done".to_string()),
                updated_workspace_ids: vec![],
                model: Some("claude-opus-4-20250514".to_string()),
                files_changed: Some(5),
                gate_result: Some("pass".to_string()),
                ..Default::default()
            },
        )?;

        // Telemetry now lives on the session itself (slice 04 collapse).
        assert_eq!(ended.id, ended.id); // session id round-trip — implicit
        assert_eq!(started.id, ended.id);
        assert!(ended.duration_secs.is_some());
        assert!(ended.duration_secs.unwrap() >= 0);
        assert_eq!(ended.primary_provider.as_deref(), Some("claude"));
        assert_eq!(ended.model.as_deref(), Some("claude-opus-4-20250514"));
        assert_eq!(ended.agent_id, started.agent_id);
        assert_eq!(ended.files_changed, Some(5));
        assert_eq!(ended.gate_result.as_deref(), Some("pass"));
        Ok(())
    }

    #[test]
    fn session_record_metrics_default_to_none() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/metrics-defaults".to_string(),
                ..Default::default()
            },
        )?;

        start_workspace_session(&ship_dir, "feature/metrics-defaults", None, None, None)?;

        let ended = end_workspace_session(
            &ship_dir,
            "feature/metrics-defaults",
            EndWorkspaceSessionRequest::default(),
        )?;

        assert!(ended.duration_secs.is_some());
        assert!(ended.duration_secs.unwrap() >= 0);
        assert!(ended.primary_provider.is_some());
        assert!(ended.model.is_none());
        assert!(ended.files_changed.is_none());
        assert!(ended.gate_result.is_none());
        Ok(())
    }

    #[test]
    fn workspace_session_start_attaches_existing_active_session() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/session-dupe".to_string(),
                ..Default::default()
            },
        )?;

        let first = start_workspace_session(
            &ship_dir,
            "feature/session-dupe",
            Some("one".into()),
            None,
            None,
        )?;
        let attached = start_workspace_session(
            &ship_dir,
            "feature/session-dupe",
            Some("two".into()),
            None,
            None,
        )?;

        assert_eq!(attached.id, first.id);
        let sessions = list_workspace_sessions(&ship_dir, Some("feature/session-dupe"), 10)?;
        assert_eq!(sessions.len(), 1);
        Ok(())
    }

    #[test]
    fn workspace_session_list_filters_by_branch_workspace() -> Result<()> {
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

        let a = start_workspace_session(&ship_dir, "feature/a", None, None, None)?;
        end_workspace_session(
            &ship_dir,
            "feature/a",
            EndWorkspaceSessionRequest::default(),
        )?;
        let b = start_workspace_session(&ship_dir, "feature/b", None, None, None)?;

        let all = list_workspace_sessions(&ship_dir, None, 10)?;
        assert!(all.iter().any(|session| session.id == a.id));
        assert!(all.iter().any(|session| session.id == b.id));

        let only_a = list_workspace_sessions(&ship_dir, Some("feature/a"), 10)?;
        assert!(
            only_a
                .iter()
                .all(|session| session.workspace_branch == "feature/a")
        );
        assert_eq!(only_a.len(), 1);
        assert_eq!(only_a[0].id, a.id);
        Ok(())
    }

    #[test]
    fn workspace_session_start_allows_explicit_primary_provider() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-ok".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/provider-ok",
            Some("Pin provider".to_string()),
            None,
            Some("claude".to_string()),
        )?;
        assert_eq!(session.primary_provider.as_deref(), Some("claude"));
        Ok(())
    }

    #[test]
    fn workspace_session_start_rejects_provider_outside_allowed_targets() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-deny".to_string(),
                ..Default::default()
            },
        )?;

        let err = start_workspace_session(
            &ship_dir,
            "feature/provider-deny",
            None,
            None,
            Some("gemini".to_string()),
        )
        .expect_err("provider outside allowed targets should be rejected");

        assert!(
            err.to_string()
                .contains("Provider 'gemini' not allowed for workspace")
        );
        Ok(())
    }

    #[test]
    fn start_workspace_session_falls_back_to_claude_when_no_providers_configured() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        // Clear project-level providers so the default fallback kicks in.
        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec![];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/no-provider".to_string(),
                ..Default::default()
            },
        )?;

        // Provider resolution falls back to "claude" by design (agents/config.rs
        // resolution priority level 4). Session start should succeed.
        let session = start_workspace_session(
            &ship_dir,
            "feature/no-provider",
            Some("fallback test".to_string()),
            None,
            None,
        )?;
        assert_eq!(
            session.primary_provider.as_deref(),
            Some("claude"),
            "empty project providers should fall back to claude"
        );
        Ok(())
    }

    #[test]
    fn start_session_auto_creates_workspace_when_missing() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        // No workspace exists yet for this branch.
        assert!(get_workspace(&ship_dir, "feature/auto-create")?.is_none());

        let session = start_workspace_session(
            &ship_dir,
            "feature/auto-create",
            Some("auto-create test".to_string()),
            None,
            None,
        )?;

        assert_eq!(session.status, WorkspaceSessionStatus::Active);
        let ws = get_workspace(&ship_dir, "feature/auto-create")?;
        assert!(ws.is_some(), "workspace should have been auto-created");
        Ok(())
    }

    #[test]
    fn start_session_on_existing_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/existing".to_string(),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/existing",
            Some("test".to_string()),
            None,
            None,
        )?;
        assert_eq!(session.status, WorkspaceSessionStatus::Active);

        let ws = get_workspace(&ship_dir, "feature/existing")?;
        assert!(ws.is_some(), "workspace should exist");
        Ok(())
    }

    #[test]
    fn end_session_fails_when_no_active_session() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/no-active".to_string(),
                ..Default::default()
            },
        )?;

        let err = end_workspace_session(
            &ship_dir,
            "feature/no-active",
            EndWorkspaceSessionRequest::default(),
        )
        .expect_err("should fail with no active session");

        assert!(
            err.to_string().contains("No active workspace session"),
            "error should mention no active session, got: {}",
            err
        );
        Ok(())
    }

    #[test]
    fn end_session_for_missing_workspace_fails() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let err = end_workspace_session(
            &ship_dir,
            "feature/nonexistent",
            EndWorkspaceSessionRequest::default(),
        )
        .expect_err("should fail for missing workspace");

        assert!(
            err.to_string().contains("Workspace not found"),
            "error should mention workspace not found, got: {}",
            err
        );
        Ok(())
    }

    #[test]
    fn concurrent_session_starts_return_same_session() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/concurrent".to_string(),
                ..Default::default()
            },
        )?;

        let s1 = start_workspace_session(
            &ship_dir,
            "feature/concurrent",
            Some("first".to_string()),
            None,
            None,
        )?;
        let s2 = start_workspace_session(
            &ship_dir,
            "feature/concurrent",
            Some("second".to_string()),
            None,
            None,
        )?;
        let s3 = start_workspace_session(
            &ship_dir,
            "feature/concurrent",
            Some("third".to_string()),
            None,
            None,
        )?;

        assert_eq!(s1.id, s2.id);
        assert_eq!(s2.id, s3.id);
        // Only one session should exist.
        let sessions = list_workspace_sessions(&ship_dir, Some("feature/concurrent"), 10)?;
        assert_eq!(sessions.len(), 1);
        Ok(())
    }

}
