use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

/// Resolve the workspace name from the current git branch.
fn current_workspace_name() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("not in a git repository");
    }
    let branch = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    Ok(branch)
}

/// List trashed files for the current workspace.
pub fn run_trash_list() -> Result<()> {
    let ws = current_workspace_name()?;
    let entries = runtime::workspace::trash::list_trash(&ws)?;
    if entries.is_empty() {
        println!("No trashed files for workspace '{ws}'.");
        return Ok(());
    }

    println!("Trashed files for workspace '{ws}':\n");
    for entry in &entries {
        let age = chrono::Utc::now()
            .signed_duration_since(entry.trashed_at);
        let age_str = if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else {
            format!("{}m ago", age.num_minutes())
        };
        println!(
            "  {:<30} {:>8}  {}  (from {})",
            entry.name,
            human_size(entry.size),
            age_str,
            entry.original_path.display(),
        );
    }
    println!("\nRestore: ship workspace restore <filename>");
    Ok(())
}

/// Force purge all trash.
pub fn run_trash_purge() -> Result<()> {
    let removed = runtime::workspace::trash::purge_old_trash(Duration::ZERO)?;
    println!("Purged {removed} trash entries.");
    Ok(())
}

/// Restore a file from trash.
pub fn run_restore(filename: &str, dest: Option<PathBuf>) -> Result<()> {
    let ws = current_workspace_name()?;
    let target = runtime::workspace::trash::restore_file(
        &ws,
        filename,
        dest.as_deref(),
    )?;
    println!("Restored '{}' to {}", filename, target.display());
    Ok(())
}

fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
