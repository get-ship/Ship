//! Side helpers used by the step monitor's main loop: git evidence
//! gathering, kill-on-timeout, and `step.observed` emission. Separated
//! from the loop itself to keep `mod.rs` under the 300-line file cap.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::events::job::{StepObservedPayload, StepTrigger, StepVerdict, event_types};
use crate::events::{EventEnvelope, KernelRouter};
use crate::services::dispatch_ports::{JobExecutor, ProcessRegistryPort};

use super::StepMonitor;

/// Stop the dispatched process for `slug` if the registry still has a
/// handle. Emits `process.signal_sent` for observability; ignores stop
/// errors (best-effort — the step is already terminal from the
/// pipeline's view). `emit_and_finish` calls `registry.unregister`
/// after us, so we use `take` here to avoid a double-remove.
pub(super) async fn kill_spawned_process(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<dyn ProcessRegistryPort>,
    executor: &Arc<dyn JobExecutor>,
    slug: &str,
) {
    let Some(handle) = registry.take(slug).await else {
        return;
    };
    let pid = handle.pid;
    crate::services::orchestrator::emit::emit_process_signal_sent(
        kernel, slug, pid, "SIGTERM", "step_timeout", "step_monitor",
    ).await;
    if let Err(e) = executor.stop(&handle).await {
        tracing::warn!(slug, "step-monitor: stop on timeout returned err: {e}");
    }
}

/// Build the payload, emit `step.observed`, and unregister from the registry.
#[allow(clippy::too_many_arguments)]
pub(super) async fn emit_and_finish(
    monitor: &StepMonitor,
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<dyn ProcessRegistryPort>,
    trigger: StepTrigger,
    exit_code: Option<i32>,
    done_declared: bool,
    reason: Option<String>,
    commits: u32,
    uncommitted: u32,
    turns: u32,
) {
    registry.unregister(&monitor.slug).await;

    let done_result = if done_declared { Some(true) } else { None };
    let verdict = StepObservedPayload::compute_verdict(
        &monitor.step_type, done_declared, done_result,
        exit_code, commits, uncommitted, &trigger,
    );
    let duration_secs = monitor.started_at.elapsed().as_secs();
    let succeeded = verdict == StepVerdict::Advance;

    tracing::info!(
        slug = monitor.slug, step = monitor.step_name,
        ?trigger, ?verdict, done_declared, ?exit_code,
        commits, uncommitted, turns, ?reason,
        "step-monitor: emitting observation"
    );

    let payload = StepObservedPayload {
        job_id: monitor.job_id.clone(),
        slug: monitor.slug.clone(),
        step_index: monitor.step_index,
        step_name: monitor.step_name.clone(),
        trigger,
        done_declared,
        done_result,
        exit_code,
        commits_since_start: commits,
        uncommitted_files: uncommitted,
        duration_secs,
        reason,
        verdict,
        succeeded,
        start_sha: monitor.start_sha.clone(),
    };

    crate::services::orchestrator::emit::emit_process_exited(
        kernel, &monitor.slug, None, exit_code, None,
        duration_secs * 1000, "unknown",
    ).await;

    emit_step_observed(kernel, &monitor.job_id, &monitor.slug, &payload).await;
}

/// Scan git state in a worktree, counting commits since `start_sha`.
///
/// Returns `(commits_since_start, uncommitted_files)`. Falls back to
/// `(0, 0)` if git commands fail — never crashes.
pub(super) fn scan_git_evidence(
    work_dir: &std::path::Path,
    start_sha: Option<&str>,
) -> (u32, u32) {
    let commits = match start_sha {
        Some(sha) => std::process::Command::new("git")
            .args(["rev-list", "--count", &format!("{sha}..HEAD")])
            .current_dir(work_dir)
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
            .unwrap_or(0),
        None => std::process::Command::new("git")
            .args(["rev-list", "--count", "HEAD", "--not", "--remotes"])
            .current_dir(work_dir)
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
            .unwrap_or(0),
    };

    let uncommitted = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(work_dir)
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .count() as u32
        })
        .unwrap_or(0);

    (commits, uncommitted)
}

/// Resolve HEAD SHA in a worktree. Returns `None` if git fails.
pub(super) fn resolve_head_sha(work_dir: &std::path::Path) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(work_dir)
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
}

async fn emit_step_observed(
    kernel: &Arc<Mutex<KernelRouter>>,
    job_id: &str,
    slug: &str,
    payload: &StepObservedPayload,
) {
    let envelope = match EventEnvelope::new(event_types::STEP_OBSERVED, job_id, payload) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "step-monitor: failed to build envelope: {e}");
            return;
        }
    };

    crate::services::orchestrator::emit::persist_and_route(kernel, envelope, slug, None).await;
}
