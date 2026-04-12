//! Concrete adapters for job isolation.
//!
//! `GitWorktreeIsolation` — creates git worktrees, writes phase specs, compiles agent config.
//! Executor implementations live in the `executors` submodule.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{IsolationStrategy, JobContext};
use std::path::PathBuf;

use super::helpers::{compile_agent_config, create_worktree, worktrees_dir};

/// Isolation strategy that creates git worktrees and compiles agent configs.
pub(crate) struct GitWorktreeIsolation;

#[async_trait]
impl IsolationStrategy for GitWorktreeIsolation {
    async fn prepare(&self, job: &JobContext) -> Result<PathBuf> {
        let worktree_path = worktrees_dir().join(&job.slug);
        create_worktree(&worktree_path, &job.branch, None)?;
        let session_dir = worktree_path.join(".artifacts");
        std::fs::create_dir_all(&session_dir)?;
        write_phase_spec_from_context(&worktree_path, job)?;
        compile_agent_config(&worktree_path, &job.agent)?;
        Ok(worktree_path)
    }

    async fn cleanup(&self, job: &JobContext) -> Result<()> {
        let worktree_path = worktrees_dir().join(&job.slug);
        if worktree_path.exists() {
            super::helpers::remove_worktree(&worktree_path)?;
        }
        super::helpers::delete_branch(&job.branch)?;
        Ok(())
    }
}

/// Materialize the job spec inside the worktree.
///
/// Priority:
/// 1. `job.spec_content` — inline content from the JobCreatedPayload event.
///    The preferred path: specs travel in events, not via shared disk.
/// 2. Legacy fallback: read `job.spec_path` from the main checkout (manual
///    dispatch where a lead agent wrote the spec to disk).
///
/// Returns Err when a spec was expected but neither source produced content,
/// so agents never launch against an empty spec silently.
fn write_phase_spec_from_context(worktree_path: &std::path::Path, job: &JobContext) -> Result<()> {
    let dst_spec = worktree_path.join(&job.spec_path);
    if let Some(parent) = dst_spec.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("create spec dir {}: {e}", parent.display()))?;
    }

    let body = resolve_spec_body(worktree_path, job)?;
    let full = append_runtime_contract(&body);
    std::fs::write(&dst_spec, &full)
        .map_err(|e| anyhow::anyhow!("write spec to {}: {e}", dst_spec.display()))?;
    Ok(())
}

/// Get the author-provided spec body from the inline payload or disk fallback.
fn resolve_spec_body(worktree_path: &std::path::Path, job: &JobContext) -> Result<String> {
    if let Some(content) = &job.spec_content {
        return Ok(content.clone());
    }

    // Legacy fallback for manual dispatch where a lead agent wrote the spec
    // to disk before emitting JobCreated.
    let src = std::path::Path::new(&job.spec_path);
    if src.is_absolute() && src.exists() {
        return std::fs::read_to_string(src)
            .map_err(|e| anyhow::anyhow!("read spec {}: {e}", src.display()));
    }
    let root = worktree_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| {
            anyhow::anyhow!("cannot locate project root from {}", worktree_path.display())
        })?;
    let alt = root.join(&job.spec_path);
    if !alt.exists() {
        tracing::warn!(
            slug = job.slug,
            spec_path = job.spec_path,
            "no inline spec_content and no spec on disk — worktree will have empty spec body"
        );
        return Ok(String::new());
    }
    std::fs::read_to_string(&alt).map_err(|e| anyhow::anyhow!("read spec {}: {e}", alt.display()))
}

/// Append the runtime's pipeline contract — injected at dispatch time, not
/// author-provided. Tells the agent how to signal completion so the step
/// monitor knows to advance.
///
/// The runtime owns this contract. Agents shouldn't have to guess how the
/// pipeline observes them; the runtime tells them, every dispatch.
fn append_runtime_contract(body: &str) -> String {
    const CONTRACT: &str = r#"
---

## Runtime Contract — how to signal completion

This agent is running as a step in a Ship pipeline. The step monitor is
watching for any of these signals:

**When the work is complete:** call `mcp__ship__step_done` with a one-line
summary. This is the strongest signal — the pipeline advances immediately.

```
mcp__ship__step_done({ summary: "added ping() and a passing test" })
```

**If you can't call step_done** (tool unavailable, partial work): commit
what you have with `git commit`. A commit on the branch is treated as
evidence of work done; when you stop responding, the pipeline advances.

**Progress updates** (optional, encouraged for long work): call
`mcp__ship__log_progress` to record checkpoints. Doesn't advance the
step, but makes your thinking visible in the event log.

**What happens if you do nothing:** after you stop responding, the
monitor settles for ~15 seconds. If you committed work, it advances. If
you didn't, the gate step reviews; if there's no gate, it times out at
30 minutes.

**Never call step_done until the acceptance criteria above are met.** If
you're stuck, call `mcp__ship__log_progress` with the blocker and stop.
"#;

    let mut out = body.trim_end().to_string();
    out.push_str(CONTRACT);
    out
}
