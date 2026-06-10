//! Tests for `execute_merge_step` — specifically that `Ok(false)` emits
//! `pipeline.run.failed` so operators are never silently left with a job
//! stuck in Completed state after a declined merge.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{event_types, PipelineRunFailedPayload};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};

use ship_code::dispatch_ports::AgentPlatform;
use crate::orchestrator::handlers::execute_merge_step;

// ── Mocks ──────────────────────────────────────────────────────────────────

struct MergePlatform(anyhow::Result<bool>);

// SAFETY: tests run in a single-threaded fashion per test; the bool is not
// shared across threads. We wrap in a newtype to satisfy Send + Sync.
unsafe impl Send for MergePlatform {}
unsafe impl Sync for MergePlatform {}

#[async_trait]
impl AgentPlatform for MergePlatform {
    fn worktrees_dir(&self) -> PathBuf {
        PathBuf::from("/tmp")
    }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> {
        match &self.0 {
            Ok(v) => Ok(*v),
            Err(e) => Err(anyhow::anyhow!("{e}")),
        }
    }
    async fn cleanup_orphaned_worktrees(&self) {}
}

// ── Harness ────────────────────────────────────────────────────────────────

fn actor_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.orchestrator".to_string(),
        write_namespaces: vec!["pipeline.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec![],
        see_directed_events: false,
    }
}

fn find_run_failed(pipeline_run_id: &str) -> Option<PipelineRunFailedPayload> {
    let events = ship_substrate::db::events::list_all_events().ok()?;
    events.into_iter().find_map(|e| {
        if e.entity_id == pipeline_run_id && e.event_type == event_types::PIPELINE_RUN_FAILED {
            serde_json::from_str(&e.payload_json).ok()
        } else {
            None
        }
    })
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn merge_ok_false_emits_pipeline_run_failed() {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base.clone()).unwrap();
    router.spawn_actor("service.orchestrator", actor_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let pipeline_run_id = "merge-false-test-run-01";
    let platform = MergePlatform(Ok(false));

    let result = execute_merge_step(
        &kernel,
        pipeline_run_id,
        "test-slug",
        "job/test-slug",
        &platform,
    )
    .await;

    assert!(!result, "execute_merge_step must return false on Ok(false)");

    let emitted = find_run_failed(pipeline_run_id)
        .expect("pipeline.run.failed must be emitted when merge returns Ok(false)");
    assert_eq!(emitted.pipeline_run_id, pipeline_run_id);
    assert!(
        emitted.error.contains("merge declined"),
        "error must mention merge declined: {}",
        emitted.error
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn merge_ok_true_does_not_emit_failed() {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base.clone()).unwrap();
    router.spawn_actor("service.orchestrator", actor_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let pipeline_run_id = "merge-true-test-run-01";
    let platform = MergePlatform(Ok(true));

    let result = execute_merge_step(
        &kernel,
        pipeline_run_id,
        "test-slug-ok",
        "job/test-slug-ok",
        &platform,
    )
    .await;

    assert!(result, "execute_merge_step must return true on Ok(true)");
    assert!(
        find_run_failed(pipeline_run_id).is_none(),
        "pipeline.run.failed must NOT be emitted on successful merge"
    );
}
