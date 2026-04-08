//! Filesystem watcher for workspace state changes.
//!
//! Monitors three directories for changes that indicate workspace state drift:
//! - `.git/worktrees/` — worktree added/removed
//! - `.git/HEAD` — branch switched
//! - `.ship/` — config changed, recompile needed
//!
//! On change: runs reconciliation and emits SSE events via the kernel router.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use runtime::events::types::{WorkspaceBranchSwitched, WorkspaceConfigChanged};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing;

/// Spawn the filesystem watcher as a background task.
///
/// Watches the project root's `.git/` and `.ship/` directories. Events are
/// debounced (500ms) and dispatched to the kernel router as workspace events.
/// Returns the watcher handle (drop to stop watching).
pub async fn spawn_workspace_watcher(
    kernel: Arc<Mutex<runtime::events::KernelRouter>>,
    ship_dir: PathBuf,
) -> Option<RecommendedWatcher> {
    let project_root = match ship_dir.parent() {
        Some(p) => p.to_path_buf(),
        None => {
            tracing::warn!("workspace-watcher: cannot determine project root from ship_dir");
            return None;
        }
    };

    let git_dir = project_root.join(".git");
    if !git_dir.is_dir() {
        tracing::info!("workspace-watcher: no .git directory, skipping filesystem watcher");
        return None;
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WatchEvent>();
    let ship_dir_clone = ship_dir.clone();
    let tx_clone = tx.clone();

    let mut watcher = match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if let Some(watch_event) = classify_event(&event, &ship_dir_clone) {
                let _ = tx_clone.send(watch_event);
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("workspace-watcher: failed to create watcher: {e}");
            return None;
        }
    };

    // Watch .git/worktrees/ for worktree add/remove.
    let worktrees_dir = git_dir.join("worktrees");
    if worktrees_dir.is_dir() {
        if let Err(e) = watcher.watch(&worktrees_dir, RecursiveMode::NonRecursive) {
            tracing::warn!("workspace-watcher: failed to watch .git/worktrees/: {e}");
        }
    }

    // Watch .git/HEAD for branch switches.
    let head_file = git_dir.join("HEAD");
    if head_file.exists() {
        if let Err(e) = watcher.watch(&head_file, RecursiveMode::NonRecursive) {
            tracing::warn!("workspace-watcher: failed to watch .git/HEAD: {e}");
        }
    }

    // Watch .ship/ for config changes.
    if ship_dir.is_dir() {
        if let Err(e) = watcher.watch(&ship_dir, RecursiveMode::Recursive) {
            tracing::warn!("workspace-watcher: failed to watch .ship/: {e}");
        }
    }

    // Spawn the event processing loop.
    let ship_dir_handler = ship_dir.clone();
    tokio::spawn(async move {
        // Debounce: collect events for 500ms then process.
        let debounce = Duration::from_millis(500);
        loop {
            let first = match rx.recv().await {
                Some(e) => e,
                None => break,
            };

            // Collect more events during debounce window.
            let mut events = vec![first];
            let deadline = tokio::time::Instant::now() + debounce;
            loop {
                tokio::select! {
                    maybe = rx.recv() => {
                        match maybe {
                            Some(e) => events.push(e),
                            None => break,
                        }
                    }
                    _ = tokio::time::sleep_until(deadline) => break,
                }
            }

            // Deduplicate by event kind.
            let has_worktree_change = events.iter().any(|e| {
                matches!(e, WatchEvent::WorktreeAdded { .. } | WatchEvent::WorktreeRemoved { .. })
            });
            let has_branch_switch = events.iter().any(|e| matches!(e, WatchEvent::BranchSwitched));
            let has_config_change = events.iter().any(|e| matches!(e, WatchEvent::ConfigChanged { .. }));

            if has_worktree_change {
                handle_worktree_change(&ship_dir_handler).await;
            }
            if has_branch_switch {
                handle_branch_switch(&ship_dir_handler).await;
            }
            if has_config_change {
                let changed = events.iter().find_map(|e| {
                    if let WatchEvent::ConfigChanged { path } = e {
                        path.clone()
                    } else {
                        None
                    }
                });
                handle_config_change(&ship_dir_handler, changed).await;
            }
        }
        tracing::info!("workspace-watcher: event loop ended");
    });

    tracing::info!("workspace-watcher: watching project at {}", project_root.display());
    Some(watcher)
}

// ── Internal types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum WatchEvent {
    WorktreeAdded { name: String },
    WorktreeRemoved { name: String },
    BranchSwitched,
    ConfigChanged { path: Option<String> },
}

/// Classify a raw notify event into our domain events.
fn classify_event(event: &Event, ship_dir: &Path) -> Option<WatchEvent> {
    let paths = &event.paths;
    if paths.is_empty() {
        return None;
    }

    let path = &paths[0];
    let path_str = path.to_string_lossy();

    // .git/worktrees/ changes
    if path_str.contains(".git/worktrees") || path_str.contains(".git\\worktrees") {
        return match event.kind {
            EventKind::Create(_) => {
                let name = path.file_name()?.to_string_lossy().to_string();
                Some(WatchEvent::WorktreeAdded { name })
            }
            EventKind::Remove(_) => {
                let name = path.file_name()?.to_string_lossy().to_string();
                Some(WatchEvent::WorktreeRemoved { name })
            }
            _ => None,
        };
    }

    // .git/HEAD changes
    if path.file_name().map(|f| f == "HEAD").unwrap_or(false)
        && path.parent().map(|p| p.ends_with(".git")).unwrap_or(false)
    {
        return match event.kind {
            EventKind::Modify(_) => Some(WatchEvent::BranchSwitched),
            _ => None,
        };
    }

    // .ship/ changes
    let ship_canonical = ship_dir.to_string_lossy();
    if path_str.starts_with(ship_canonical.as_ref()) {
        return match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                Some(WatchEvent::ConfigChanged {
                    path: Some(path_str.to_string()),
                })
            }
            _ => None,
        };
    }

    None
}

// ── Event handlers ──────────────────────────────────────────────────────────

async fn handle_worktree_change(ship_dir: &Path) {
    match runtime::workspace::reconcile_all_workspaces(ship_dir) {
        Ok(workspaces) => {
            tracing::info!(
                "workspace-watcher: reconciled {} workspaces after worktree change",
                workspaces.len()
            );
        }
        Err(e) => {
            tracing::warn!("workspace-watcher: reconciliation failed: {e}");
        }
    }
}

async fn handle_branch_switch(ship_dir: &Path) {
    // Read current branch from git HEAD.
    let branch = match read_current_branch(ship_dir) {
        Some(b) => b,
        None => return,
    };

    // Emit branch switch event.
    let payload = WorkspaceBranchSwitched {
        previous_branch: None, // We don't track previous state in the watcher.
        new_branch: branch.clone(),
        detected_by: "fs-watcher".to_string(),
    };
    if let Err(e) = runtime::db::workspace_events::emit_workspace_branch_switched(&branch, &payload) {
        tracing::warn!("workspace-watcher: failed to emit branch_switched: {e}");
    }

    // Reconcile the newly active branch.
    let _ = runtime::workspace::reconcile_workspace(ship_dir, &branch);
}

async fn handle_config_change(ship_dir: &Path, changed_path: Option<String>) {
    let branch = match read_current_branch(ship_dir) {
        Some(b) => b,
        None => "ship".to_string(),
    };

    let payload = WorkspaceConfigChanged {
        changed_path,
        detected_by: "fs-watcher".to_string(),
    };
    if let Err(e) = runtime::db::workspace_events::emit_workspace_config_changed(&branch, &payload) {
        tracing::warn!("workspace-watcher: failed to emit config_changed: {e}");
    }
}

/// Read the current branch from .git/HEAD.
fn read_current_branch(ship_dir: &Path) -> Option<String> {
    let project_root = ship_dir.parent()?;
    let head_path = project_root.join(".git").join("HEAD");
    let content = std::fs::read_to_string(head_path).ok()?;
    let trimmed = content.trim();
    if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
        Some(branch.to_string())
    } else {
        None // Detached HEAD.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_worktree_create() {
        let ship_dir = Path::new("/project/.ship");
        let event = Event {
            kind: EventKind::Create(notify::event::CreateKind::Folder),
            paths: vec![PathBuf::from("/project/.git/worktrees/feature-auth")],
            attrs: Default::default(),
        };
        let result = classify_event(&event, ship_dir);
        assert!(matches!(result, Some(WatchEvent::WorktreeAdded { ref name }) if name == "feature-auth"));
    }

    #[test]
    fn classify_worktree_remove() {
        let ship_dir = Path::new("/project/.ship");
        let event = Event {
            kind: EventKind::Remove(notify::event::RemoveKind::Folder),
            paths: vec![PathBuf::from("/project/.git/worktrees/feature-auth")],
            attrs: Default::default(),
        };
        let result = classify_event(&event, ship_dir);
        assert!(matches!(result, Some(WatchEvent::WorktreeRemoved { ref name }) if name == "feature-auth"));
    }

    #[test]
    fn classify_head_modify() {
        let ship_dir = Path::new("/project/.ship");
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/project/.git/HEAD")],
            attrs: Default::default(),
        };
        let result = classify_event(&event, ship_dir);
        assert!(matches!(result, Some(WatchEvent::BranchSwitched)));
    }

    #[test]
    fn classify_ship_config_change() {
        let ship_dir = Path::new("/project/.ship");
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/project/.ship/agents/default.md")],
            attrs: Default::default(),
        };
        let result = classify_event(&event, ship_dir);
        assert!(matches!(result, Some(WatchEvent::ConfigChanged { .. })));
    }

    #[test]
    fn classify_irrelevant_event_returns_none() {
        let ship_dir = Path::new("/project/.ship");
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/project/src/main.rs")],
            attrs: Default::default(),
        };
        let result = classify_event(&event, ship_dir);
        assert!(result.is_none());
    }

    #[test]
    fn read_branch_from_head_content() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project = tmp.path();
        let git_dir = project.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/feature/auth\n").unwrap();

        let ship_dir = project.join(".ship");
        let branch = read_current_branch(&ship_dir);
        assert_eq!(branch, Some("feature/auth".to_string()));
    }

    #[test]
    fn read_branch_detached_head_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project = tmp.path();
        let git_dir = project.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join("HEAD"), "abc123def456\n").unwrap();

        let ship_dir = project.join(".ship");
        let branch = read_current_branch(&ship_dir);
        assert!(branch.is_none());
    }
}
