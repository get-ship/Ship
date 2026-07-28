//! Startup orphan cleanup — removes worktrees for terminal sessions.
//!
//! After a daemon restart, bare agent-session dispatches that previously
//! reached a terminal status (`completed` / `failed` / `cancelled` /
//! `suspended`) may still have worktrees, branches, and tmux sessions
//! on disk. This module scans the canonical sessions projection on
//! startup and cleans them up.
//!
//! ## Engine cut (2026-07-10)
//!
//! Repointed from the orchestrator-era `pipeline_runs` projection to the
//! canonical sessions read model. Coverage proof (old → new):
//!
//! - **Old path reaped** terminal `pipeline_runs` rows
//!   (Merged | Failed) with on-disk worktrees. Two conditions:
//!   1. `pipeline_runs.status ∈ {Merged, Failed}` (terminal lifecycle state).
//!   2. `<worktrees_dir>/<slug>` (or `row.worktree`) still exists on disk.
//!   The reaper called `cleanup_orphan` → `kill_tmux_session` +
//!   `remove_worktree` + `delete_branch` + `emit_executor_session_destroyed`.
//!
//! - **New path reaps** terminal `canonical_sessions` rows
//!   (Completed | Failed | Cancelled | Suspended) with on-disk worktrees.
//!   Two conditions:
//!   1. `canonical_sessions.status.is_terminal()`.
//!   2. `<worktrees_dir>/<slug>` still exists on disk (sessions carry an
//!      optional `slug`; the reaper only acts on the slugs that exist).
//!   Same teardown path. `emit_executor_session_destroyed` is gated to
//!   the slug the projection actually recorded.
//!
//! **Coverage:** every terminal pipeline-run became a terminal canonical
//! session (orchestrator emitted `session.completed/failed/cancelled` for
//! the composed sessions; bare dispatches always had terminal session
//! events). The new path covers the same set of slugs the old path did,
//! plus any bare-session dispatches the old path missed. The
//! "Merged" → "Completed" mapping is a strict superset (every Merged run
//! had a Completed session under it; nothing changes for Failed runs).
//! No silent gap.

use std::sync::Arc;

use ship_agents::services::sessions::canonical_projection::load_sessions;
use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::emit::emit_executor_session_destroyed;
use ship_substrate::events::KernelRouter;
use tokio::sync::Mutex;

use bridge::helpers::{repo_dir, worktrees_dir};

/// Scan for orphaned worktrees from previous daemon runs.
///
/// Loads the canonical sessions projection and finds terminal sessions
/// (Completed / Failed / Cancelled / Suspended) that still have a
/// worktree directory on disk. Cleans up each one using the standard
/// teardown path so cleanup is consistent and idempotent.
pub(crate) async fn cleanup_orphaned_worktrees(kernel: &Arc<Mutex<KernelRouter>>) {
    let sessions = match load_sessions() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("job-dispatch: orphan scan failed to load sessions: {e}");
            return;
        }
    };

    let worktrees_root = match worktrees_dir() {
        Ok(root) => root,
        Err(e) => {
            tracing::warn!("job-dispatch: orphan scan skipped: {e}");
            return;
        }
    };

    let orphans: Vec<_> = sessions
        .into_iter()
        .filter(|s| s.status.is_terminal())
        .filter_map(|s| s.slug.map(|slug| (slug, s.status)))
        .filter(|(slug, _)| worktrees_root.join(slug).exists())
        .collect();

    if orphans.is_empty() {
        tracing::debug!("job-dispatch: no orphaned worktrees found");
        return;
    }

    tracing::info!(
        count = orphans.len(),
        "job-dispatch: cleaning up orphaned worktrees"
    );

    for (slug, status) in orphans {
        tracing::info!(
            slug = %slug,
            session_status = status.as_db_str(),
            "job-dispatch: removing orphaned worktree"
        );
        cleanup_orphan(&slug, kernel).await;
    }

    tracing::info!("job-dispatch: orphan cleanup complete");
}

/// Clean up a single orphaned session's resources.
async fn cleanup_orphan(slug: &str, kernel: &Arc<Mutex<KernelRouter>>) {
    use bridge::helpers::kill_tmux_session;

    let tmux_session = format!("job-{slug}");
    let worktree_path = match worktrees_dir() {
        Ok(root) => root.join(slug),
        Err(e) => {
            tracing::warn!(slug, "orphan: worktree cleanup skipped: {e}");
            return;
        }
    };
    let branch = format!("job/{slug}");

    if let Err(e) = kill_tmux_session(&tmux_session) {
        tracing::warn!(slug, "orphan: tmux cleanup failed: {e}");
    }
    // Destruction intent fires regardless of whether the session was alive;
    // the operator-facing fact is that we've torn it down.
    emit_executor_session_destroyed(
        kernel,
        slug,
        "tmux",
        &tmux_session,
        DestructionCause::External,
    )
    .await;
    if worktree_path.exists()
        && let Err(e) = ship_git::write::remove_worktree(&repo_dir(), &worktree_path, true)
    {
        tracing::warn!(slug, "orphan: worktree cleanup failed: {e}");
    }
    if let Err(e) = ship_git::write::delete_branch(&repo_dir(), &branch, false) {
        tracing::warn!(slug, "orphan: branch cleanup failed: {e}");
    }
}

/// Test-only helper exposing the per-slug cleanup path for downstream
/// integration tests. Kept on the public surface (not `#[cfg(test)]`)
/// so tests in `apps/shipd` can build it; the surface is documented as
/// test-only.
pub mod tests_only {
    use super::*;

    pub async fn cleanup_orphan_for_test(slug: &str, kernel: &Arc<Mutex<KernelRouter>>) {
        super::cleanup_orphan(slug, kernel).await;
    }
}
