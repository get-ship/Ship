//! Step monitor — evidence-based per-step execution tracker.
//!
//! Each dispatched step gets a `StepMonitor` that listens for multiple signals:
//! 1. `step.done_declared` event (agent called `step_done`) — strongest signal
//! 2. Activity signals (provider Stop hook, turn ended) — scan triggers
//! 3. Process exit signal (executor sends `ExitSignal`)
//! 4. Timeout (configurable, default 30 minutes)
//!
//! The monitor accumulates evidence across signals. It does not advance
//! the pipeline on a single event — it waits for activity to settle and
//! checks outputs (git commits, step_done) before deciding.

use crate::events::job::{StepObservedPayload, StepTrigger, StepVerdict, event_types};
use crate::events::{EventEnvelope, KernelRouter};
use crate::services::dispatch_ports::{ExitSignal, ProcessRegistryPort};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

use super::StepActivity;

/// Default step timeout: 30 minutes.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// After evidence appears (commits, turn ended), wait this long for more
/// activity before declaring the step done. Prevents advancing mid-task
/// when the agent is just between tool-call loops.
const SETTLE_DURATION: Duration = Duration::from_secs(15);

/// Per-step execution monitor.
pub struct StepMonitor {
    pub job_id: String,
    pub step_index: usize,
    pub slug: String,
    pub step_name: String,
    pub work_dir: PathBuf,
    pub started_at: Instant,
    pub timeout: Duration,
    pub done_declared: Arc<AtomicBool>,
    /// HEAD SHA at step start — baseline for counting this step's commits.
    pub start_sha: Option<String>,
}

impl StepMonitor {
    pub fn new(
        job_id: String,
        step_index: usize,
        slug: String,
        step_name: String,
        work_dir: PathBuf,
    ) -> Self {
        let start_sha = resolve_head_sha(&work_dir);
        Self {
            job_id,
            step_index,
            slug,
            step_name,
            work_dir,
            started_at: Instant::now(),
            timeout: DEFAULT_TIMEOUT,
            done_declared: Arc::new(AtomicBool::new(false)),
            start_sha,
        }
    }

    /// Returns a clone of the `done_declared` flag for external signaling.
    pub fn done_flag(&self) -> Arc<AtomicBool> {
        self.done_declared.clone()
    }
}

/// Spawn the step monitor task.
///
/// Listens on three channels simultaneously:
/// - `exit_rx`: process/session death
/// - `activity_rx`: turn-ended signals from Stop hook
/// - timeout: global step timeout
///
/// On each signal, gathers git evidence and decides whether to advance.
pub fn start_step_monitor(
    monitor: StepMonitor,
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<dyn ProcessRegistryPort>,
    mut exit_rx: UnboundedReceiver<ExitSignal>,
    mut activity_rx: UnboundedReceiver<StepActivity>,
) {
    tokio::spawn(async move {
        let deadline = tokio::time::Instant::now() + monitor.timeout;
        let mut settle_deadline: Option<tokio::time::Instant> = None;
        let mut exit_signal: Option<ExitSignal> = None;
        let mut turns: u32 = 0;

        loop {
            let done = monitor.done_declared.load(Ordering::Acquire);

            // If step_done was called, emit immediately.
            if done {
                let (commits, uncommitted) = scan_git_evidence(
                    &monitor.work_dir, monitor.start_sha.as_deref(),
                );
                emit_and_finish(
                    &monitor, &kernel, &registry,
                    StepTrigger::DoneDeclared, None,
                    true, None, commits, uncommitted, turns,
                ).await;
                return;
            }

            // If we're settling and the deadline passed, emit.
            if let Some(sd) = settle_deadline {
                if tokio::time::Instant::now() >= sd {
                    let (commits, uncommitted) = scan_git_evidence(
                        &monitor.work_dir, monitor.start_sha.as_deref(),
                    );
                    let trigger = if exit_signal.is_some() {
                        StepTrigger::ProcessExit
                    } else {
                        StepTrigger::TurnSettled
                    };
                    let exit_code = exit_signal.as_ref().and_then(|s| s.exit_code);
                    emit_and_finish(
                        &monitor, &kernel, &registry,
                        trigger, exit_code,
                        false, None, commits, uncommitted, turns,
                    ).await;
                    return;
                }
            }

            // Compute how long until the next interesting deadline.
            let sleep_until = match settle_deadline {
                Some(sd) => sd.min(deadline),
                None => deadline,
            };

            tokio::select! {
                signal = exit_rx.recv() => {
                    match signal {
                        Some(sig) => {
                            tracing::info!(
                                slug = monitor.slug,
                                exit_code = ?sig.exit_code,
                                "step-monitor: process exit signal"
                            );
                            exit_signal = Some(sig);
                            // Process died — scan immediately, but use settle
                            // to avoid racing with a near-simultaneous step_done.
                            settle_deadline = Some(
                                tokio::time::Instant::now() + Duration::from_secs(3)
                            );
                        }
                        None => {
                            // Channel closed — emit with what we have.
                            let (commits, uncommitted) = scan_git_evidence(
                                &monitor.work_dir, monitor.start_sha.as_deref(),
                            );
                            emit_and_finish(
                                &monitor, &kernel, &registry,
                                StepTrigger::ChannelClosed, None,
                                false, Some("exit channel closed".into()),
                                commits, uncommitted, turns,
                            ).await;
                            return;
                        }
                    }
                }
                activity = activity_rx.recv() => {
                    match activity {
                        Some(StepActivity::TurnEnded) => {
                            turns += 1;
                            tracing::info!(
                                slug = monitor.slug, turns,
                                "step-monitor: turn ended (Stop hook)"
                            );
                            // Check for evidence. If we see commits, start
                            // the settle timer. If not, just note the turn.
                            let (commits, _) = scan_git_evidence(
                                &monitor.work_dir, monitor.start_sha.as_deref(),
                            );
                            if commits > 0 || monitor.done_declared.load(Ordering::Acquire) {
                                settle_deadline = Some(
                                    tokio::time::Instant::now() + SETTLE_DURATION
                                );
                            }
                            // If no commits yet, don't start settling —
                            // the agent may still be working.
                        }
                        None => {
                            // Activity channel closed — continue waiting
                            // on exit_rx and timeout.
                        }
                    }
                }
                _ = tokio::time::sleep_until(sleep_until) => {
                    // Either settle deadline or global timeout fired.
                    // The loop body above handles both cases.
                    if tokio::time::Instant::now() >= deadline {
                        let (commits, uncommitted) = scan_git_evidence(
                            &monitor.work_dir, monitor.start_sha.as_deref(),
                        );
                        emit_and_finish(
                            &monitor, &kernel, &registry,
                            StepTrigger::Timeout, None,
                            false, Some("timeout".into()),
                            commits, uncommitted, turns,
                        ).await;
                        return;
                    }
                }
            }
        }
    });
}

/// Build the payload, emit step.observed, and unregister from the registry.
async fn emit_and_finish(
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
        done_declared, done_result, exit_code, commits, uncommitted, &trigger,
    );
    let duration_secs = monitor.started_at.elapsed().as_secs();
    let succeeded = verdict == StepVerdict::Advance;

    tracing::info!(
        slug = monitor.slug,
        step = monitor.step_name,
        ?trigger,
        ?verdict,
        done_declared,
        ?exit_code,
        commits,
        uncommitted,
        turns,
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
    };

    emit_step_observed(kernel, &monitor.job_id, &monitor.slug, &payload).await;
}

/// Scan git state in a worktree, counting commits since `start_sha`.
///
/// Returns (commits_since_start, uncommitted_files).
/// Falls back to (0, 0) if git commands fail — never crashes.
fn scan_git_evidence(work_dir: &std::path::Path, start_sha: Option<&str>) -> (u32, u32) {
    let commits = match start_sha {
        Some(sha) => {
            std::process::Command::new("git")
                .args(["rev-list", "--count", &format!("{sha}..HEAD")])
                .current_dir(work_dir)
                .output()
                .ok()
                .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
                .unwrap_or(0)
        }
        None => {
            // No baseline — count all local-only commits as fallback.
            std::process::Command::new("git")
                .args(["rev-list", "--count", "HEAD", "--not", "--remotes"])
                .current_dir(work_dir)
                .output()
                .ok()
                .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
                .unwrap_or(0)
        }
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

/// Resolve HEAD SHA in a worktree. Returns None if git fails.
fn resolve_head_sha(work_dir: &std::path::Path) -> Option<String> {
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

/// Emit a `step.observed` event through the kernel.
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

    crate::services::orchestrator::emit::persist_and_route(
        kernel, envelope, slug, None,
    ).await;
}
