//! Behavior tests for the carved agent-session domain.
//!
//! These follow the units down from the orchestrator: the bare dispatch path
//! (`handle_dispatch_requested`), the no-double-spawn skip for composed
//! sessions, the spawn-env port wiring, and the agent-cancel decision
//! (`handle_agent_cancelled`). All run against the public `ship-agents` surface
//! with port test-doubles — no pipeline-DAG state, no `crate::projections`.
//!
//! Gated behind `ship-code`: these build ship_code harness port test-doubles
//! (incl. proving a floki session SKIPS the harness executor). With the feature off
//! there is no harness to test against, and `cargo test -p ship-agents` in isolation
//! must compile without ship_code. The floki routing path itself is ungated code;
//! the workspace `--all-targets` build (feature unified on via shipd) runs these.
#![cfg(feature = "ship-code")]

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

use agent_def::canonical_session::{SessionCancelled, event_types as canonical_session_event_types};
use agent_def::dispatch_intent::{DispatchRequestedPayload, event_types as dispatch_event_types};
use ship_agents::services::sessions::{
    CanonicalSessionProjection, CanonicalSessionStatus, load_canonical_session_by_id,
};
use ship_substrate::projections::Projection;
use ship_substrate::db::block_on;
use ship_agents::ports::AgentPorts;
use ship_substrate::dispatch_gate::EventDedup;
use agent_def::InvocationMode;
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_substrate::events::process::event_types as process_event_types;
use ship_substrate::events::process::ProcessKilledPayload;
use ship_substrate::events::EventEnvelope;
use ship_code::dispatch_ports::{
    AgentPlatform, Authority, ExecutorHandle, IsolationStrategy, ProcessRegistryPort,
    ProjectionReader, SessionContext, SessionExecutor, SpawnPolicy,
};

// ── Port doubles ────────────────────────────────────────────────────────────

struct OkIsolation(PathBuf);

#[async_trait]
impl IsolationStrategy for OkIsolation {
    async fn prepare(&self, _: &SessionContext) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.0).ok();
        Ok(self.0.clone())
    }
    async fn cleanup(&self, _: &SessionContext) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Records the SessionContext it was asked to spawn, then errors so dispatch
/// skips post-spawn work (it still emits session.failed, but the test
/// asserts on the captured ctx, not on the spawn succeeding).
#[derive(Clone, Default)]
struct Captured {
    harness: Option<String>,
    model: Option<String>,
    branch: String,
    env: HashMap<String, String>,
    spec_path: String,
    autostart_prompt: Option<String>,
}

struct RecordingExecutor {
    captured: Arc<StdMutex<Option<Captured>>>,
    spawn_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SessionExecutor for RecordingExecutor {
    async fn spawn(&self, ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        self.spawn_calls.fetch_add(1, Ordering::SeqCst);
        *self.captured.lock().unwrap() = Some(Captured {
            harness: ctx.launch.harness.clone(),
            model: ctx.launch.model.clone(),
            branch: ctx.branch.clone(),
            env: ctx.env.clone(),
            spec_path: ctx.spec_path.clone(),
            autostart_prompt: ctx.launch.autostart_prompt.clone(),
        });
        Err(anyhow::anyhow!("recording executor: refusing to spawn"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool {
        true
    }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> {
        Ok(())
    }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Executor that records stop() calls — used by the cancel test.
struct StoppingExecutor {
    stop_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SessionExecutor for StoppingExecutor {
    async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("not used"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool {
        true
    }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> {
        self.stop_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

struct TestPlatform {
    worktrees: PathBuf,
    cleanup_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl AgentPlatform for TestPlatform {
    fn worktrees_dir(&self) -> anyhow::Result<PathBuf> {
        Ok(self.worktrees.clone())
    }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {
        self.cleanup_calls.fetch_add(1, Ordering::SeqCst);
    }
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> {
        Ok(true)
    }
    async fn cleanup_orphaned_worktrees(&self) {}
}

struct NoopRegistry;

#[async_trait]
impl ProcessRegistryPort for NoopRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
}

/// Registry seeded with one (slug → handle) entry. `take` drains it once.
struct SeededRegistry {
    handle: Mutex<Option<ExecutorHandle>>,
}

#[async_trait]
impl ProcessRegistryPort for SeededRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
    async fn take(&self, _: &str) -> Option<ExecutorHandle> {
        self.handle.lock().await.take()
    }
}

/// SpawnPolicy double that injects a sentinel token so the test can prove the
/// dispatch path threads the `Authority::Agent` principal through the port.
struct TokenSpawnPolicy;

#[async_trait]
impl SpawnPolicy for TokenSpawnPolicy {
    async fn spawn_env(&self, principal: &Authority, actor_id: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), actor_id.to_string());
        if let Authority::Agent { id } = principal {
            env.insert("SHIP_AGENT_PRINCIPAL".to_string(), id.clone());
        }
        env
    }
}

/// ProjectionReader double — nothing is terminal, so the gate always admits.
/// slug_for_session_id returns None (default impl) — not valid for cancel tests.
struct NeverTerminal;

#[async_trait]
impl ProjectionReader for NeverTerminal {
    async fn known_dispatch_slugs(&self) -> Vec<String> {
        vec![]
    }
    async fn is_run_terminal(&self, _: &str) -> bool {
        false
    }
}

/// ProjectionReader double for cancel tests — always returns a fixed slug so
/// the cancel handler can resolve slug from session_id without a real projection.
struct FixedSlugProjection {
    slug: String,
}

#[async_trait]
impl ProjectionReader for FixedSlugProjection {
    async fn known_dispatch_slugs(&self) -> Vec<String> {
        vec![]
    }
    async fn is_run_terminal(&self, _: &str) -> bool {
        false
    }
    async fn slug_for_session_id(&self, _session_id: &str) -> Option<String> {
        Some(self.slug.clone())
    }
}

// ── Harness ─────────────────────────────────────────────────────────────────

fn agents_actor_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.agents".to_string(),
        write_namespaces: vec!["process.".to_string(), "session.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["dispatch.".to_string(), "session.".to_string()],
        see_directed_events: false,
    }
}

fn started_payload(
    session_id: &str,
    slug: &str,
    harness: Option<&str>,
    model: Option<&str>,
) -> DispatchRequestedPayload {
    DispatchRequestedPayload {
        session_id: session_id.into(),
        slug: slug.into(),
        agent_id: "rust-lane".into(),
        harness: harness.map(str::to_string),
        model: model.map(str::to_string),
        branch: format!("job/{slug}"),
        // Test fixture — no per-call fork source.
        repo_path: None,
        base_ref: None,
        source_spec_path: None,
        baseline_sha: None,
        open_tab: false,
        invocation: InvocationMode::Print,
        keepalive: false,
        prompt: None,
        spec: None,
        resume: false,
        resume_session_id: None,
        depends_on: None,
        vars: None,
    }
}

fn ports_with(
    isolation: Arc<dyn IsolationStrategy>,
    executor: Arc<dyn SessionExecutor>,
    registry: Arc<dyn ProcessRegistryPort>,
    platform: Arc<dyn AgentPlatform>,
) -> AgentPorts {
    AgentPorts {
        floki_run_fn: None,
        isolation,
        executor,
        platform,
        registry,
        spawn_policy: Arc::new(TokenSpawnPolicy),
        projections: Arc::new(NeverTerminal),
        event_dedup: Arc::new(EventDedup::new()),
    }
}

fn ports_with_projection(
    isolation: Arc<dyn IsolationStrategy>,
    executor: Arc<dyn SessionExecutor>,
    registry: Arc<dyn ProcessRegistryPort>,
    platform: Arc<dyn AgentPlatform>,
    projections: Arc<dyn ProjectionReader>,
) -> AgentPorts {
    AgentPorts {
        floki_run_fn: None,
        isolation,
        executor,
        platform,
        registry,
        spawn_policy: Arc::new(TokenSpawnPolicy),
        projections,
        event_dedup: Arc::new(EventDedup::new()),
    }
}

fn kernel_with_actor() -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router
        .spawn_actor("service.agents", agents_actor_config())
        .unwrap();
    Arc::new(Mutex::new(router))
}

/// Check for a `process.spawned` event keyed on slug (entity_id == slug).
fn find_process_spawned(slug: &str) -> bool {
    ship_substrate::db::events::list_all_events()
        .ok()
        .map(|events| {
            events.iter().any(|e| {
                e.entity_id == slug
                    && e.event_type == process_event_types::PROCESS_SPAWNED
            })
        })
        .unwrap_or(false)
}

fn find_process_killed(slug: &str) -> Option<ProcessKilledPayload> {
    ship_substrate::db::events::list_all_events()
        .ok()?
        .into_iter()
        .find_map(|e| {
            if e.entity_id == slug && e.event_type == process_event_types::PROCESS_KILLED {
                serde_json::from_str(&e.payload_json).ok()
            } else {
                None
            }
        })
}

/// Run handle_agents_event on a dispatch.requested envelope and return the
/// captured spawn context. handle_dispatch_requested detaches the prep onto a
/// task, so we give it a moment to run before reading the capture.
async fn dispatch_and_capture(
    kernel: &Arc<Mutex<KernelRouter>>,
    ports: &AgentPorts,
    captured: &Arc<StdMutex<Option<Captured>>>,
    payload: &DispatchRequestedPayload,
) -> Option<Captured> {
    let envelope =
        EventEnvelope::new(dispatch_event_types::DISPATCH_REQUESTED, &payload.session_id, payload)
            .unwrap()
            .with_actor_id("test");
    ship_agents::handle_agents_event(kernel, &envelope, ports).await;
    // The detached prep task runs under DISPATCH_GATE; poll briefly.
    for _ in 0..50 {
        if captured.lock().unwrap().is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    captured.lock().unwrap().clone()
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bare_session_dispatch_threads_harness_model_branch_and_spawn_env() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kernel = kernel_with_actor();
    let captured = Arc::new(StdMutex::new(None));
    let spawn_calls = Arc::new(AtomicUsize::new(0));
    let ports = ports_with(
        Arc::new(OkIsolation(tmp.path().join("worktrees/sess-1"))),
        Arc::new(RecordingExecutor {
            captured: captured.clone(),
            spawn_calls: spawn_calls.clone(),
        }),
        Arc::new(NoopRegistry),
        Arc::new(TestPlatform {
            worktrees: tmp.path().join("worktrees"),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
        }),
    );

    let payload = started_payload("s-bare-1", "sess-1", Some("codex"), Some("gpt-5-codex"));
    let got = dispatch_and_capture(&kernel, &ports, &captured, &payload)
        .await
        .expect("executor.spawn called");

    assert_eq!(got.harness.as_deref(), Some("codex"), "harness threads into LaunchProfile");
    assert_eq!(got.model.as_deref(), Some("gpt-5-codex"), "model threads into LaunchProfile");
    assert_eq!(got.branch, "job/sess-1", "branch threads from payload");
    assert_eq!(
        got.spec_path, "",
        "no source_spec_path on the payload → no spec pointer for the executor"
    );
    // The Authority::Agent principal must reach the SpawnPolicy port verbatim.
    assert_eq!(
        got.env.get("SHIP_AGENT_PRINCIPAL").map(String::as_str),
        Some("rust-lane"),
        "Authority::Agent principal flows through build_spawn_env into the port"
    );
    assert_eq!(
        got.env.get("SHIP_ACTOR_ID").map(String::as_str),
        Some("sess-1"),
        "target slug threads as the actor id"
    );
}

/// SPEC (c-2026-07-05T12-36-00-000Z-b2c3d4e5, live-path trace): the dispatch
/// wire carries the spec body as `prompt` and its origin file as
/// `source_spec_path`. Both MUST reach the executor — `prompt` as
/// `launch.autostart_prompt` (the content channel for headless/argv
/// transports) and `source_spec_path` as `SessionContext.spec_path` (the
/// pointer channel for interactive TUI panes). The bug: the handler built
/// the LaunchProfile with `..Default::default()`, dropping the prompt, so
/// the tmux executor had nothing to send but the adapter's static "go".
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bare_session_dispatch_threads_prompt_and_spec_source_to_executor() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kernel = kernel_with_actor();
    let captured = Arc::new(StdMutex::new(None));
    let spawn_calls = Arc::new(AtomicUsize::new(0));
    let ports = ports_with(
        Arc::new(OkIsolation(tmp.path().join("worktrees/sess-p1"))),
        Arc::new(RecordingExecutor {
            captured: captured.clone(),
            spawn_calls: spawn_calls.clone(),
        }),
        Arc::new(NoopRegistry),
        Arc::new(TestPlatform {
            worktrees: tmp.path().join("worktrees"),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
        }),
    );

    let spec_body = "# Job Spec\n\nDo the real work.\nThen commit.";
    let mut payload = started_payload("s-prompt-1", "sess-p1", Some("omp"), None);
    payload.prompt = Some(spec_body.to_string());
    payload.spec = Some(spec_body.to_string());
    payload.source_spec_path =
        Some("/home/op/dev/Ship/helm/data/missions/m-x/scratch/spec.md".to_string());

    let got = dispatch_and_capture(&kernel, &ports, &captured, &payload)
        .await
        .expect("executor.spawn called");

    assert_eq!(
        got.autostart_prompt.as_deref(),
        Some(spec_body),
        "the dispatch prompt must reach the executor as launch.autostart_prompt \
         — this is the content the agent works on, not the adapter's static kick"
    );
    assert_eq!(
        got.spec_path, "/home/op/dev/Ship/helm/data/missions/m-x/scratch/spec.md",
        "source_spec_path must reach the executor as the spec pointer for TUI delivery"
    );
}

/// SPEC: callers that send only `spec` (no `prompt`) still get content
/// delivery — spec is the fallback body for autostart_prompt.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bare_session_dispatch_falls_back_to_spec_body_for_prompt() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kernel = kernel_with_actor();
    let captured = Arc::new(StdMutex::new(None));
    let spawn_calls = Arc::new(AtomicUsize::new(0));
    let ports = ports_with(
        Arc::new(OkIsolation(tmp.path().join("worktrees/sess-p2"))),
        Arc::new(RecordingExecutor {
            captured: captured.clone(),
            spawn_calls: spawn_calls.clone(),
        }),
        Arc::new(NoopRegistry),
        Arc::new(TestPlatform {
            worktrees: tmp.path().join("worktrees"),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
        }),
    );

    let mut payload = started_payload("s-prompt-2", "sess-p2", Some("omp"), None);
    payload.spec = Some("spec-only body".to_string());

    let got = dispatch_and_capture(&kernel, &ports, &captured, &payload)
        .await
        .expect("executor.spawn called");

    assert_eq!(
        got.autostart_prompt.as_deref(),
        Some("spec-only body"),
        "with no prompt, the spec body is the autostart content"
    );
}
// (Engine cut 2026-07-10: `composed_session_is_not_spawned_here` retired
// with the engine. The no-double-spawn invariant that
// `parent_pipeline_run_id` set → `service.agents` skips the spawn is gone
// — sessions never carry pipeline identity post-cut. The bare
// `dispatch.requested` path is the only path.)


#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_emits_process_spawned_on_success() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kernel = kernel_with_actor();
    // Executor that returns a handle so the dispatch reaches the process.spawned emit.
    struct OkExecutor;
    #[async_trait]
    impl SessionExecutor for OkExecutor {
        async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
            Ok(ExecutorHandle { pid: Some(99), inner: Box::new(()) })
        }
        async fn is_alive(&self, _: &ExecutorHandle) -> bool {
            true
        }
        async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> {
            Ok(())
        }
        async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }

    let ports = ports_with(
        Arc::new(OkIsolation(tmp.path().join("worktrees/sess-3"))),
        Arc::new(OkExecutor),
        Arc::new(NoopRegistry),
        Arc::new(TestPlatform {
            worktrees: tmp.path().join("worktrees"),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
        }),
    );

    let payload = started_payload("s-disp-1", "sess-3", None, None);
    let envelope = EventEnvelope::new(
        dispatch_event_types::DISPATCH_REQUESTED,
        &payload.session_id,
        &payload,
    )
    .unwrap()
    .with_actor_id("test");
    ship_agents::handle_agents_event(&kernel, &envelope, &ports).await;
    // Poll for the detached prep to emit process.spawned.
    let mut ok = false;
    for _ in 0..50 {
        if find_process_spawned("sess-3") {
            ok = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(ok, "process.spawned emitted (entity_id=slug) after a successful spawn");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_stops_registered_handle_and_emits_process_killed() {
    let kernel = kernel_with_actor();
    let stop_calls = Arc::new(AtomicUsize::new(0));
    let cleanup_calls = Arc::new(AtomicUsize::new(0));
    let registry = Arc::new(SeededRegistry {
        handle: Mutex::new(Some(ExecutorHandle { pid: Some(42), inner: Box::new(()) })),
    });
    let ports = ports_with_projection(
        Arc::new(OkIsolation(PathBuf::from("/tmp"))),
        Arc::new(StoppingExecutor { stop_calls: stop_calls.clone() }),
        registry,
        Arc::new(TestPlatform {
            worktrees: PathBuf::from("/tmp/test-worktrees"),
            cleanup_calls: cleanup_calls.clone(),
        }),
        Arc::new(FixedSlugProjection { slug: "cancel-1".into() }),
    );

    let slug = "cancel-1";
    let payload = SessionCancelled {
        session_id: "s-cancel-1".into(),
        reason: Some("operator test".into()),
    };
    let envelope = EventEnvelope::new(
        canonical_session_event_types::SESSION_CANCELLED,
        "s-cancel-1",
        &payload,
    )
    .unwrap()
    .with_actor_id("test");

    ship_agents::handle_agents_event(&kernel, &envelope, &ports).await;

    assert_eq!(stop_calls.load(Ordering::SeqCst), 1, "executor.stop called once via L0 teardown");
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1, "platform.cleanup_job called once");
    let killed = find_process_killed(slug).expect("process.killed emitted");
    assert_eq!(killed.slug, slug);
    assert_eq!(killed.pid, Some(42));
    assert_eq!(killed.reason, "cancellation");
    assert_eq!(killed.actor, "operator");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_without_handle_still_cleans_and_emits() {
    let kernel = kernel_with_actor();
    let cleanup_calls = Arc::new(AtomicUsize::new(0));
    let ports = ports_with_projection(
        Arc::new(OkIsolation(PathBuf::from("/tmp"))),
        Arc::new(StoppingExecutor { stop_calls: Arc::new(AtomicUsize::new(0)) }),
        Arc::new(NoopRegistry),
        Arc::new(TestPlatform {
            worktrees: PathBuf::from("/tmp/test-worktrees"),
            cleanup_calls: cleanup_calls.clone(),
        }),
        Arc::new(FixedSlugProjection { slug: "cancel-2".into() }),
    );

    let slug = "cancel-2";
    let payload = SessionCancelled {
        session_id: "s-cancel-2".into(),
        reason: None,
    };
    let envelope = EventEnvelope::new(
        canonical_session_event_types::SESSION_CANCELLED,
        "s-cancel-2",
        &payload,
    )
    .unwrap()
    .with_actor_id("test");

    ship_agents::handle_agents_event(&kernel, &envelope, &ports).await;

    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1, "cleanup still runs on a registry miss");
    let killed = find_process_killed(slug).expect("process.killed emitted");
    assert_eq!(killed.pid, None, "no handle → pid None in the audit event");
}

// ── Floki routing tests ─────────────────────────────────────────────────────

/// SPEC: when `payload.harness == "floki"` and `floki_run_fn` is set,
/// dispatch routes to the closure and skips the executor entirely.
///
/// The closure increments a counter so the test can assert it was called.
/// `spawn_calls` remains zero because the harness-driver path (executor.spawn)
/// is never entered.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn floki_harness_routes_to_floki_run_fn_and_skips_executor() {
    use ship_agents::ports::FlokiRunFn;
    use std::sync::atomic::AtomicBool;

    let tmp = tempfile::TempDir::new().unwrap();
    let kernel = kernel_with_actor();

    // Closure flag — set to true when the floki_run_fn fires.
    let floki_called = Arc::new(AtomicBool::new(false));
    let floki_called_inner = floki_called.clone();
    let spawn_calls = Arc::new(AtomicUsize::new(0));

    let floki_run_fn: FlokiRunFn = Arc::new(move |_payload| {
        let flag = floki_called_inner.clone();
        Box::pin(async move {
            flag.store(true, Ordering::SeqCst);
        })
    });

    let ports = AgentPorts {
        floki_run_fn: Some(floki_run_fn),
        isolation: Arc::new(OkIsolation(tmp.path().join("worktrees/fl-sess"))),
        executor: Arc::new(RecordingExecutor {
            captured: Arc::new(StdMutex::new(None)),
            spawn_calls: spawn_calls.clone(),
        }),
        platform: Arc::new(TestPlatform {
            worktrees: tmp.path().join("worktrees"),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
        }),
        registry: Arc::new(NoopRegistry),
        spawn_policy: Arc::new(TokenSpawnPolicy),
        projections: Arc::new(NeverTerminal),
        event_dedup: Arc::new(EventDedup::new()),
    };

    // harness == "floki" is the routing key.
    let payload = started_payload("s-floki-1", "fl-sess", Some("floki"), None);
    let envelope = EventEnvelope::new(
        dispatch_event_types::DISPATCH_REQUESTED,
        &payload.session_id,
        &payload,
    )
    .unwrap()
    .with_actor_id("test");

    ship_agents::handle_agents_event(&kernel, &envelope, &ports).await;

    // Poll: the floki_run_fn is spawned as a task; give it time to run.
    for _ in 0..50 {
        if floki_called.load(Ordering::SeqCst) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert!(
        floki_called.load(Ordering::SeqCst),
        "floki_run_fn must be called when harness == 'floki'",
    );
    assert_eq!(
        spawn_calls.load(Ordering::SeqCst),
        0,
        "executor.spawn must NOT be called when floki_run_fn routes the session",
    );
}

/// Loud refusal per seam-leak fix §6: when `harness == "floki"` and no floki
/// closure is wired, the dispatch was previously *silently* routed to the
/// harness executor (a footgun — floki's whole point is to skip worktree /
/// sandbox). After the fix the actor emits a typed `session.failed` event
/// and the call returns without touching the executor.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn floki_harness_without_closure_emits_session_failed() {
    let tmp = tempfile::TempDir::new().unwrap();
    let kernel = kernel_with_actor();
    let spawn_calls = Arc::new(AtomicUsize::new(0));

    // floki_run_fn = None — closure not wired.
    let ports = AgentPorts {
        floki_run_fn: None,
        isolation: Arc::new(OkIsolation(tmp.path().join("worktrees/fl-no-fn"))),
        executor: Arc::new(RecordingExecutor {
            captured: Arc::new(StdMutex::new(None)),
            spawn_calls: spawn_calls.clone(),
        }),
        platform: Arc::new(TestPlatform {
            worktrees: tmp.path().join("worktrees"),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
        }),
        registry: Arc::new(NoopRegistry),
        spawn_policy: Arc::new(TokenSpawnPolicy),
        projections: Arc::new(NeverTerminal),
        event_dedup: Arc::new(EventDedup::new()),
    };

    let payload = started_payload("s-floki-2", "fl-no-fn", Some("floki"), None);
    let envelope = EventEnvelope::new(
        dispatch_event_types::DISPATCH_REQUESTED,
        &payload.session_id,
        &payload,
    )
    .unwrap()
    .with_actor_id("test");

    ship_agents::handle_agents_event(&kernel, &envelope, &ports).await;
    // emit_session_failed runs synchronously on the actor's mailbox loop —
    // no detached task, no need to poll.

    assert_eq!(
        spawn_calls.load(Ordering::SeqCst),
        0,
        "no floki closure + harness='floki' must NOT route to executor — \
         the loud-refusal fix lands a typed `session.failed` instead",
    );

}

/// SPEC: `session.cancelled` emitted by the Stop RPC is consumed by TWO
/// components driven by the same event:
///
/// 1. The `service.agents` actor (`handle_agent_cancelled`) resolves the slug,
///    tears down the process (executor.stop + cleanup_job), and emits
///    `process.killed` for audit.
/// 2. `CanonicalSessionProjection` transitions the read-model to
///    status = Cancelled, outcome = "cancelled", ended_at set.
///
/// This test drives the full dual-consumer path to prove the #26 design:
/// the Stop RPC is the emit seam — it emits one `session.cancelled` event
/// (Space-scoped, canonical payload) which both the actor teardown logic AND
/// the projection engine respond to independently.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_rpc_session_cancelled_actor_tears_down_and_projection_shows_cancelled() {
    const SESSION_ID: &str = "s-stop-3";
    const SLUG: &str = "stop-3";

    let ship_dir = ship_substrate::project::get_global_dir().unwrap();
    let mut conn = ship_substrate::db::open_runtime_db(&ship_dir).unwrap();

    // ── 1. Clean slate ─────────────────────────────────────────────────────
    let _ = block_on(async {
        sqlx::query("DELETE FROM canonical_sessions WHERE session_id = ?")
            .bind(SESSION_ID)
            .execute(&mut conn)
            .await
    });

    // ── 2. Seed: session.started → CanonicalSessionProjection ─────────────
    // Mimics what dispatch_core emits when a harness session is launched.
    let started_env = EventEnvelope::new(
        canonical_session_event_types::SESSION_STARTED,
        SESSION_ID,
        &serde_json::json!({
            "session_id": SESSION_ID,
            "agent_id":   "claude",
            "harness":    "claude",
            "model":      "claude-sonnet-4-6",
            "space_id":   "sp-stop-3",
            "slug":       SLUG,
            "input":      "implement feature X"
        }),
    )
    .unwrap();
    let proj = CanonicalSessionProjection;
    proj.apply(&started_env, &mut conn).unwrap();

    // Precondition: session exists and is Active before cancellation.
    let before = load_canonical_session_by_id(SESSION_ID)
        .unwrap()
        .expect("seeded session must exist");
    assert_eq!(
        before.status,
        CanonicalSessionStatus::Active,
        "precondition: session is Active before cancellation"
    );
    assert_eq!(before.slug.as_deref(), Some(SLUG), "precondition: slug populated");

    // ── 3. Build session.cancelled envelope ───────────────────────────────
    // Matches exactly what stop_fn in apps/shipd/src/ops_rpc.rs emits:
    // canonical payload (session_id + reason), actor_id = "service.agents".
    let cancel_payload = SessionCancelled {
        session_id: SESSION_ID.into(),
        reason: Some("operator stop".into()),
    };
    let cancel_env = EventEnvelope::new(
        canonical_session_event_types::SESSION_CANCELLED,
        SESSION_ID,
        &cancel_payload,
    )
    .unwrap()
    .with_actor_id("service.agents");

    // ── 4. Actor: session.cancelled → process teardown ────────────────────
    // handle_agents_event routes to handle_agent_cancelled.
    let kernel = kernel_with_actor();
    let stop_calls = Arc::new(AtomicUsize::new(0));
    let cleanup_calls = Arc::new(AtomicUsize::new(0));
    let registry = Arc::new(SeededRegistry {
        handle: Mutex::new(Some(ExecutorHandle { pid: Some(99), inner: Box::new(()) })),
    });
    let ports = ports_with_projection(
        Arc::new(OkIsolation(PathBuf::from("/tmp"))),
        Arc::new(StoppingExecutor { stop_calls: stop_calls.clone() }),
        registry,
        Arc::new(TestPlatform {
            worktrees: PathBuf::from("/tmp/test-worktrees"),
            cleanup_calls: cleanup_calls.clone(),
        }),
        Arc::new(FixedSlugProjection { slug: SLUG.into() }),
    );

    ship_agents::handle_agents_event(&kernel, &cancel_env, &ports).await;

    // Process torn down via L0.
    assert_eq!(stop_calls.load(Ordering::SeqCst), 1, "executor.stop called once via actor teardown");
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1, "platform.cleanup_job called once");
    let killed = find_process_killed(SLUG).expect("process.killed emitted for audit");
    assert_eq!(killed.slug, SLUG, "process.killed keyed on slug");
    assert_eq!(killed.pid, Some(99), "pid from registered handle threads into process.killed");
    assert_eq!(killed.reason, "cancellation");
    assert_eq!(killed.actor, "operator");

    // ── 5. Projection: session.cancelled → Cancelled read-model ──────────
    // Simulates the KernelRouter routing the same event through
    // CanonicalSessionProjection (CanonicalSessionProjection::apply opens its
    // own runtime.db connection internally; _conn is only a placeholder).
    proj.apply(&cancel_env, &mut conn).unwrap();

    let after = load_canonical_session_by_id(SESSION_ID)
        .unwrap()
        .expect("session must still exist in the read-model after cancellation");
    assert_eq!(
        after.status,
        CanonicalSessionStatus::Cancelled,
        "canonical projection must show Cancelled after session.cancelled applied"
    );
    assert_eq!(after.outcome.as_deref(), Some("cancelled"), "outcome = cancelled");
    assert_eq!(
        after.reason.as_deref(),
        Some("operator stop"),
        "reason threads from payload into the read-model"
    );
    assert!(after.ended_at.is_some(), "ended_at must be populated on cancellation");
}

// ── session.failed emit scope ───────────────────────────────────────────────
//
// Closes c-2026-07-05T20-30-00-000Z-b8c9d0e1. The dispatch actor's
// `emit_session_failed` (in src/dispatch.rs) builds an EventEnvelope
// for the Space-scoped `session.failed` event and hands it to
// `persist_and_route`. If the envelope lacks a `scope_key`, the kernel
// router's `enforce_scope` rejects it as "space-unbound caller cannot
// emit a Space-scoped event" (NoSpaceForSpaceEvent) — failed sessions
// leave no event trail.
//
// The discipline: the canonical session.* family uses the session_id
// itself as the scope_key (see agent-def/src/emit.rs::build_space_envelope
// — Space, scope_key=session_id, session_id=session_id). The dispatch
// actor must follow the same discipline so the binding derived from the
// envelope is space-bound and the kernel accepts the emit.
fn read_session_failed_from_space_db(
    ship_dir: &std::path::Path,
    session_id: &str,
) -> Option<EventEnvelope> {
    use ship_substrate::persistence::EventStore;
    let path = ship_substrate::db::space_db_path(ship_dir, session_id);
    if !path.exists() {
        return None;
    }
    let store = ship_substrate::persistence::SqliteEventStore::new_at_db_path(&path).ok()?;
    let mut found = store
        .query(&ship_substrate::events::EventFilter {
            event_type: Some(canonical_session_event_types::SESSION_FAILED.to_string()),
            entity_id: Some(session_id.to_string()),
            ..Default::default()
        })
        .ok()?;
    found.pop()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_dispatch_emits_session_failed_with_space_bound_envelope() {
    // The `agent_session_failed_nonzero_keys_on_session_id` test (in
    // ship-code) covers the post-spawn case. This test covers the
    // prep-failure path in the dispatch actor specifically: isolation
    // succeeds, executor.spawn errors → the actor must emit
    // `session.failed` AND the kernel must accept it (no
    // space-unbound reject).
    let tmp = tempfile::TempDir::new().unwrap();
    let ship_dir = ship_substrate::project::get_global_dir().unwrap();
    let session_id = "s-sf-bug-1";
    let slug = "sf-bug-1";

    // Clean any prior state for this session.
    let _ = block_on(async {
        let mut conn = ship_substrate::db::open_runtime_db(&ship_dir).unwrap();
        sqlx::query("DELETE FROM canonical_sessions WHERE session_id = ?")
            .bind(session_id)
            .execute(&mut conn)
            .await
    });
    let space_path = ship_substrate::db::space_db_path(&ship_dir, session_id);
    if space_path.exists() {
        std::fs::remove_file(&space_path).ok();
    }

    let kernel = kernel_with_actor();
    let ports = ports_with(
        Arc::new(OkIsolation(tmp.path().join("worktrees/sf-bug-1"))),
        Arc::new(RecordingExecutor {
            // RecordingExecutor errors on spawn → triggers the
            // `executor spawn failed` branch in dispatch_agent_session_internal
            // which calls emit_session_failed.
            captured: Arc::new(StdMutex::new(None)),
            spawn_calls: Arc::new(AtomicUsize::new(0)),
        }),
        Arc::new(NoopRegistry),
        Arc::new(TestPlatform {
            worktrees: tmp.path().join("worktrees"),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
        }),
    );

    let payload = started_payload(session_id, slug, Some("codex"), Some("gpt-5-codex"));
    let envelope = EventEnvelope::new(
        dispatch_event_types::DISPATCH_REQUESTED,
        session_id,
        &payload,
    )
    .unwrap()
    .with_actor_id("test");
    ship_agents::handle_agents_event(&kernel, &envelope, &ports).await;

    // The dispatch prep is detached under DISPATCH_GATE. Poll the
    // per-space db until the session.failed lands (or we time out).
    let mut found = false;
    for _ in 0..50 {
        if read_session_failed_from_space_db(&ship_dir, session_id).is_some() {
            found = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(
        found,
        "session.failed for {session_id} must land in spaces/{session_id}/space.db — \
         a failing dispatch leaves no event trail when the dispatch actor's \
         emit_session_failed is rejected by the kernel as a space-unbound caller \
         (c-2026-07-05T20-30-00-000Z-b8c9d0e1)"
    );

    // The event itself carries the canonical discipline: Space scope,
    // scope_key=session_id, session_id=session_id. Pinning these
    // guarantees the actor stays aligned with the canonical builder.
    let env = read_session_failed_from_space_db(&ship_dir, session_id)
        .expect("just observed the event");
    assert_eq!(
        env.event_type,
        canonical_session_event_types::SESSION_FAILED
    );
    assert_eq!(env.entity_id, session_id);
    assert_eq!(env.scope, ship_types::event_registry::EventScope::Space);
    assert_eq!(env.scope_key.as_deref(), Some(session_id));
    assert_eq!(env.session_id.as_deref(), Some(session_id));
}
