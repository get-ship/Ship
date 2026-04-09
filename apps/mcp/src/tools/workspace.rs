use std::path::Path;
use std::process::Command as ProcessCommand;

use crate::requests::CreateWorkspaceRequest;
use crate::util::configured_worktree_dir;

pub fn create_workspace(project_dir: &Path, req: CreateWorkspaceRequest) -> String {
    let project_root = project_dir;

    let branch = req
        .branch
        .as_deref()
        .map(|b| b.to_string())
        .unwrap_or_else(|| {
            req.name
                .to_ascii_lowercase()
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' {
                        c
                    } else {
                        '-'
                    }
                })
                .collect::<String>()
                .trim_matches('-')
                .to_string()
        });

    let worktrees_dir = configured_worktree_dir(project_root);
    let worktree_path = worktrees_dir.join(&branch);
    let base_branch = req.base_branch.as_deref().unwrap_or("main");

    if let Err(e) = std::fs::create_dir_all(&worktrees_dir) {
        return format!("Error: could not create worktrees dir: {}", e);
    }
    if let Some(msg) = create_git_worktree(project_root, &worktree_path, &branch, base_branch) {
        return msg;
    }
    if let Err(warn) = write_workspace_config(
        &worktree_path,
        &req.name,
        "feature",
        &req.preset_id,
        &req.file_scope,
    ) {
        return warn;
    }
    format!(
        "Created workspace '{}' (branch: {})\nWorktree: {}",
        req.name,
        branch,
        worktree_path.display()
    )
}

fn create_git_worktree(
    project_root: &Path,
    worktree_path: &Path,
    branch: &str,
    base_branch: &str,
) -> Option<String> {
    let status = ProcessCommand::new("git")
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap_or_default(),
            "-b",
            branch,
            base_branch,
        ])
        .current_dir(project_root)
        .status();

    let ok = match status {
        Ok(s) => s.success(),
        Err(e) => return Some(format!("Error running git worktree add: {}", e)),
    };

    if !ok {
        let status2 = ProcessCommand::new("git")
            .args([
                "worktree",
                "add",
                worktree_path.to_str().unwrap_or_default(),
                branch,
            ])
            .current_dir(project_root)
            .status();
        match status2 {
            Ok(s) if s.success() => {}
            Ok(_) => {
                return Some(format!(
                    "Error: git worktree add failed for branch '{}'. \
                    The branch may not exist or the worktree path is already in use.",
                    branch
                ));
            }
            Err(e) => return Some(format!("Error running git worktree add: {}", e)),
        }
    }
    None
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct WorkspaceConfig {
    pub name: String,
    pub kind: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_scope: Option<String>,
}

fn write_workspace_config(
    worktree_path: &Path,
    name: &str,
    kind: &str,
    preset_id: &Option<String>,
    file_scope: &Option<String>,
) -> Result<(), String> {
    let config = WorkspaceConfig {
        name: name.to_string(),
        kind: kind.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        preset_id: preset_id.clone(),
        file_scope: file_scope.clone(),
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize workspace config: {}", e))?;
    let path = worktree_path.join("workspace.jsonc");
    std::fs::write(&path, json).map_err(|e| {
        format!(
            "Warning: worktree created at '{}' but failed to write workspace.jsonc: {}",
            worktree_path.display(),
            e
        )
    })
}
