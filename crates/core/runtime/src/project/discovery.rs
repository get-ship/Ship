//! Ship dir discovery: walk ancestors, follow git worktree pointers, redirect
//! linked worktrees back to the main checkout.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use super::SHIP_DIR_NAME;

/// Resolve the enclosing `.ship` directory from any descendant path.
pub fn ship_dir_from_path(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| {
            ancestor
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == SHIP_DIR_NAME)
        })
        .map(Path::to_path_buf)
}

pub(crate) fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn parse_gitdir_pointer(dot_git_file: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(dot_git_file).ok()?;
    let line = content.lines().next()?.trim();
    let raw = line.strip_prefix("gitdir:")?.trim();
    if raw.is_empty() {
        return None;
    }

    let parsed = PathBuf::from(raw);
    if parsed.is_absolute() {
        Some(parsed)
    } else {
        let base = dot_git_file.parent()?;
        Some(base.join(parsed))
    }
}

fn git_common_dir_from(dot_git_path: &Path) -> Option<PathBuf> {
    if dot_git_path.is_dir() {
        return Some(canonicalize_lossy(dot_git_path));
    }

    let gitdir = canonicalize_lossy(&parse_gitdir_pointer(dot_git_path)?);
    // Worktree pointers usually target: <main>/.git/worktrees/<name>
    // In that case, the common git dir is <main>/.git.
    let worktrees = gitdir.parent()?;
    let marker = worktrees.file_name()?.to_str()?;
    if marker == "worktrees" {
        return worktrees.parent().map(canonicalize_lossy);
    }
    Some(gitdir)
}

fn ship_dir_from_git_worktree(start_dir: &Path) -> Option<PathBuf> {
    for ancestor in start_dir.ancestors() {
        let dot_git = ancestor.join(".git");
        if !dot_git.exists() {
            continue;
        }

        let common_git = git_common_dir_from(&dot_git)?;
        let main_root = common_git.parent()?;
        let ship_candidate = main_root.join(SHIP_DIR_NAME);
        if ship_candidate.exists() && ship_candidate.is_dir() {
            return Some(canonicalize_lossy(&ship_candidate));
        }
    }
    None
}

fn ship_dir_from_git_worktree_pointer(start_dir: &Path) -> Option<PathBuf> {
    for ancestor in start_dir.ancestors() {
        let dot_git = ancestor.join(".git");
        if !dot_git.exists() {
            continue;
        }

        if dot_git.is_dir() {
            return None;
        }

        let gitdir = canonicalize_lossy(&parse_gitdir_pointer(&dot_git)?);
        let worktrees = gitdir.parent()?;
        if worktrees.file_name()?.to_str()? != "worktrees" {
            return None;
        }
        let common_git = canonicalize_lossy(worktrees.parent()?);
        let main_root = common_git.parent()?;
        let ship_candidate = main_root.join(SHIP_DIR_NAME);
        if ship_candidate.exists() && ship_candidate.is_dir() {
            return Some(canonicalize_lossy(&ship_candidate));
        }
        return None;
    }
    None
}

/// Parse `git worktree list --porcelain` output and return the main working
/// tree root if `current_dir` is a linked worktree, or `None` if it is the
/// main worktree (or the output is unparseable).
pub(crate) fn parse_main_worktree_root(porcelain: &str, current_dir: &Path) -> Option<PathBuf> {
    let dir_canonical = canonicalize_lossy(current_dir);
    let worktree_paths: Vec<PathBuf> = porcelain
        .split("\n\n")
        .filter_map(|block| {
            let block = block.trim();
            if block.is_empty() {
                return None;
            }
            let path_str = block
                .lines()
                .find(|l| l.starts_with("worktree "))?
                .strip_prefix("worktree ")?
                .trim();
            Some(canonicalize_lossy(PathBuf::from(path_str).as_path()))
        })
        .collect();

    let main_path = worktree_paths.first()?.clone();

    // Determine which worktree current_dir lives under.
    let current_wt = worktree_paths
        .iter()
        .find(|wt| dir_canonical.starts_with(wt.as_path()))?;

    if *current_wt == main_path {
        return None; // already in the main working tree
    }

    Some(main_path)
}

/// If `dir` is a linked git worktree, return the main working tree root.
/// Uses `git worktree list --porcelain` for detection.
fn main_worktree_root_for(dir: &Path) -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = std::str::from_utf8(&output.stdout).ok()?;
    parse_main_worktree_root(text, dir)
}

fn resolve_project_dir_from_start(
    start_dir: &Path,
    migrate_legacy: bool,
) -> Result<Option<PathBuf>> {
    // 1. Walk up from CWD looking for .ship/ — this finds the current
    //    worktree's own .ship/ before reaching any parent.
    let mut current_dir = start_dir.to_path_buf();
    loop {
        let ship_path = current_dir.join(SHIP_DIR_NAME);
        if ship_path.exists() && ship_path.is_dir() {
            // Worktree dev opt-out: when SHIP_RESPECT_WORKTREE=1 (set by
            // scripts/dev-daemon.sh and other dev tooling), the worktree's
            // local .ship/ wins. This lets developers iterate on per-worktree
            // views/themes/agents without having every edit redirect to the
            // main checkout. Production / single-worktree dev: leave unset
            // and the redirect-to-main behavior below stays canonical.
            let respect_worktree = crate::env::read(crate::env::Var::RespectWorktree)
                .filter(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .is_some();
            if respect_worktree {
                return Ok(Some(canonicalize_lossy(&ship_path)));
            }

            // Default: if this .ship/ lives inside a linked git worktree,
            // redirect to the main working tree's .ship/ so worktrees never
            // register as separate projects in ~/.ship/projects/.
            // Primary detection: git worktree list --porcelain.
            // Fallback: parse the .git file pointer directly (no subprocess).
            let redirected = main_worktree_root_for(&current_dir)
                .and_then(|main_root| {
                    let s = main_root.join(SHIP_DIR_NAME);
                    if s.exists() && s.is_dir() {
                        Some(canonicalize_lossy(&s))
                    } else {
                        None
                    }
                })
                .or_else(|| ship_dir_from_git_worktree_pointer(&current_dir));

            if let Some(main_ship) = redirected {
                return Ok(Some(main_ship));
            }
            return Ok(Some(canonicalize_lossy(&ship_path)));
        }

        if migrate_legacy {
            let legacy_path = current_dir.join(".project");
            if legacy_path.exists() && legacy_path.is_dir() {
                fs::rename(&legacy_path, &ship_path)
                    .context("Failed to migrate .project to .ship")?;
                return Ok(Some(canonicalize_lossy(&ship_path)));
            }
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    // 2. Fallback: if no .ship/ found in ancestors, try resolving via git
    //    worktree pointer to the main checkout's .ship/.
    if let Some(ship_path) = ship_dir_from_git_worktree_pointer(start_dir) {
        return Ok(Some(ship_path));
    }

    Ok(ship_dir_from_git_worktree(start_dir))
}

/// Resolve the canonical `.ship` directory for a given path without using env
/// overrides and without mutating legacy folders.
pub fn resolve_project_ship_dir(start_dir: &Path) -> Option<PathBuf> {
    resolve_project_dir_from_start(start_dir, false)
        .ok()
        .flatten()
}

/// Resolves the .ship directory by searching upwards from the given directory.
/// Also checks for legacy `.project` and migrates it to `.ship` if found.
/// Supports `SHIP_DIR` environment variable override.
pub fn get_project_dir(start_dir: Option<PathBuf>) -> Result<PathBuf> {
    // 1. Check for environment variable override
    if let Some(env_path) = crate::env::read(crate::env::Var::Dir) {
        let path = PathBuf::from(env_path);
        if path.exists() && path.is_dir() {
            return Ok(path);
        }
    }

    // 2. Traversal logic — any directory containing a .ship folder is a project.
    // If none is found, attempt git-worktree resolution back to the main checkout.
    let start = start_dir.unwrap_or(env::current_dir()?);
    if let Some(project_dir) = resolve_project_dir_from_start(&start, true)? {
        return Ok(project_dir);
    }

    Err(anyhow!(
        "Project tracking not initialized in this directory or its parents. Run `ship init` to create a .ship directory."
    ))
}
