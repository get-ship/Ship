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
//!
//! **Journaled (C5 / P2.3):** the step's lifecycle runs as `AgentStepProgram`
//! through the durable-execution engine. At monitor start the program parks
//! on the `agent.running` suspend (journal saved under
//! `<pipeline_run_id>/agent-step/<step_name>`); the monitor is now the
//! EVIDENCE FEEDER — when its timing machinery decides the step is over, it
//! resolves the suspend with a [`StepEvidence`] and the program's
//! `agent.verdict` record + `agent.emit_observed` op produce the verdict and
//! the `pipeline.step.observed` emission, exactly once under replay. A step
//! whose journal is already complete (re-dispatch after the observation)
//! exits without re-emitting. The timers themselves move to the coordinator
//! at C7/P2.5, deleting this loop.

mod evidence;

use durable_execution::{Dispatch, Journal, JournalStore, RunStatus};
use ship_substrate::events::pipeline::{StepTrigger, StepType};
use ship_substrate::events::KernelRouter;
use ship_code::dispatch_ports::{
    ExitSignal, SessionExecutor, ProcessRegistryPort, ProcessTreeProbe, TreeSample,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

use super::StepActivity;
use super::coordinator_programs::{AgentStepProgram, StepEvidence};

use evidence::{AgentStepDispatch, kill_spawned_process};
use ship_code::runner::observe::{resolve_head_sha, scan_git_evidence};

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
    pub pipeline_run_id: String,
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
        pipeline_run_id: String,
        step_index: usize,
        slug: String,
        step_name: String,
        step_type: StepType,
        work_dir: PathBuf,
    ) -> Self {
        let start_sha = resolve_head_sha(&work_dir);
        Self {
            pipeline_run_id,
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

/// The journal key for one agent step of one run. Distinct namespaces:
/// the bare `pipeline_run_id` is the `DispatchProgram` journal and
/// `<run_id>/step/<id>` is the `RunStepProgram` journal — different op
/// sequences must never share a key (replay-divergence guard).
fn agent_step_journal_key(pipeline_run_id: &str, step_name: &str) -> String {
    format!("{pipeline_run_id}/agent-step/{step_name}")
}

/// Resolve the parked `agent.running` suspend with the monitor's evidence —
/// the program then records the verdict and emits `pipeline.step.observed`
/// through its journaled ops.
async fn resolve_with_evidence(
    journal_store: &Arc<dyn JournalStore>,
    key: &str,
    journal: Journal,
    dispatch: Arc<dyn Dispatch>,
    program: &AgentStepProgram,
    evidence: StepEvidence,
    slug: &str,
) {
    let decision = match serde_json::to_value(&evidence) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(slug, "step-monitor: evidence serialization failed: {e}");
            return;
        }
    };
    // NOTE (at-least-once edge): the observed emission happens INSIDE resume,
    // the save AFTER it. A daemon crash between the two re-emits the
    // observation once on recovery — accepted until C7 makes the coordinator
    // own the emit/save ordering. Replay-dedup covers every other path.
    let (status, journal) =
        durable_execution::resume(journal, decision, dispatch, program).await;
    if let Err(e) = journal_store.save(key, &journal).await {
        tracing::warn!(slug, "step-monitor: journal save failed (non-fatal): {e}");
    }
    if let RunStatus::Failed(e) = status {
        tracing::error!(slug, "step-monitor: engine failure resolving step: {e}");
    }
}

/// Spawn the step monitor task.
///
/// Listens on four inputs simultaneously: `exit_rx` (process death),
/// `activity_rx` (turn-ended), the subprocess-tree heartbeat probe, and
/// a combined (idle, hard-cap) deadline.
#[allow(clippy::too_many_arguments)]
pub fn start_step_monitor(
    monitor: StepMonitor,
    kernel: Arc<Mutex<KernelRouter>>,
    registry: Arc<dyn ProcessRegistryPort>,
    executor: Arc<dyn SessionExecutor>,
    probe: Arc<dyn ProcessTreeProbe>,
    mut exit_rx: UnboundedReceiver<ExitSignal>,
    mut activity_rx: UnboundedReceiver<StepActivity>,
    journal_store: Arc<dyn JournalStore>,
) {
    tokio::spawn(async move {
        // ── Open the step's program journal ──────────────────────────────
        // Fresh step: runs the (bridge-echo) spawn op and parks on the
        // `agent.running` suspend. Already-observed step (re-dispatch after a
        // restart): the journal replays to Completed — exit WITHOUT starting
        // the loop, so the observation is never emitted twice.
        let program = AgentStepProgram { step_type: monitor.step_type.clone() };
        let dispatch: Arc<dyn Dispatch> = Arc::new(AgentStepDispatch {
            kernel: kernel.clone(),
            registry: registry.clone(),
            pipeline_run_id: monitor.pipeline_run_id.clone(),
            slug: monitor.slug.clone(),
            step_index: monitor.step_index,
            step_name: monitor.step_name.clone(),
            work_dir: monitor.work_dir.clone(),
            started_at: monitor.started_at,
            start_sha: monitor.start_sha.clone(),
        });
        let key = agent_step_journal_key(&monitor.pipeline_run_id, &monitor.step_name);
        let journal = match journal_store.load(&key).await {
            Ok(Some(j)) => j,
            Ok(None) => Journal::new(),
            Err(e) => {
                tracing::warn!(
                    slug = monitor.slug,
                    "step-monitor: journal load failed, starting fresh: {e}"
                );
                Journal::new()
            }
        };
        let (status, journal) =
            durable_execution::execute(journal, dispatch.clone(), &program).await;
        if let Err(e) = journal_store.save(&key, &journal).await {
            tracing::warn!(slug = monitor.slug, "step-monitor: journal save failed (non-fatal): {e}");
        }
        match status {
            RunStatus::Suspended => {} // parked at agent.running — the normal path
            RunStatus::Completed(_) => {
                tracing::info!(
                    slug = monitor.slug, step = monitor.step_name,
                    "step-monitor: step already observed (journal complete) — monitor not started"
                );
                return;
            }
            RunStatus::Failed(e) => {
                tracing::error!(
                    slug = monitor.slug,
                    "step-monitor: engine failure opening step journal: {e}"
                );
                return;
            }
        }
        let mut journal = Some(journal);

        // ── Timing machinery (the C5 bridge's remaining job) ──────────────
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
                resolve_with_evidence(
                    &journal_store, &key, journal.take().expect("journal present"),
                    dispatch.clone(), &program,
                    StepEvidence {
                        trigger: StepTrigger::DoneDeclared, exit_code: None,
                        done_declared: true, reason: None,
                        commits, uncommitted, turns,
                    },
                    &monitor.slug,
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
                resolve_with_evidence(
                    &journal_store, &key, journal.take().expect("journal present"),
                    dispatch.clone(), &program,
                    StepEvidence {
                        trigger, exit_code, done_declared: false, reason: None,
                        commits, uncommitted, turns,
                    },
                    &monitor.slug,
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
                            resolve_with_evidence(
                                &journal_store, &key, journal.take().expect("journal present"),
                                dispatch.clone(), &program,
                                StepEvidence {
                                    trigger: StepTrigger::ChannelClosed, exit_code: None,
                                    done_declared: false,
                                    reason: Some("exit channel closed".into()),
                                    commits, uncommitted, turns,
                                },
                                &monitor.slug,
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
                        resolve_with_evidence(
                            &journal_store, &key, journal.take().expect("journal present"),
                            dispatch.clone(), &program,
                            StepEvidence {
                                trigger: StepTrigger::Timeout, exit_code: None,
                                done_declared: false, reason: Some(reason.to_string()),
                                commits, uncommitted, turns,
                            },
                            &monitor.slug,
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
