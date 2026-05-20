//! Step monitor — evidence-based per-step execution tracker.
//!
//! Each dispatched step gets a `StepMonitor` that listens for multiple signals:
//! 1. `step.done_declared` event (agent called `step_done`) — strongest signal
//! 2. Activity signals (harness Stop hook, turn ended) — scan triggers
//! 3. Process exit signal (executor sends `ExitSignal`)
//! 4. Subprocess-tree heartbeat (CPU progress across root + descendants)
//! 5. Idle timeout: no activity for `idle_timeout` → kill
//! 6. Hard cap: absolute wall-clock safety net from step start
//!
//! The monitor accumulates evidence across signals. It does not advance
//! the pipeline on a single event — it waits for activity to settle and
//! checks outputs (git commits, step_done) before deciding.

mod evidence;

use ship_substrate::events::pipeline::{StepTrigger, StepType};
use ship_substrate::events::KernelRouter;
use crate::services::dispatch_ports::{
    ExitSignal, SessionExecutor, ProcessRegistryPort, ProcessTreeProbe, TreeSample,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

use super::StepActivity;

use evidence::{emit_and_finish, kill_spawned_process, resolve_head_sha, scan_git_evidence};

/// Primary kill signal: no subprocess CPU progress for this long → stuck.
pub(crate) const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Safety net: upper-bound from step start regardless of activity. Only
/// fires on pathological cases where an agent or its children keep CPU
/// hot forever without making pipeline progress.
pub(crate) const DEFAULT_HARD_CAP: Duration = Duration::from_secs(60 * 60);

/// How often the monitor re-samples the process tree. 60s is cheap and
/// still fine-grained enough for the 5-minute idle decision.
pub(crate) const DEFAULT_PROBE_INTERVAL: Duration = Duration::from_secs(60);

/// Emit an operator warn log once at this fraction of the idle budget.
const IDLE_WARN_FRACTION: f32 = 0.8;

/// After evidence appears (commits, turn ended), wait this long for more
/// activity before declaring the step done. Prevents advancing mid-task
/// when the agent is just between tool-call loops.
pub(crate) const DEFAULT_SETTLE_DURATION: Duration = Duration::from_secs(15);

/// Per-step execution monitor.
pub struct StepMonitor {
    pub job_id: String,
    pub step_index: usize,
    pub slug: String,
    pub step_name: String,
    /// Pipeline step kind — feeds the verdict rules. Run steps short-circuit
    /// on exit code; agent/review steps require explicit evidence.
    pub step_type: StepType,
    pub work_dir: PathBuf,
    pub started_at: Instant,
    pub idle_timeout: Duration,
    pub hard_cap: Duration,
    pub probe_interval: Duration,
    pub settle_duration: Duration,
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
        step_type: StepType,
        work_dir: PathBuf,
    ) -> Self {
        let start_sha = resolve_head_sha(&work_dir);
        Self {
            job_id,
            step_index,
            slug,
            step_name,
            step_type,
            work_dir,
            started_at: Instant::now(),
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
            hard_cap: DEFAULT_HARD_CAP,
            probe_interval: DEFAULT_PROBE_INTERVAL,
            settle_duration: DEFAULT_SETTLE_DURATION,
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
/// Listens on four inputs simultaneously: `exit_rx` (process death),
/// `activity_rx` (turn-ended), the subprocess-tree heartbeat probe, and
/// a combined (idle, hard-cap) deadline.
pub fn start_step_monitor(
    monitor: StepMonitor,
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<dyn ProcessRegistryPort>,
    executor: Arc<dyn SessionExecutor>,
    probe: Arc<dyn ProcessTreeProbe>,
    mut exit_rx: UnboundedReceiver<ExitSignal>,
    mut activity_rx: UnboundedReceiver<StepActivity>,
) {
    tokio::spawn(async move {
        let hard_deadline = tokio::time::Instant::from_std(monitor.started_at) + monitor.hard_cap;
        let mut probe_ticker = tokio::time::interval(monitor.probe_interval);
        probe_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        probe_ticker.tick().await;

        let mut last_activity_at = tokio::time::Instant::now();
        let mut last_sample: Option<TreeSample> = None;
        let mut idle_warn_fired = false;
        let mut settle_deadline: Option<tokio::time::Instant> = None;
        let mut exit_signal: Option<ExitSignal> = None;
        let mut turns: u32 = 0;

        loop {
            if monitor.done_declared.load(Ordering::Acquire) {
                let (commits, uncommitted) =
                    scan_git_evidence(&monitor.work_dir, monitor.start_sha.as_deref());
                emit_and_finish(
                    &monitor, &kernel, &registry, StepTrigger::DoneDeclared, None,
                    true, None, commits, uncommitted, turns,
                ).await;
                return;
            }

            if let Some(sd) = settle_deadline
                && tokio::time::Instant::now() >= sd
            {
                let (commits, uncommitted) =
                    scan_git_evidence(&monitor.work_dir, monitor.start_sha.as_deref());
                let trigger = if exit_signal.is_some() {
                    StepTrigger::ProcessExit
                } else {
                    StepTrigger::TurnSettled
                };
                let exit_code = exit_signal.as_ref().and_then(|s| s.exit_code);
                emit_and_finish(
                    &monitor, &kernel, &registry, trigger, exit_code,
                    false, None, commits, uncommitted, turns,
                ).await;
                return;
            }

            let idle_deadline = last_activity_at + monitor.idle_timeout;
            let mut next_deadline = hard_deadline.min(idle_deadline);
            if let Some(sd) = settle_deadline {
                next_deadline = next_deadline.min(sd);
            }

            tokio::select! {
                signal = exit_rx.recv() => {
                    match signal {
                        Some(sig) => {
                            tracing::info!(
                                slug = monitor.slug, exit_code = ?sig.exit_code,
                                "step-monitor: process exit signal"
                            );
                            exit_signal = Some(sig);
                            settle_deadline = Some(
                                tokio::time::Instant::now() + Duration::from_secs(3),
                            );
                        }
                        None => {
                            let (commits, uncommitted) =
                                scan_git_evidence(&monitor.work_dir, monitor.start_sha.as_deref());
                            emit_and_finish(
                                &monitor, &kernel, &registry, StepTrigger::ChannelClosed,
                                None, false, Some("exit channel closed".into()),
                                commits, uncommitted, turns,
                            ).await;
                            return;
                        }
                    }
                }
                activity = activity_rx.recv() => {
                    if let Some(StepActivity::TurnEnded) = activity {
                        turns += 1;
                        tracing::info!(
                            slug = monitor.slug, turns,
                            "step-monitor: turn ended (Stop hook), starting settle"
                        );
                        let now = tokio::time::Instant::now();
                        last_activity_at = now;
                        idle_warn_fired = false;
                        settle_deadline = Some(now + monitor.settle_duration);
                    }
                }
                _ = probe_ticker.tick() => {
                    if let Some(pid) = registry.get_pid(&monitor.slug).await
                        && let Some(sample) = probe.sample(pid).await
                    {
                        if let Some(prev) = last_sample
                            && sample.cpu_ticks > prev.cpu_ticks
                        {
                            let now = tokio::time::Instant::now();
                            tracing::debug!(
                                slug = monitor.slug,
                                delta = sample.cpu_ticks - prev.cpu_ticks,
                                procs = sample.process_count,
                                "step-monitor: subprocess heartbeat"
                            );
                            last_activity_at = now;
                            idle_warn_fired = false;
                            if settle_deadline.is_some() {
                                settle_deadline = Some(now + monitor.settle_duration);
                            }
                        }
                        last_sample = Some(sample);
                    }
                }
                _ = tokio::time::sleep_until(next_deadline) => {
                    let now = tokio::time::Instant::now();
                    // While a settle is active the step has already reached a
                    // terminal signal (exit, turn-ended, ...); idle timeout
                    // must not pre-empt the settle-then-emit path. Hard cap
                    // still applies as the ultimate safety net.
                    let idle_active = settle_deadline.is_none();
                    if now >= hard_deadline || (idle_active && now >= idle_deadline) {
                        let reason = if now >= hard_deadline { "hard_cap" } else { "idle_timeout" };
                        let (commits, uncommitted) =
                            scan_git_evidence(&monitor.work_dir, monitor.start_sha.as_deref());
                        kill_spawned_process(&kernel, &registry, &executor, &monitor.slug).await;
                        emit_and_finish(
                            &monitor, &kernel, &registry, StepTrigger::Timeout, None,
                            false, Some(reason.to_string()),
                            commits, uncommitted, turns,
                        ).await;
                        return;
                    }
                    if !idle_warn_fired {
                        let warn_at = last_activity_at
                            + Duration::from_secs_f32(
                                monitor.idle_timeout.as_secs_f32() * IDLE_WARN_FRACTION,
                            );
                        if now >= warn_at {
                            let remaining = idle_deadline.saturating_duration_since(now).as_secs();
                            tracing::warn!(
                                slug = monitor.slug, step = monitor.step_name,
                                remaining_secs = remaining,
                                "step-monitor: step approaching idle timeout"
                            );
                            idle_warn_fired = true;
                        }
                    }
                }
            }
        }
    });
}
