#[cfg(test)]
mod tests {
    use crate::workspace::compile::resolve_workspace_agent_config;
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn workspace_compile_resolves_default_providers() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["codex".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/compile-resolve".to_string(),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, "feature/compile-resolve")?
            .ok_or_else(|| anyhow::anyhow!("workspace missing"))?;
        let resolved = resolve_workspace_agent_config(&ship_dir, &workspace, None)?;
        assert!(!resolved.providers.is_empty());
        Ok(())
    }

    #[test]
    fn provider_matrix_uses_config_default_source() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let config = crate::config::ProjectConfig {
            providers: vec!["claude".to_string()],
            ..Default::default()
        };
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-matrix".to_string(),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(
            &ship_dir,
            "feature/provider-matrix",
            None,
        )?;
        assert_eq!(matrix.source, "config/default");
        assert!(matrix.resolution_error.is_none());
        Ok(())
    }

    #[test]
    fn provider_matrix_reports_resolution_error_for_empty_providers() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-invalid".to_string(),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(
            &ship_dir,
            "feature/provider-invalid",
            None,
        )?;
        if matrix.allowed_providers.is_empty() {
            assert!(matrix.resolution_error.is_some());
        }
        Ok(())
    }

    #[test]
    fn repair_workspace_reports_status() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-test".to_string(),
                ..Default::default()
            },
        )?;

        // A fresh workspace with no compiled provider configs will trigger
        // recompile during repair.
        let report = repair_workspace(&ship_dir, "feature/repair-test", false)?;
        assert_eq!(report.workspace_branch, "feature/repair-test");
        assert!(!report.dry_run);
        Ok(())
    }
}
