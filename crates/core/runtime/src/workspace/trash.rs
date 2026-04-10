use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::project::get_global_dir;

/// Metadata for a trashed file.
#[derive(Debug, Clone, Serialize)]
pub struct TrashEntry {
    /// Original absolute path of the file.
    pub original_path: PathBuf,
    /// Path inside the trash directory.
    pub trash_path: PathBuf,
    /// File name.
    pub name: String,
    /// When the file was trashed.
    pub trashed_at: DateTime<Utc>,
    /// File size in bytes.
    pub size: u64,
}

/// Copy a file to trash before it is destroyed.
///
/// Layout: `{trash_root}/{workspace}/{timestamp}/{filename}`
pub fn trash_file(workspace_name: &str, path: &Path) -> Result<PathBuf> {
    trash_file_at(&get_global_dir()?.join("trash"), workspace_name, path)
}

/// List all trashed files for a workspace, newest first.
pub fn list_trash(workspace_name: &str) -> Result<Vec<TrashEntry>> {
    list_trash_at(&get_global_dir()?.join("trash"), workspace_name)
}

/// Restore a trashed file to `dest` (or its original location).
pub fn restore_file(
    workspace_name: &str,
    file_name: &str,
    dest: Option<&Path>,
) -> Result<PathBuf> {
    restore_file_at(
        &get_global_dir()?.join("trash"),
        workspace_name,
        file_name,
        dest,
    )
}

/// Purge trash entries older than `max_age`. Returns count removed.
pub fn purge_old_trash(max_age: Duration) -> Result<u64> {
    purge_old_trash_at(&get_global_dir()?.join("trash"), max_age)
}

// ── Internal implementations (testable with explicit paths) ─────────────

pub fn trash_file_at(
    trash_root: &Path,
    workspace_name: &str,
    path: &Path,
) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    if !path.exists() {
        anyhow::bail!("cannot trash non-existent file: {}", path.display());
    }

    let timestamp = Utc::now().format("%Y%m%d-%H%M%S%.3f").to_string();
    let trash_dir = trash_root.join(workspace_name).join(&timestamp);
    fs::create_dir_all(&trash_dir)
        .with_context(|| format!("create trash dir {}", trash_dir.display()))?;

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let dest = trash_dir.join(&file_name);
    fs::copy(&path, &dest)
        .with_context(|| format!("copy {} to trash", path.display()))?;

    // Write a small manifest so we can restore to the original location.
    let manifest = trash_dir.join(".manifest");
    let manifest_content = serde_json::json!({
        "original_path": path.to_string_lossy(),
        "trashed_at": Utc::now().to_rfc3339(),
    });
    fs::write(&manifest, manifest_content.to_string())?;

    Ok(dest)
}

pub fn list_trash_at(
    trash_root: &Path,
    workspace_name: &str,
) -> Result<Vec<TrashEntry>> {
    let root = trash_root.join(workspace_name);
    if !root.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for dir_entry in fs::read_dir(&root)? {
        let dir_entry = dir_entry?;
        let dir_path = dir_entry.path();
        if !dir_path.is_dir() {
            continue;
        }
        let manifest_path = dir_path.join(".manifest");
        let original_path = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)?;
            serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .and_then(|v| v["original_path"].as_str().map(PathBuf::from))
                .unwrap_or_default()
        } else {
            PathBuf::new()
        };

        let trashed_at = parse_timestamp_from_dir_name(
            &dir_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
        );

        for file_entry in fs::read_dir(&dir_path)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();
            if file_path
                .file_name()
                .map(|n| n == ".manifest")
                .unwrap_or(false)
            {
                continue;
            }
            if !file_path.is_file() {
                continue;
            }
            let meta = fs::metadata(&file_path)?;
            entries.push(TrashEntry {
                original_path: original_path.clone(),
                trash_path: file_path.clone(),
                name: file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                trashed_at,
                size: meta.len(),
            });
        }
    }
    // Newest first.
    entries.sort_by(|a, b| b.trashed_at.cmp(&a.trashed_at));
    Ok(entries)
}

pub fn restore_file_at(
    trash_root: &Path,
    workspace_name: &str,
    file_name: &str,
    dest: Option<&Path>,
) -> Result<PathBuf> {
    let entries = list_trash_at(trash_root, workspace_name)?;
    let entry = entries
        .iter()
        .find(|e| e.name == file_name)
        .ok_or_else(|| anyhow::anyhow!("no trashed file named '{file_name}'"))?;

    let target = match dest {
        Some(d) => d.to_path_buf(),
        None => {
            if entry.original_path.as_os_str().is_empty() {
                anyhow::bail!(
                    "no original path recorded for '{file_name}'; specify a destination"
                );
            }
            entry.original_path.clone()
        }
    };

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&entry.trash_path, &target)
        .with_context(|| format!("restore {} to {}", entry.name, target.display()))?;

    // Remove the trash entry directory.
    if let Some(trash_parent) = entry.trash_path.parent() {
        let _ = fs::remove_dir_all(trash_parent);
    }

    Ok(target)
}

pub fn purge_old_trash_at(trash_root: &Path, max_age: Duration) -> Result<u64> {
    if !trash_root.exists() {
        return Ok(0);
    }

    let cutoff = Utc::now() - chrono::Duration::from_std(max_age)?;
    let mut removed = 0u64;

    for ws_entry in fs::read_dir(trash_root)? {
        let ws_entry = ws_entry?;
        let ws_path = ws_entry.path();
        if !ws_path.is_dir() {
            continue;
        }
        for ts_entry in fs::read_dir(&ws_path)? {
            let ts_entry = ts_entry?;
            let ts_path = ts_entry.path();
            if !ts_path.is_dir() {
                continue;
            }
            let dir_name = ts_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let trashed_at = parse_timestamp_from_dir_name(&dir_name);
            if trashed_at < cutoff {
                fs::remove_dir_all(&ts_path)?;
                removed += 1;
            }
        }
        // Remove empty workspace dirs.
        if fs::read_dir(&ws_path)?.next().is_none() {
            let _ = fs::remove_dir(&ws_path);
        }
    }
    Ok(removed)
}

fn parse_timestamp_from_dir_name(name: &str) -> DateTime<Utc> {
    chrono::NaiveDateTime::parse_from_str(name, "%Y%m%d-%H%M%S%.3f")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(name, "%Y%m%d-%H%M%S"))
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC)
}

#[cfg(test)]
mod tests;
