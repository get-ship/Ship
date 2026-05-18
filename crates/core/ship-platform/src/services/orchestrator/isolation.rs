//! Pure helpers shared across `IsolationStrategy` implementations. Today
//! that's spec materialization — the file I/O involved in writing a job's
//! initial spec into its prepared worktree. Concrete strategies (git
//! worktree, future container-based, etc.) live with the host process
//! that owns the OS resources; the helpers here are pure file ops.

use anyhow::Result;
use std::path::Path;

use crate::services::dispatch_ports::JobContext;

/// Materialize the job spec inside a prepared worktree.
///
/// Priority:
/// 1. `job.spec_content` — inline content from the `PipelineRunStartedPayload` event.
///    The preferred path: specs travel in events, not via shared disk.
/// 2. Legacy fallback: read `job.spec_path` from the main checkout (manual
///    dispatch where a lead agent wrote the spec to disk).
///
/// Returns `Err` when a spec was expected but neither source produced
/// content, so agents never launch against an empty spec silently.
///
/// Note: the orchestrator's pipeline-specs writer overwrites this file
/// with a fresh dispatch prompt right after the isolation strategy's
/// `prepare()` returns. The placeholder is kept so legacy manual-
/// dispatch flows that call only the adapter see something sensible.
pub fn write_phase_spec_from_context(
    worktree_path: &Path,
    job: &JobContext,
) -> Result<()> {
    let dst_spec = worktree_path.join(&job.spec_path);
    if let Some(parent) = dst_spec.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("create spec dir {}: {e}", parent.display()))?;
    }

    let body = if let Some(content) = &job.spec_content {
        content.clone()
    } else {
        // Legacy disk fallback — read from project root.
        let src = Path::new(&job.spec_path);
        if src.is_absolute() && src.exists() {
            std::fs::read_to_string(src)
                .map_err(|e| anyhow::anyhow!("read spec {}: {e}", src.display()))?
        } else {
            String::new()
        }
    };
    std::fs::write(&dst_spec, &body)
        .map_err(|e| anyhow::anyhow!("write spec to {}: {e}", dst_spec.display()))?;
    Ok(())
}
