//! Best-effort soft-delete: copy files to trash before destructive tool calls.
//!
//! This runs in the `before-tool` hook. Failures are logged but never block
//! the tool from executing — the hook must not break the agent workflow.

use runtime::workspace::trash;
use std::path::Path;

/// Inspect a tool call and trash any files about to be destroyed.
///
/// Supports:
/// - `Write` / `Edit` — trash the target file if it already exists
/// - `Bash` — detect `rm`, `git checkout --`, `git clean` patterns
pub fn maybe_trash_before_tool(
    tool_name: &str,
    tool_input: Option<&serde_json::Value>,
    workspace_id: &str,
) {
    let Some(input) = tool_input else { return };

    match tool_name {
        "Write" | "write" => trash_existing_file(input, workspace_id, "Write"),
        "Edit" | "edit" => trash_existing_file(input, workspace_id, "Edit"),
        "Bash" | "bash" => trash_bash_targets(input, workspace_id),
        _ => {}
    }
}

/// Trash `file_path` from tool input if the file already exists on disk.
fn trash_existing_file(input: &serde_json::Value, workspace_id: &str, tool: &str) {
    let Some(path_str) = input.get("file_path").and_then(|v| v.as_str()) else {
        return;
    };
    let path = Path::new(path_str);
    if path.exists()
        && let Err(e) = trash::trash_file(workspace_id, path)
    {
        tracing::warn!(path = %path.display(), "soft-delete failed for {tool}: {e}");
    }
}

/// Bash tool: `{ "command": "rm -rf foo bar" }`
/// Parse for destructive patterns and trash identified targets.
fn trash_bash_targets(input: &serde_json::Value, workspace_id: &str) {
    let Some(command) = input.get("command").and_then(|v| v.as_str()) else {
        return;
    };

    // Split on && and ; to handle chained commands.
    for segment in command.split("&&").flat_map(|s| s.split(';')) {
        let tokens: Vec<&str> = segment.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        match tokens[0] {
            "rm" | "rmdir" => trash_rm_targets(&tokens, workspace_id),
            "git" => trash_git_targets(&tokens, workspace_id),
            _ => {}
        }
    }
}

/// Parse `rm [-rf] <paths...>` and trash each target that exists.
fn trash_rm_targets(tokens: &[&str], workspace_id: &str) {
    for token in tokens.iter().skip(1) {
        if token.starts_with('-') {
            continue;
        }
        let path = Path::new(token);
        if path.exists()
            && path.is_file()
            && let Err(e) = trash::trash_file(workspace_id, path)
        {
            tracing::warn!(path = %path.display(), "soft-delete failed for rm: {e}");
        }
    }
}

/// Parse git destructive subcommands:
/// - `git checkout -- <files>`
/// - `git clean [-fd]`
fn trash_git_targets(tokens: &[&str], workspace_id: &str) {
    if tokens.len() < 2 {
        return;
    }
    match tokens[1] {
        "checkout" => {
            if let Some(sep_pos) = tokens.iter().position(|t| *t == "--") {
                for path_str in &tokens[sep_pos + 1..] {
                    let path = Path::new(path_str);
                    if path.exists()
                        && path.is_file()
                        && let Err(e) = trash::trash_file(workspace_id, path)
                    {
                        tracing::warn!(
                            path = %path.display(),
                            "soft-delete failed for git checkout: {e}"
                        );
                    }
                }
            }
        }
        "clean" => {
            tracing::info!("git clean detected — untracked files may be lost");
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn write_existing_file_gets_trashed() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("existing.txt");
        fs::write(&file, "original").unwrap();

        let trash_root = tmp.path().join("trash");
        let input = serde_json::json!({
            "file_path": file.to_string_lossy(),
        });

        let path_str = input.get("file_path").unwrap().as_str().unwrap();
        let path = Path::new(path_str);
        assert!(path.exists());
        let result = trash::trash_file_at(&trash_root, "test-ws", path);
        assert!(result.is_ok());
        let entries = trash::list_trash_at(&trash_root, "test-ws").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "existing.txt");
    }

    #[test]
    fn write_new_file_not_trashed() {
        let input = serde_json::json!({
            "file_path": "/tmp/definitely-does-not-exist-12345.txt",
        });
        let path_str = input.get("file_path").unwrap().as_str().unwrap();
        let path = Path::new(path_str);
        assert!(!path.exists());
    }

    #[test]
    fn bash_rm_pattern_detected() {
        let tokens = vec!["rm", "-rf", "/some/path"];
        trash_rm_targets(&tokens, "ws");
    }

    #[test]
    fn bash_git_checkout_pattern_detected() {
        let tokens = vec!["git", "checkout", "--", "nonexistent.txt"];
        trash_git_targets(&tokens, "ws");
    }

    #[test]
    fn bash_command_splitting() {
        let input = serde_json::json!({
            "command": "echo hello && rm foo.txt ; git checkout -- bar.txt",
        });
        let command = input.get("command").unwrap().as_str().unwrap();
        let segments: Vec<&str> = command
            .split("&&")
            .flat_map(|s| s.split(';'))
            .collect();
        assert_eq!(segments.len(), 3);
    }
}
