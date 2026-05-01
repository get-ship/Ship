//! Job event payload types.
//!
//! Job state is derived entirely from the event log. These payloads are the
//! only things written to the `events` table under the `job.` namespace.

use serde::{Deserialize, Serialize};
use specta::Type;

/// A single phase in a job pipeline (legacy linear format).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelinePhase {
    pub agent: String,
    pub goal: String,
    #[serde(default)]
    pub model: Option<String>,
}

/// Step type in a pipeline DAG.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    Agent,
    /// Agent-backed review of the previous step. Distinct from a
    /// step's declarative `gate:` attribute (a pure diff-rule check,
    /// see `GateSpec`). A `Review` step spawns a reviewer agent that
    /// inspects the previous step's work and emits a verdict; on
    /// failure, the gated step is retried up to its `max_retries`.
    Review,
    /// Execute a shell command. Exit 0 = success, non-zero = failure.
    Run,
}

/// Scope of a step output artifact.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputScope {
    /// Ephemeral — lives for the duration of the run. Default.
    #[default]
    Run,
    /// Persistent project-level artifact.
    Project,
    /// Persistent user-personal artifact.
    User,
}

/// A named output produced by a pipeline step.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepOutput {
    /// Logical handle, unique within this step.
    pub name: String,
    /// Storage scope. Defaults to `run`.
    #[serde(default)]
    pub scope: OutputScope,
    /// When `true`, the step fails if this output is absent from `step_done.outputs`.
    #[serde(default)]
    pub required: bool,
}

/// Promote a named output to a wider scope when the step succeeds.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepPromote {
    /// Must match a `produces.name` on this step.
    pub name: String,
    /// Valid transitions: Run→Project, Run→User, Project→User.
    pub target_scope: OutputScope,
}

/// A single lane in a fanout step — runs the same spec through a different agent/model/provider.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FanoutLane {
    /// Unique identifier within the fanout.
    pub id: String,
    /// Agent profile to use for this lane.
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Points to the judge step that arbitrates lane outputs.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FanoutJudge {
    /// Step ID of the judge. Must come after the fanout step in the pipeline DAG.
    pub step: String,
}

/// Fanout configuration — run K agents in parallel, then a judge step picks or merges.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FanoutSpec {
    /// Parallel lanes. Each runs the fanout step's spec through a different agent/model/provider.
    pub lanes: Vec<FanoutLane>,
    /// Judge step that receives all lane outputs and arbitrates.
    pub judge: FanoutJudge,
    /// Minimum number of successful lanes required for the fanout step to succeed. Default: 1.
    #[serde(default = "default_min_successful_lanes")]
    pub min_successful_lanes: usize,
}

fn default_min_successful_lanes() -> usize { 1 }

/// A single step in a pipeline DAG.
///
/// Custom `Deserialize` allows YAML shorthand: `run: <command>` implies
/// `type: run` and `name:` is accepted as an alias for `id:`.
#[derive(Debug, Clone, Serialize, Type)]
pub struct PipelineStep {
    /// Step identifier, unique within the pipeline.
    pub id: String,
    /// What kind of step this is.
    #[serde(rename = "type")]
    pub step_type: StepType,
    /// Agent profile to use (required for agent/gate steps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Goal or instruction for the agent step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    /// Steps that must complete before this one runs.
    #[serde(default)]
    pub after: Vec<String>,
    /// Model override for this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Provider override for this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Shell command to run (populated for StepType::Run).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Maximum number of retries when a gate fails on this step.
    /// `None` or `0` means no retries — gate failure is terminal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// Post-step diff gate. Declarative structural checks evaluated against
    /// the files changed by this step's commits. See `services::orchestrator::gate_rules`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<GateSpec>,
    /// Artifacts this step is expected to produce. The agent uploads them via
    /// `artifact_upload` and reports `{ name: artifact_id }` pairs in
    /// `step_done.outputs`. Required outputs fail the step when absent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub produces: Vec<StepOutput>,
    /// Names of outputs from prior steps that this step needs. Resolved to
    /// artifact records by the orchestrator before dispatch and written to
    /// `.artifacts/input/<name>` in the worktree.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumes: Vec<String>,
    /// Outputs to promote to a wider scope when this step succeeds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub promotes: Vec<StepPromote>,
    /// When present, run K parallel lanes instead of a single agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fanout: Option<FanoutSpec>,
}

/// Declarative diff-based gate attached to a step. Runtime evaluates after the
/// step's commits are sealed, before the next step is dispatched.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct GateSpec {
    /// Every entry must match at least `min` changed paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_rules")]
    pub require_changed: Vec<GateRule>,
    /// Any match across entries fails the gate.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_rules")]
    pub forbid_changed: Vec<GateRule>,
}

/// A single gate rule. `match` and `not_match` are lists of glob patterns.
///
/// Canonical YAML/JSON shape (match and not_match take glob patterns):
///
///     match: ["src/**", "Cargo.toml"]
///     not_match: ["tests/**"]
///     min: 1
///
/// Earlier revisions accepted a pipe-separated string for `match`/`not_match`
/// and a single rule object for `require_changed`/`forbid_changed`; both
/// shorthand forms are now rejected at load time. See
/// `crates/core/runtime/src/events/job.rs::deserialize_rules` for the error
/// the loader surfaces and the contract-freeze (Item 1) note in
/// `apps/docs/src/content/docs/reference/pipelines/reference.md`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GateRule {
    #[serde(rename = "match", deserialize_with = "deserialize_patterns")]
    pub match_patterns: Vec<String>,
    #[serde(default, rename = "not_match",
            deserialize_with = "deserialize_patterns")]
    pub not_match: Vec<String>,
    #[serde(default = "default_min")]
    pub min: u32,
}

fn default_min() -> u32 { 1 }

/// Canonical: a list of rule objects. The legacy single-object shorthand is
/// rejected with an error pointing at the canonical syntax.
fn deserialize_rules<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<GateRule>, D::Error> {
    use serde::de::Error;
    let val = serde_json::Value::deserialize(d)?;
    if !val.is_array() {
        return Err(D::Error::custom(
            "gate rule list must be a YAML/JSON array of rule objects \
             (e.g. `require_changed: [{ match: [\"**/*.rs\"], min: 1 }]`); \
             the legacy single-object shorthand was removed in the v1 \
             contract freeze",
        ));
    }
    serde_json::from_value(val).map_err(D::Error::custom)
}

/// Canonical: a list of glob strings. The legacy pipe-separated string
/// shorthand is rejected with an error pointing at the canonical syntax.
fn deserialize_patterns<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    use serde::de::Error;
    let val = serde_json::Value::deserialize(d)?;
    match val {
        serde_json::Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (idx, item) in items.into_iter().enumerate() {
                match item {
                    serde_json::Value::String(s) => {
                        if s.contains('|') {
                            return Err(D::Error::custom(format!(
                                "gate pattern[{idx}] '{s}' contains '|'; \
                                 pipe-separated patterns were removed in the \
                                 v1 contract freeze. Split into separate list \
                                 entries: `match: [\"a\", \"b\"]`"
                            )));
                        }
                        if s.is_empty() {
                            return Err(D::Error::custom(format!(
                                "gate pattern[{idx}] is empty"
                            )));
                        }
                        out.push(s);
                    }
                    other => return Err(D::Error::custom(format!(
                        "gate pattern[{idx}] must be a glob string, got {other:?}"
                    ))),
                }
            }
            Ok(out)
        }
        serde_json::Value::String(_) => Err(D::Error::custom(
            "gate `match`/`not_match` must be a list of glob strings \
             (e.g. `match: [\"**/*.rs\", \"**/*.toml\"]`); the legacy \
             pipe-separated string shorthand was removed in the v1 \
             contract freeze",
        )),
        other => Err(D::Error::custom(format!(
            "gate `match`/`not_match` must be a list of glob strings, got {other:?}"
        ))),
    }
}


impl<'de> serde::Deserialize<'de> for PipelineStep {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            #[serde(alias = "name")]
            id: String,
            #[serde(rename = "type", default)]
            step_type: Option<StepType>,
            #[serde(default)]
            agent: Option<String>,
            #[serde(default)]
            goal: Option<String>,
            #[serde(default)]
            after: Vec<String>,
            #[serde(default)]
            model: Option<String>,
            #[serde(default)]
            provider: Option<String>,
            /// Accept both `run:` (YAML shorthand) and `command:`.
            #[serde(alias = "run", default)]
            command: Option<String>,
            #[serde(default)]
            max_retries: Option<u32>,
            #[serde(default)]
            gate: Option<GateSpec>,
            #[serde(default)]
            produces: Vec<StepOutput>,
            #[serde(default)]
            consumes: Vec<String>,
            #[serde(default)]
            promotes: Vec<StepPromote>,
            #[serde(default)]
            fanout: Option<FanoutSpec>,
        }
        let raw = Raw::deserialize(d)?;
        let step_type = raw.step_type.unwrap_or_else(|| {
            if raw.command.is_some() {
                StepType::Run
            } else {
                StepType::Agent
            }
        });
        Ok(PipelineStep {
            id: raw.id,
            step_type,
            agent: raw.agent,
            goal: raw.goal,
            after: raw.after,
            model: raw.model,
            provider: raw.provider,
            command: raw.command,
            max_retries: raw.max_retries,
            gate: raw.gate,
            produces: raw.produces,
            consumes: raw.consumes,
            promotes: raw.promotes,
            fanout: raw.fanout,
        })
    }
}

/// A pipeline definition — a DAG of steps.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
    /// When true, the orchestrator runs `git merge` on the job branch
    /// after the final step completes and emits `job.merged`. Replaces
    /// the former `StepType::Merge` step kind — merging is a property
    /// of the pipeline, not a step type.
    #[serde(default)]
    pub auto_merge: bool,
}

impl Pipeline {
    /// Find root steps (no dependencies).
    pub fn root_steps(&self) -> Vec<&PipelineStep> {
        self.steps.iter().filter(|s| s.after.is_empty()).collect()
    }

    /// Find steps unblocked by completed step IDs.
    pub fn unblocked_steps(&self, completed: &[String]) -> Vec<&PipelineStep> {
        self.steps.iter()
            .filter(|s| !completed.contains(&s.id))
            .filter(|s| s.after.iter().all(|dep| completed.contains(dep)))
            .collect()
    }
}

/// Emitted when a job is created. `entity_id` in the envelope is the job_id.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobCreatedPayload {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    /// Operator-facing path to the spec file as passed to `ship run --spec`
    /// (or the dispatch.sh equivalent). Distinct from `spec_path` which is
    /// the daemon-side worktree destination. None for legacy events and
    /// for `--spec-text` / inline-spec flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_spec_path: Option<String>,
    pub plan_id: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
    /// Legacy linear pipeline. Kept for backward compat.
    #[serde(default)]
    pub pipeline: Option<Vec<PipelinePhase>>,
    /// DAG pipeline. Takes precedence over `pipeline` if both are set.
    #[serde(default)]
    pub steps: Option<Pipeline>,
    /// Pipeline name reference (e.g. "tdd"). Resolved by daemon from .ship/pipelines/.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_name: Option<String>,
    /// Job-level variables for pipeline step interpolation. Override step-level vars.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vars: Option<std::collections::HashMap<String, String>>,
    /// Inline spec content. When present, the worktree adapter writes this to
    /// `spec_path` at dispatch time. Makes jobs self-contained across worktree
    /// boundaries — no reliance on gitignored files existing in the main checkout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_content: Option<String>,
    /// Per-step spec bodies, indexed by step position. Populated by
    /// `ensure_pipeline` from the pipeline YAML's `step_specs:` block when
    /// present. Written to `.artifacts/pipeline/NN-<step_id>.md` at launch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_specs: Option<Vec<String>>,
    /// When true, run steps headless (ProcessExecutor). When false (default),
    /// run steps as tmux windows in a shared per-slug session
    /// (TmuxWindowExecutor) so users can `tmux attach` and watch the agent
    /// live.
    #[serde(default)]
    pub headless: bool,
    /// HEAD SHA of the source repo when this job was created. Pipeline
    /// gates evaluate `require_changed` / `forbid_changed` against the
    /// diff from this SHA to HEAD, so the checks see work committed in
    /// any earlier pipeline step, not just the gating step. `None` for
    /// legacy events predating this field; gates fall back to the
    /// step's start SHA in that case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_sha: Option<String>,
}

/// Emitted when the job is assigned to a worktree and (optionally) a process.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobDispatchedPayload {
    pub job_id: String,
    pub worktree: String,
    pub pid: Option<u32>,
    /// Provider-specific session metadata (e.g. Claude session ID for --resume).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_meta: Option<serde_json::Value>,
}

/// Emitted when a gate review is requested for the job.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobGateRequestedPayload {
    pub job_id: String,
    pub gate_agent: String,
}

/// Emitted when the gate review passes.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobGatePassedPayload {
    pub job_id: String,
}

/// Emitted when the gate review fails.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobGateFailedPayload {
    pub job_id: String,
    pub reason: String,
}

/// Emitted when the job is blocked and cannot proceed without resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobBlockedPayload {
    pub job_id: String,
    /// Human-readable description of what is blocking progress.
    pub blocker: String,
    pub needs_human: bool,
}

/// Emitted when the agent signals the job is done but before gate/merge.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobCompletedPayload {
    pub job_id: String,
    pub slug: String,
}

/// Emitted when the job's branch is merged.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobMergedPayload {
    pub job_id: String,
    pub slug: String,
}

/// Emitted when the job terminates with an error.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobFailedPayload {
    pub job_id: String,
    pub error: String,
}

/// Emitted when an executor's spawn() returns Err — i.e. the agent
/// process never started. Distinct from JobFailedPayload because it
/// carries the *stage* of dispatch that failed (isolation prepare,
/// agent compile, executor spawn) so the operator can debug the
/// right layer.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobSpawnFailedPayload {
    pub job_id: String,
    pub slug: String,
    /// Which dispatch phase failed: "isolation_prepare" |
    /// "agent_compile" | "executor_spawn".
    pub stage: String,
    /// Error message captured at the failure site.
    pub error: String,
}

/// Emitted when an operator (or system policy) cancels a job. Terminal.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobCancelledPayload {
    pub job_id: String,
    pub slug: String,
    /// Free-form reason. UI surfaces this verbatim.
    pub reason: String,
    /// Who/what initiated the cancellation: "operator", "supervisor",
    /// "policy:timeout", etc. Open-vocabulary string.
    pub requested_by: String,
}

/// Emitted by either side (human, commander, or agent) to send a mid-flight
/// update, instruction, or course correction. The daemon routes these to the
/// agent's mailbox.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobUpdatePayload {
    pub job_id: String,
    pub message: String,
    /// Who sent this update — e.g. "human", "commander", agent mesh ID.
    pub sender: String,
}

/// Emitted when a pipeline step begins executing.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepDispatchedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub step_type: String,
}

/// Emitted when a pipeline step finishes successfully.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepCompletedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
}

/// Emitted when a pipeline step fails. Halts pipeline advancement.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepFailedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub error: String,
}

/// Emitted by run steps to capture stdout/stderr output.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepOutputPayload {
    pub job_id: String,
    pub step_id: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Emitted by an agent via the `step_done` MCP tool to declare step completion.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepDoneDeclaredPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub summary: Option<String>,
    /// Map of `produces.name → artifact_id` for outputs the agent uploaded.
    /// Empty when the step declares no `produces` or the agent omitted outputs.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub outputs: std::collections::BTreeMap<String, String>,
}

/// Emitted when a gated step is retried after gate failure.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepRetryPayload {
    pub job_id: String,
    pub slug: String,
    pub step_id: String,
    pub step_name: String,
    pub attempt: u32,
    pub reason: String,
}

/// What triggered the step observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum StepTrigger {
    /// Agent explicitly called step_done
    DoneDeclared,
    /// Process/session exited
    ProcessExit,
    /// Activity settled — turns ended, evidence accumulated, no new activity
    TurnSettled,
    /// Step timed out
    Timeout,
    /// Exit channel closed unexpectedly
    ChannelClosed,
}

/// Verdict derived from accumulated evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum StepVerdict {
    /// Strong evidence of success — advance pipeline
    Advance,
    /// Strong evidence of failure — halt pipeline
    Failed,
    /// Inconclusive — needs human review or gate
    Inconclusive,
}

/// Emitted by the step monitor after gathering evidence about a step's outcome.
///
/// This is evidence-based, not verdict-based. The step monitor collects all
/// available signals (process exit, step_done, git state) and the pipeline
/// router decides what to do based on the full picture.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StepObservedPayload {
    pub job_id: String,
    pub slug: String,
    pub step_index: usize,
    pub step_name: String,
    /// What triggered this observation.
    pub trigger: StepTrigger,
    // ── Evidence fields ──────────────────────────────────────────────
    /// Agent explicitly called step_done (strongest signal).
    pub done_declared: bool,
    /// Value passed to step_done, if called. True = agent says success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done_result: Option<bool>,
    /// Process exit code, if available. None for tmux (no exit code exposed).
    pub exit_code: Option<i32>,
    /// Number of git commits on the branch since the step started.
    #[serde(default)]
    pub commits_since_start: u32,
    /// Number of files with uncommitted changes in the worktree.
    #[serde(default)]
    pub uncommitted_files: u32,
    /// Wall-clock duration of the step.
    pub duration_secs: u64,
    /// Human-readable reason (e.g. "timeout", error message).
    pub reason: Option<String>,
    // ── Derived ──────────────────────────────────────────────────────
    /// Verdict computed from evidence. Used by pipeline router.
    pub verdict: StepVerdict,
    /// Back-compat: true when verdict == Advance. Existing consumers and
    /// projections that read `succeeded` continue to work.
    #[serde(default)]
    pub succeeded: bool,
    /// HEAD SHA at step start — baseline for post-step diff gate evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_sha: Option<String>,
    /// Last ~30 lines of the agent's stdout/stderr log. Populated only on
    /// non-Advance verdicts so operators can see *why* a step failed
    /// without SSHing into the worktree. ANSI escapes stripped. Provider-
    /// agnostic — works for any executor that writes to
    /// `.artifacts/agent-<slug>.log` (ProcessExecutor does).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_tail: Option<String>,
}

impl StepObservedPayload {
    /// Derive a verdict from the collected evidence.
    ///
    /// Philosophy: no single signal maps to a verdict. Triggers, exit codes,
    /// commits, uncommitted files, and step_done are evidence to weigh. The
    /// verdict is Advance only when evidence is clearly positive, Failed only
    /// when clearly negative, Inconclusive otherwise — the router will push
    /// Inconclusive outcomes through a gate step where the reviewer decides.
    ///
    /// The only signal that maps 1-1 to a verdict is `step_done(bool)` —
    /// an explicit declaration from the agent itself.
    ///
    /// **Step contract for agents:** if your step produces no commits and no
    /// uncommitted changes, you must call `step_done(true)` explicitly to mark
    /// success. Channel closure or process exit with zero evidence is treated
    /// as failure — this pins down previously-implicit "noop step" semantics
    /// surfaced by dev-pipe-01 Finding 3.2.
    ///
    /// **Run steps short-circuit on exit code.** A `StepType::Run` step has no
    /// agent and no `step_done` mechanism; its shell exit code is authoritative.
    pub fn compute_verdict(
        step_type: &StepType,
        done_declared: bool,
        done_result: Option<bool>,
        exit_code: Option<i32>,
        commits: u32,
        uncommitted_files: u32,
        trigger: &StepTrigger,
    ) -> StepVerdict {
        // Run steps have no agent and no step_done mechanism. The shell exit
        // code is the whole story — route around the evidence machinery so
        // `cargo check` style verify steps don't require commits to advance.
        if matches!(step_type, StepType::Run) {
            return match exit_code {
                Some(0) => StepVerdict::Advance,
                _ => StepVerdict::Failed,
            };
        }

        // Agent explicitly declared outcome. Trust it. Only authoritative rule.
        if done_declared {
            return match done_result {
                Some(true) | None => StepVerdict::Advance,
                Some(false) => StepVerdict::Failed,
            };
        }

        let nonzero_exit = matches!(exit_code, Some(c) if c != 0);
        let has_commits = commits > 0;
        let has_uncommitted = uncommitted_files > 0;

        // Committed work present. The agent produced something durable.
        // Non-zero exit modifies — agent committed but died angry, gate reviews.
        if has_commits {
            return if nonzero_exit {
                StepVerdict::Inconclusive
            } else {
                StepVerdict::Advance
            };
        }

        // No commits. Uncommitted work means the agent was actively doing
        // something — might have been cut short. Never fail on mid-flight
        // work; let a human or gate decide.
        if has_uncommitted {
            return StepVerdict::Inconclusive;
        }

        // Nothing produced. Weigh the trigger and exit code together.
        match trigger {
            StepTrigger::Timeout => StepVerdict::Failed,
            // No commits, no uncommitted files, no done_declared, and the
            // agent's channel was cut or the process exited. The agent
            // produced zero evidence of any work. Treat as failure — the
            // agent must call step_done(true) to mark a legitimate noop.
            StepTrigger::ChannelClosed | StepTrigger::ProcessExit => StepVerdict::Failed,
            StepTrigger::DoneDeclared => {
                // done_declared == false already handled above. If we get here
                // with DoneDeclared trigger, something is off.
                StepVerdict::Inconclusive
            }
            // Activity-based settlement is different — turns ended but the
            // agent might still have pending work. Leave for gate review
            // rather than failing outright.
            StepTrigger::TurnSettled => StepVerdict::Inconclusive,
        }
    }
}

/// Event type constants for the `job.` namespace.
pub mod event_types {
    pub const JOB_CREATED: &str = "job.created";
    pub const JOB_DISPATCHED: &str = "job.dispatched";
    pub const JOB_GATE_REQUESTED: &str = "job.gate_requested";
    pub const JOB_GATE_PASSED: &str = "job.gate_passed";
    pub const JOB_GATE_FAILED: &str = "job.gate_failed";
    pub const JOB_BLOCKED: &str = "job.blocked";
    pub const JOB_COMPLETED: &str = "job.completed";
    pub const JOB_MERGED: &str = "job.merged";
    pub const JOB_FAILED: &str = "job.failed";
    pub const JOB_SPAWN_FAILED: &str = "job.spawn_failed";
    pub const JOB_CANCELLED: &str = "job.cancelled";
    pub const JOB_UPDATE: &str = "job.update";
    pub const STEP_DISPATCHED: &str = "job.step_dispatched";
    pub const STEP_COMPLETED: &str = "job.step_completed";
    pub const STEP_FAILED: &str = "job.step_failed";
    pub const STEP_OUTPUT: &str = "job.step_output";
    pub const STEP_RETRY: &str = "step.retry";
    pub const STEP_DONE_DECLARED: &str = "step.done_declared";
    pub const STEP_OBSERVED: &str = "step.observed";
}

#[cfg(test)]
mod job_created_payload_tests {
    use super::*;

    #[test]
    fn source_spec_path_round_trips_when_present() {
        let json = r#"{
            "job_id": "01XXX",
            "slug": "test",
            "agent": "rust-runtime",
            "branch": "job/test",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "model": null,
            "provider": null,
            "source_spec_path": "/abs/path/to/spec.md"
        }"#;
        let p: JobCreatedPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.source_spec_path.as_deref(), Some("/abs/path/to/spec.md"));
        // Round-trip through serialization preserves the field.
        let reserialized = serde_json::to_string(&p).unwrap();
        assert!(reserialized.contains("\"source_spec_path\":\"/abs/path/to/spec.md\""));
    }

    #[test]
    fn baseline_sha_round_trips_when_present() {
        // Pin the wire contract for the field gates rely on. Both the
        // serialize-with-value and deserialize-from-value paths must hold
        // — without that, the reviewer step's gate can't see commits made
        // by earlier pipeline steps and fails with an empty diff range.
        let json = r#"{
            "job_id": "01ZZZ",
            "slug": "test",
            "agent": "rust-runtime",
            "branch": "job/test",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "model": null,
            "provider": null,
            "baseline_sha": "deadbeefcafe"
        }"#;
        let p: JobCreatedPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.baseline_sha.as_deref(), Some("deadbeefcafe"));
        let reserialized = serde_json::to_string(&p).unwrap();
        assert!(reserialized.contains("\"baseline_sha\":\"deadbeefcafe\""));
    }

    #[test]
    fn baseline_sha_absent_in_legacy_events() {
        // Legacy payloads must still deserialize and not emit the field
        // when None, so the event log doesn't grow explicit nulls on
        // every new write.
        let legacy_json = r#"{
            "job_id": "01WWW",
            "slug": "old",
            "agent": "rust-runtime",
            "branch": "job/old",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "model": null,
            "provider": null
        }"#;
        let p: JobCreatedPayload = serde_json::from_str(legacy_json).unwrap();
        assert!(p.baseline_sha.is_none());
        let reserialized = serde_json::to_string(&p).unwrap();
        assert!(
            !reserialized.contains("baseline_sha"),
            "None must skip serializing, got: {reserialized}"
        );
    }

    #[test]
    fn source_spec_path_absent_in_legacy_events() {
        // Legacy payloads predating this field must still deserialize — and the
        // field must serialize away when None (skip_serializing_if) so the
        // event log doesn't grow explicit nulls on every new write either.
        let legacy_json = r#"{
            "job_id": "01YYY",
            "slug": "old",
            "agent": "rust-runtime",
            "branch": "job/old",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "model": null,
            "provider": null
        }"#;
        let p: JobCreatedPayload = serde_json::from_str(legacy_json).unwrap();
        assert!(p.source_spec_path.is_none());
        let reserialized = serde_json::to_string(&p).unwrap();
        assert!(!reserialized.contains("source_spec_path"),
            "None must skip serializing, got: {reserialized}");
    }
}

#[cfg(test)]
mod verdict_tests {
    use super::*;

    /// Shorthand for compute_verdict with named args, so each test reads as
    /// a sentence about the evidence, not a pile of positional arguments.
    struct Evidence {
        step_type: StepType,
        done: bool,
        done_result: Option<bool>,
        exit: Option<i32>,
        commits: u32,
        uncommitted: u32,
        trigger: StepTrigger,
    }

    impl Default for Evidence {
        fn default() -> Self {
            Self {
                step_type: StepType::Agent,
                done: false,
                done_result: None,
                exit: None,
                commits: 0,
                uncommitted: 0,
                trigger: StepTrigger::TurnSettled,
            }
        }
    }

    fn verdict(e: Evidence) -> StepVerdict {
        StepObservedPayload::compute_verdict(
            &e.step_type, e.done, e.done_result,
            e.exit, e.commits, e.uncommitted, &e.trigger,
        )
    }

    // ─── The one authoritative signal: step_done ───────────────────────────

    #[test]
    fn done_declared_true_wins_over_everything() {
        // Even with timeout + non-zero exit + no commits, the agent said yes.
        assert_eq!(
            verdict(Evidence {
                done: true, done_result: Some(true),
                exit: Some(1), trigger: StepTrigger::Timeout,
                ..Default::default()
            }),
            StepVerdict::Advance,
        );
    }

    #[test]
    fn done_declared_false_wins_over_commits() {
        // Agent said it failed. Trust over git state.
        assert_eq!(
            verdict(Evidence {
                done: true, done_result: Some(false),
                commits: 10, trigger: StepTrigger::DoneDeclared,
                ..Default::default()
            }),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn done_declared_without_result_is_advance() {
        assert_eq!(
            verdict(Evidence {
                done: true, done_result: None,
                trigger: StepTrigger::DoneDeclared,
                ..Default::default()
            }),
            StepVerdict::Advance,
        );
    }

    // ─── Commits are strong positive evidence ──────────────────────────────

    // Regression: tmux drops exit_tx after sending one ExitSignal. The monitor
    // sees the channel close next recv. With commits present, that's success —
    // agent did the work then cleaned up. Pre-fix code failed here.
    #[test]
    fn commits_with_channel_closed_advance() {
        assert_eq!(
            verdict(Evidence {
                commits: 5, trigger: StepTrigger::ChannelClosed,
                ..Default::default()
            }),
            StepVerdict::Advance,
        );
    }

    #[test]
    fn commits_with_timeout_advance() {
        // Agent timed out but had produced committed work. Advance — the gate
        // can reject if what it did is wrong.
        assert_eq!(
            verdict(Evidence {
                commits: 3, trigger: StepTrigger::Timeout,
                ..Default::default()
            }),
            StepVerdict::Advance,
        );
    }

    #[test]
    fn commits_with_nonzero_exit_is_inconclusive() {
        // Agent committed work then died with an error. Gate decides — maybe
        // the implementation is fine and a final test run bailed.
        assert_eq!(
            verdict(Evidence {
                commits: 5, exit: Some(127),
                trigger: StepTrigger::ProcessExit,
                ..Default::default()
            }),
            StepVerdict::Inconclusive,
        );
    }

    // ─── Uncommitted work means "in flight" — never fail ───────────────────

    #[test]
    fn uncommitted_files_with_timeout_is_inconclusive() {
        // Agent was doing something — we cut it off. Don't punish work in
        // progress; let a human decide.
        assert_eq!(
            verdict(Evidence {
                uncommitted: 4, trigger: StepTrigger::Timeout,
                ..Default::default()
            }),
            StepVerdict::Inconclusive,
        );
    }

    #[test]
    fn uncommitted_files_with_channel_close_is_inconclusive() {
        assert_eq!(
            verdict(Evidence {
                uncommitted: 2, trigger: StepTrigger::ChannelClosed,
                ..Default::default()
            }),
            StepVerdict::Inconclusive,
        );
    }

    // ─── No evidence of work: trigger + exit code decide ───────────────────

    #[test]
    fn timeout_with_no_evidence_fails() {
        // Nothing produced, nothing dirty, we timed out. Real failure.
        assert_eq!(
            verdict(Evidence {
                trigger: StepTrigger::Timeout,
                ..Default::default()
            }),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn channel_close_no_evidence_nonzero_exit_fails() {
        assert_eq!(
            verdict(Evidence {
                exit: Some(1), trigger: StepTrigger::ChannelClosed,
                ..Default::default()
            }),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn process_exit_no_evidence_zero_exit_is_failed() {
        // Agent exited cleanly without producing anything. Before dev-pipe-01
        // this was Inconclusive (pushed to gate); that let no-op pipeline
        // runs silently proceed. Agents must now call step_done(true) for a
        // legitimate noop — bare channel closure with zero evidence fails.
        assert_eq!(
            verdict(Evidence {
                exit: Some(0), trigger: StepTrigger::ProcessExit,
                ..Default::default()
            }),
            StepVerdict::Failed,
        );
    }

    #[test]
    fn turn_settled_with_no_evidence_is_inconclusive() {
        // Agent stopped talking, no commits, no dirty files. Ambiguous.
        assert_eq!(
            verdict(Evidence {
                trigger: StepTrigger::TurnSettled,
                ..Default::default()
            }),
            StepVerdict::Inconclusive,
        );
    }

    // ─── Exit code alone is not a verdict ──────────────────────────────────

    #[test]
    fn nonzero_exit_with_commits_does_not_fail() {
        // Specifically asserts: non-zero exit is a signal to weigh, not a
        // verdict. Earlier code short-circuited Failed here; that violated
        // the evidence-based principle.
        let v = verdict(Evidence {
            exit: Some(2), commits: 1,
            trigger: StepTrigger::ProcessExit,
            ..Default::default()
        });
        assert!(
            matches!(v, StepVerdict::Inconclusive | StepVerdict::Advance),
            "non-zero exit with commits must not short-circuit to Failed, got {v:?}",
        );
    }

    // ─── dev-pipe-01 Finding 3.2 regression guards ─────────────────────────

    #[test]
    fn agent_no_evidence_channel_closed_clean_exit_is_failed() {
        // The exact dev-pipe-01 Finding 3.2 case: the supervisor cut the
        // agent's channel, the agent produced no commits and no dirty files
        // and never called step_done. Before this fix the router advanced
        // via gate review; now it must halt as a failure.
        let v = verdict(Evidence {
            step_type: StepType::Agent,
            exit: Some(0),
            trigger: StepTrigger::ChannelClosed,
            ..Default::default()
        });
        assert_eq!(v, StepVerdict::Failed);
    }

    #[test]
    fn agent_no_evidence_channel_closed_no_exit_is_failed() {
        // tmux can't see an exit code, so this exact shape shows up a lot in
        // practice. Same verdict: no evidence + external channel cut = fail.
        let v = verdict(Evidence {
            step_type: StepType::Agent,
            trigger: StepTrigger::ChannelClosed,
            ..Default::default()
        });
        assert_eq!(v, StepVerdict::Failed);
    }

    #[test]
    fn run_step_clean_exit_advances_regardless_of_commits() {
        // `run:` step with `cargo check && cargo test` exit 0 — verify
        // commands produce no commits, and must still advance.
        let v = verdict(Evidence {
            step_type: StepType::Run,
            exit: Some(0),
            trigger: StepTrigger::ProcessExit,
            ..Default::default()
        });
        assert_eq!(v, StepVerdict::Advance);
    }

    #[test]
    fn run_step_nonzero_exit_fails_regardless_of_commits() {
        // Even with commits, a run step that exits non-zero is a failure —
        // its exit code is the authoritative signal.
        let v = verdict(Evidence {
            step_type: StepType::Run,
            exit: Some(1),
            commits: 5,
            trigger: StepTrigger::ProcessExit,
            ..Default::default()
        });
        assert_eq!(v, StepVerdict::Failed);
    }

    #[test]
    fn run_step_missing_exit_fails() {
        // Shell process gone with no exit code = failure for run steps.
        let v = verdict(Evidence {
            step_type: StepType::Run,
            trigger: StepTrigger::ChannelClosed,
            ..Default::default()
        });
        assert_eq!(v, StepVerdict::Failed);
    }

    #[test]
    fn agent_turn_settled_no_evidence_stays_inconclusive() {
        // Activity-based settlement is different from channel closure —
        // the agent might be paused mid-task, not terminated. Gate reviews.
        let v = verdict(Evidence {
            step_type: StepType::Agent,
            trigger: StepTrigger::TurnSettled,
            ..Default::default()
        });
        assert_eq!(v, StepVerdict::Inconclusive);
    }

    #[test]
    fn agent_done_declared_still_authoritative() {
        // Even with zero commits, an agent that explicitly calls
        // step_done(true) advances. Legitimate noop steps take this path.
        let v = verdict(Evidence {
            step_type: StepType::Agent,
            done: true,
            done_result: Some(true),
            exit: Some(0),
            trigger: StepTrigger::ChannelClosed,
            ..Default::default()
        });
        assert_eq!(v, StepVerdict::Advance);
    }
}

#[cfg(test)]
mod gate_canonical_format_tests {
    //! Contract-freeze Item 1: GateRule must use the list-of-rules + list-of-globs
    //! canonical shape. The legacy single-object and pipe-separated string
    //! shorthands are rejected at deserialize with a pointer to the canonical syntax.
    use super::*;

    #[test]
    fn canonical_list_of_rules_with_glob_lists_round_trips() {
        let yaml = r#"
require_changed:
  - match: ["**/*.rs", "**/*.toml"]
    not_match: ["**/tests/**"]
    min: 2
forbid_changed:
  - match: [".artifacts/**"]
    min: 0
"#;
        let g: GateSpec = serde_yaml::from_str(yaml).expect("canonical shape parses");
        assert_eq!(g.require_changed.len(), 1);
        assert_eq!(g.require_changed[0].match_patterns,
            vec!["**/*.rs".to_string(), "**/*.toml".to_string()]);
        assert_eq!(g.require_changed[0].not_match, vec!["**/tests/**".to_string()]);
        assert_eq!(g.require_changed[0].min, 2);
        assert_eq!(g.forbid_changed[0].match_patterns, vec![".artifacts/**".to_string()]);
    }

    #[test]
    fn legacy_single_object_rule_is_rejected_with_actionable_error() {
        let yaml = r#"
require_changed:
  match: ["**/*.rs"]
  min: 1
"#;
        let err = serde_yaml::from_str::<GateSpec>(yaml).unwrap_err().to_string();
        assert!(
            err.contains("array") && err.contains("contract freeze"),
            "error must point at canonical syntax + freeze rationale: {err}"
        );
    }

    #[test]
    fn legacy_pipe_separated_match_string_is_rejected() {
        let yaml = r#"
require_changed:
  - match: "**/*.rs|**/*.toml"
    min: 1
"#;
        let err = serde_yaml::from_str::<GateSpec>(yaml).unwrap_err().to_string();
        assert!(
            err.contains("list of glob strings") || err.contains("contract freeze"),
            "string match value must be rejected with canonical-syntax pointer: {err}"
        );
    }

    #[test]
    fn pipe_inside_list_entry_is_rejected() {
        // The list shape was always canonical for the loader, but we used to
        // also split entries on '|'. That's the same dual shape one level
        // deeper — reject.
        let yaml = r#"
require_changed:
  - match: ["**/*.rs|**/*.toml"]
    min: 1
"#;
        let err = serde_yaml::from_str::<GateSpec>(yaml).unwrap_err().to_string();
        assert!(err.contains("contains '|'"), "pipe inside list entry must be flagged: {err}");
    }
}

