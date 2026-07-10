use anyhow::{Context, Result};
use std::path::PathBuf;

pub(super) fn daemon_cleanup() -> Result<()> {
    use ship_agents::services::sessions::canonical_projection::{load_sessions, CanonicalSessionStatus};

    let sessions = load_sessions().context("failed to load canonical sessions projection")?;

    let repo_dir = if let Ok(dir) = ship_substrate::env::read_raw(ship_substrate::env::Var::RepoDir) {
        PathBuf::from(dir)
    } else {
        std::env::current_dir().context("cannot determine working directory")?
    };

    // Workflow-side tool: resolves the explicit `SHIP_WORKTREE_DIR` knob
    // itself (same as the dispatch shell scripts). Never guesses a layout.
    let worktrees_base = match ship_substrate::env::read_raw(ship_substrate::env::Var::WorktreeDir) {
        Ok(dir) if !dir.is_empty() => {
            let expanded = dir.strip_prefix("~/").and_then(|rest| home::home_dir().map(|h| h.join(rest)));
            expanded.unwrap_or_else(|| PathBuf::from(dir))
        }
        _ => anyhow::bail!("missing worktrees.dir: set SHIP_WORKTREE_DIR before daemon cleanup"),
    };

    // Reap terminal sessions (Completed / Failed / Cancelled / Suspended)
    // whose worktree is still on disk. The engine's `pipeline_runs`
    // projection is gone (2026-07-10 cut); the canonical sessions projection
    // is the post-cut source of truth for "this session is done — clean up
    // its worktree". See apps/shipd/src/dispatch/orphan_cleanup.rs for the
    // in-process sibling.
    let orphans: Vec<_> = sessions
        .into_iter()
        .filter(|s| s.status.is_terminal())
        .filter_map(|s| s.slug.clone().map(|slug| (slug, s)))
        .filter(|(slug, _)| worktrees_base.join(slug).exists())
        .collect();

    if orphans.is_empty() {
        println!("No orphaned worktrees found.");
        return Ok(());
    }

    println!("Found {} orphaned worktree(s):", orphans.len());

    let mut cleaned = 0usize;
    let mut errors = 0usize;

    for (slug, session) in &orphans {
        let worktree_path = worktrees_base.join(slug);
        let branch = format!("job/{slug}");
        let tmux_session = format!("job-{slug}");

        println!(
            "  {} — {} [status={}]",
            slug,
            worktree_path.display(),
            session.status.as_db_str()
        );

        // Kill tmux session (best-effort).
        let _ = std::process::Command::new("tmux")
            .args(["kill-session", "-t", &tmux_session])
            .status();

        // Remove git worktree.
        let wt_str = match worktree_path.to_str() {
            Some(s) => s,
            None => {
                eprintln!("    error: worktree path is not valid UTF-8, skipping");
                errors += 1;
                continue;
            }
        };
        let remove_ok = std::process::Command::new("git")
            .args(["worktree", "remove", wt_str, "--force"])
            .current_dir(&repo_dir)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !remove_ok {
            eprintln!("    warning: worktree removal failed for {}", worktree_path.display());
            errors += 1;
            continue;
        }

        // Delete branch (best-effort — it may already be gone).
        let _ = std::process::Command::new("git")
            .args(["branch", "-D", &branch])
            .current_dir(&repo_dir)
            .status();

        cleaned += 1;
    }

    if errors > 0 {
        println!("Cleaned: {cleaned}, Errors: {errors}");
    } else {
        println!("Cleaned: {cleaned}");
    }
    Ok(())
}
