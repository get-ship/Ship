//! Run step executor — executes shell commands as pipeline steps.
//!
//! A run step is a pipeline step with `type: run`. It executes a shell
//! command in the job's worktree directory, capturing output and emitting
//! step lifecycle events to the kernel.
//!
//! On exit 0: emits `pipeline.step.output` then `pipeline.step.completed`.
//! On non-zero exit: emits `pipeline.step.output` then `pipeline.step.failed`.
//!
//! **Journaled (C4 / P2.2):** the step runs as `RunStepProgram` through the
//! durable-execution engine. Each op (emit-started / execute / emit-terminal)
//! is journaled under the key `<pipeline_run_id>/step/<step_id>` — distinct
//! from the run's `DispatchProgram` journal, which lives under the bare
//! `pipeline_run_id`. A step whose journal already holds a completed
//! `run_step.execute` op replays the logged result: the command is NOT
//! re-executed and the step events are NOT re-emitted.

use async_trait::async_trait;
use durable_execution::{Dispatch, DispatchError, Journal, JournalStore, RunStatus};
use serde_json::{Value, json};
use ship_substrate::StepExecutor;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::coordinator_programs::RunStepProgram;
use super::emit::persist_and_route;

/// The journal key for one step of one run. The bare `pipeline_run_id` is the
/// `DispatchProgram` journal — a different op sequence, so sharing the key
/// would trip the engine's replay-divergence guard.
fn step_journal_key(pipeline_run_id: &str, step_id: &str) -> String {
    format!("{pipeline_run_id}/step/{step_id}")
}

/// Live dispatch for [`RunStepProgram`]: owns the real `StepExecutor` call and
/// the `pipeline.step.*` emissions. Ops run on the live frontier only — on
/// replay the engine returns logged results without calling back in here.
struct RunStepDispatch {
    kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
    pipeline_run_id: String,
    slug: String,
    step_id: String,
    command: String,
    work_dir: PathBuf,
    ship_dir: Option<PathBuf>,
}

impl RunStepDispatch {
    async fn emit(&self, event_type: &str, payload: &Value) {
        if let Ok(env) = EventEnvelope::new(event_type, &self.pipeline_run_id, payload) {
            persist_and_route(
                &self.kernel,
                env.with_actor_id("service.orchestrator"),
                &self.slug,
                self.ship_dir.as_deref(),
            )
            .await;
        }
    }
}

#[async_trait]
impl Dispatch for RunStepDispatch {
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
        match op {
            "run_step.emit_started" => {
                tracing::info!(
                    slug = %self.slug, step_id = %self.step_id, command = %self.command,
                    "run-step: executing"
                );
                self.emit(
                    event_types::PIPELINE_STEP_STARTED,
                    &json!({
                        "pipeline_run_id": self.pipeline_run_id, "slug": self.slug,
                        "step_id": self.step_id, "step_type": "run",
                    }),
                )
                .await;
                Ok(json!({}))
            }
            // Executor failures (spawn error / timeout / memory cap) are encoded
            // in the result value, NOT surfaced as Err — the terminal-emission op
            // must still run and be journaled on the failure path.
            "run_step.execute" => match StepExecutor.run(&self.command, &self.work_dir).await {
                Ok(out) => Ok(json!({
                    "ran": true,
                    "success": out.success,
                    "exit_code": out.exit_code,
                    "stdout": out.stdout,
                    "stderr": out.stderr,
                })),
                Err(e) => Ok(json!({
                    "ran": false,
                    "success": false,
                    "error": format!("{e}"),
                })),
            },
            "run_step.emit_terminal" => {
                let result = &args["result"];
                if !result["ran"].as_bool().unwrap_or(false) {
                    let error = result["error"].as_str().unwrap_or("unknown");
                    tracing::error!(
                        slug = %self.slug, step_id = %self.step_id,
                        "run-step: step executor error: {error}"
                    );
                    self.emit(
                        event_types::PIPELINE_STEP_FAILED,
                        &json!({
                            "pipeline_run_id": self.pipeline_run_id, "slug": self.slug,
                            "step_id": self.step_id,
                            "error": format!("step executor: {error}"),
                        }),
                    )
                    .await;
                    return Ok(json!({}));
                }

                let stdout = result["stdout"].as_str().unwrap_or("");
                let stderr = result["stderr"].as_str().unwrap_or("");
                self.emit(
                    event_types::PIPELINE_STEP_OUTPUT,
                    &json!({
                        "pipeline_run_id": self.pipeline_run_id, "step_id": self.step_id,
                        "stdout": if stdout.is_empty() { Value::Null } else { Value::String(stdout.into()) },
                        "stderr": if stderr.is_empty() { Value::Null } else { Value::String(stderr.into()) },
                    }),
                )
                .await;

                if result["success"].as_bool().unwrap_or(false) {
                    tracing::info!(
                        slug = %self.slug, step_id = %self.step_id,
                        "run-step: completed successfully"
                    );
                    self.emit(
                        event_types::PIPELINE_STEP_COMPLETED,
                        &json!({
                            "pipeline_run_id": self.pipeline_run_id, "slug": self.slug,
                            "step_id": self.step_id,
                        }),
                    )
                    .await;
                } else {
                    let exit_code = result["exit_code"].as_i64().unwrap_or(-1);
                    tracing::warn!(
                        slug = %self.slug, step_id = %self.step_id, exit_code,
                        "run-step: command failed"
                    );
                    self.emit(
                        event_types::PIPELINE_STEP_FAILED,
                        &json!({
                            "pipeline_run_id": self.pipeline_run_id, "slug": self.slug,
                            "step_id": self.step_id,
                            "error": format!("command exited with code {exit_code}: {stderr}"),
                        }),
                    )
                    .await;
                }
                Ok(json!({}))
            }
            other => Err(DispatchError::Terminal(format!(
                "RunStepDispatch: unknown op `{other}`"
            ))),
        }
    }
}

/// Execute a shell command as a pipeline step, journaled.
///
/// Loads (or starts) the step's journal, runs [`RunStepProgram`] through the
/// engine — emitting `pipeline.step.started`, running the command via the
/// `StepExecutor` port (backed by `spawn_bounded` — 2 GB virtual-memory cap,
/// 30 s timeout), then emitting `pipeline.step.output` +
/// `pipeline.step.completed`/`failed` — and saves the advanced journal.
/// A replayed step (journal already complete) re-executes nothing.
pub async fn execute_run_step(
    pipeline_run_id: &str,
    slug: &str,
    step_id: &str,
    command: &str,
    work_dir: &Path,
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    ship_dir: Option<&Path>,
    journal_store: &Arc<dyn JournalStore>,
) {
    let key = step_journal_key(pipeline_run_id, step_id);
    let journal = match journal_store.load(&key).await {
        Ok(Some(j)) => j,
        Ok(None) => Journal::new(),
        Err(e) => {
            tracing::warn!(slug, step_id, "run-step: journal load failed, starting fresh: {e}");
            Journal::new()
        }
    };

    let dispatch = Arc::new(RunStepDispatch {
        kernel: kernel.clone(),
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        step_id: step_id.to_string(),
        command: command.to_string(),
        work_dir: work_dir.to_path_buf(),
        ship_dir: ship_dir.map(|p| p.to_path_buf()),
    });

    let (status, journal) = durable_execution::execute(journal, dispatch, &RunStepProgram).await;

    if let Err(e) = journal_store.save(&key, &journal).await {
        tracing::warn!(slug, step_id, "run-step: journal save failed (non-fatal): {e}");
    }
    if let RunStatus::Failed(e) = status {
        // Engine-level failure (dispatch error / replay divergence) — the
        // step's own command failures are handled inside emit_terminal.
        tracing::error!(slug, step_id, "run-step: engine failure: {e}");
    }
}
