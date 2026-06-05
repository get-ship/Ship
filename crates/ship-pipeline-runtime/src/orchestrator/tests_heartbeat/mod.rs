//! Tests for the step monitor's subprocess-tree heartbeat.
//!
//! We use short real-time durations rather than virtual time because
//! `spawn_blocking` in the event store requires a multi-threaded runtime
//! and `start_paused` requires `current_thread`.

mod fakes;

use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use ship_substrate::events::pipeline::{
    event_types as run_event_types, PipelineStepObservedPayload, StepTrigger, StepType,
};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_substrate::services::dispatch_ports::{ExitSignal, ProcessTreeProbe};
use crate::orchestrator::step_monitor::{StepMonitor, start_step_monitor};
use crate::orchestrator::StepActivity;

use fakes::{AdvancingProbe, IdleProbe, NoopExecutor, NullProbe, StubRegistry, ToggleProbe};

// ── Harness ────────────────────────────────────────────────────────────────

fn dispatch_actor_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.orchestrator".into(),
        write_namespaces: vec!["job.".into(), "step.".into(), "process.".into()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["pipeline.".into()],
        see_directed_events: false,
    }
}

async fn fresh_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router
        .spawn_actor("service.orchestrator", dispatch_actor_config())
        .unwrap();
    Arc::new(Mutex::new(router))
}

struct Cfg {
    slug: &'static str,
    idle_timeout: Duration,
    hard_cap: Duration,
    probe_interval: Duration,
    settle_duration: Duration,
}

impl Cfg {
    fn fast(slug: &'static str) -> Self {
        Self {
            slug,
            idle_timeout: Duration::from_millis(400),
            hard_cap: Duration::from_secs(30),
            probe_interval: Duration::from_millis(50),
            settle_duration: Duration::from_millis(100),
        }
    }
}

struct Launch {
    pipeline_run_id: String,
    exit_tx: mpsc::UnboundedSender<ExitSignal>,
    activity_tx: mpsc::UnboundedSender<StepActivity>,
}

fn launch(
    kernel: Arc<Mutex<KernelRouter>>,
    work_dir: std::path::PathBuf,
    cfg: Cfg,
    probe: Arc<dyn ProcessTreeProbe>,
) -> Launch {
    let mut monitor = StepMonitor::new(
        format!("j-{}", cfg.slug),
        0,
        cfg.slug.to_string(),
        "heartbeat-test".into(),
        StepType::Agent,
        work_dir,
    );
    monitor.idle_timeout = cfg.idle_timeout;
    monitor.hard_cap = cfg.hard_cap;
    monitor.probe_interval = cfg.probe_interval;
    monitor.settle_duration = cfg.settle_duration;
    let pipeline_run_id = monitor.pipeline_run_id.clone();

    let (exit_tx, exit_rx) = mpsc::unbounded_channel::<ExitSignal>();
    let (activity_tx, activity_rx) = mpsc::unbounded_channel::<StepActivity>();
    start_step_monitor(
        monitor,
        kernel,
        Arc::new(StubRegistry { pid: Some(std::process::id()) }),
        Arc::new(NoopExecutor),
        probe,
        exit_rx,
        activity_rx,
    );
    Launch { pipeline_run_id, exit_tx, activity_tx }
}

async fn wait_observed(pipeline_run_id: &str, deadline: Duration) -> Option<PipelineStepObservedPayload> {
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        if let Some(p) = find_observed(pipeline_run_id) {
            return Some(p);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    None
}

fn find_observed(pipeline_run_id: &str) -> Option<PipelineStepObservedPayload> {
    ship_substrate::db::events::list_all_events()
        .ok()?
        .into_iter()
        .rev()
        .find_map(|e| {
            if e.entity_id == pipeline_run_id && e.event_type == run_event_types::PIPELINE_STEP_OBSERVED {
                serde_json::from_str(&e.payload_json).ok()
            } else {
                None
            }
        })
}

// ── Tests ──────────────────────────────────────────────────────────────────

/// Idle tree → monitor kills at `idle_timeout` with `reason="idle_timeout"`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn idle_subprocess_kills_with_idle_timeout_reason() {
    let tmp = TempDir::new().unwrap();
    let kernel = fresh_kernel().await;
    let l = launch(
        kernel,
        tmp.path().to_path_buf(),
        Cfg::fast("heartbeat-idle"),
        Arc::new(IdleProbe),
    );
    let observed = wait_observed(&l.pipeline_run_id, Duration::from_secs(3))
        .await
        .expect("idle monitor must emit step.observed");
    assert_eq!(observed.trigger, StepTrigger::Timeout);
    assert_eq!(observed.reason.as_deref(), Some("idle_timeout"));
}

/// Runaway tree → monitor kills at `hard_cap` even with continuous CPU.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runaway_subprocess_kills_with_hard_cap_reason() {
    let tmp = TempDir::new().unwrap();
    let kernel = fresh_kernel().await;
    let cfg = Cfg {
        slug: "heartbeat-runaway",
        idle_timeout: Duration::from_secs(30),
        hard_cap: Duration::from_millis(400),
        probe_interval: Duration::from_millis(50),
        settle_duration: Duration::from_millis(100),
    };
    let l = launch(kernel, tmp.path().to_path_buf(), cfg, Arc::new(AdvancingProbe::new()));
    let observed = wait_observed(&l.pipeline_run_id, Duration::from_secs(3))
        .await
        .expect("runaway monitor must emit step.observed");
    assert_eq!(observed.trigger, StepTrigger::Timeout);
    assert_eq!(observed.reason.as_deref(), Some("hard_cap"));
}

/// Active CPU resets the idle timer → monitor survives well past the
/// idle budget. Guards the "cargo test inside a tool call" regression.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn active_subprocess_keeps_step_alive_past_idle_budget() {
    let tmp = TempDir::new().unwrap();
    let kernel = fresh_kernel().await;
    let cfg = Cfg {
        slug: "heartbeat-active",
        idle_timeout: Duration::from_millis(200),
        ..Cfg::fast("heartbeat-active")
    };
    let (probe, freeze) = ToggleProbe::new();
    let l = launch(kernel, tmp.path().to_path_buf(), cfg, Arc::new(probe));

    // 4x the idle budget — active CPU must have kept the monitor alive.
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert!(find_observed(&l.pipeline_run_id).is_none());

    // Freeze (tree "died") and send exit; settle fires, emit ProcessExit.
    freeze.store(true, std::sync::atomic::Ordering::Relaxed);
    l.exit_tx
        .send(ExitSignal {
            slug: "heartbeat-active".into(),
            pipeline_run_id: l.pipeline_run_id.clone(),
            exit_code: Some(0),
            cause: ship_substrate::services::dispatch_ports::ExitCause::AgentExited,
        })
        .unwrap();
    let observed = wait_observed(&l.pipeline_run_id, Duration::from_secs(6))
        .await
        .expect("exit signal must produce a step.observed");
    assert_eq!(observed.trigger, StepTrigger::ProcessExit);
    assert_eq!(observed.reason, None);
}

/// Probe returning `None` must not be read as activity — idle still fires.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn probe_none_does_not_spuriously_reset_idle_timer() {
    let tmp = TempDir::new().unwrap();
    let kernel = fresh_kernel().await;
    let l = launch(
        kernel,
        tmp.path().to_path_buf(),
        Cfg::fast("heartbeat-none"),
        Arc::new(NullProbe),
    );
    let observed = wait_observed(&l.pipeline_run_id, Duration::from_secs(3))
        .await
        .expect("null-probe path still falls through to idle timeout");
    assert_eq!(observed.trigger, StepTrigger::Timeout);
    assert_eq!(observed.reason.as_deref(), Some("idle_timeout"));
}

/// Turn-ended opens settle, but active CPU keeps extending it. Guards
/// advancing the pipeline while `cargo test` is still running post-turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn turn_end_plus_active_cpu_extends_settle() {
    let tmp = TempDir::new().unwrap();
    let kernel = fresh_kernel().await;
    let cfg = Cfg {
        slug: "heartbeat-settle",
        idle_timeout: Duration::from_secs(10),
        hard_cap: Duration::from_secs(30),
        probe_interval: Duration::from_millis(50),
        settle_duration: Duration::from_millis(200),
    };
    let l = launch(kernel, tmp.path().to_path_buf(), cfg, Arc::new(AdvancingProbe::new()));

    l.activity_tx.send(StepActivity::TurnEnded).unwrap();
    // 4x the settle budget; CPU keeps climbing every 50ms → settle resets.
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert!(
        find_observed(&l.pipeline_run_id).is_none(),
        "settle must be extended while CPU is still climbing"
    );
}
