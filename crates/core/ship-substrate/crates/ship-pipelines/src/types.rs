//! Pipeline DAG types — extracted from `ship_substrate::events::job` per
//! modular-runtime-01. These types are pure data; the daemon, the
//! cloud worker (per cloud-specs/08), and the eventual browser embed
//! all share the same structures.

use serde::{Deserialize, Serialize};

use crate::gate::GateSpec;

/// A single phase in a job pipeline (legacy linear format).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PipelinePhase {
    pub agent: String,
    pub goal: String,
    #[serde(default)]
    pub model: Option<String>,
}

/// Step type in a pipeline DAG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum OutputScope {
    /// Ephemeral — lives for the duration of the run. Default.
    #[default]
    Run,
    /// Persistent project-level artifact.
    Project,
}

/// A named output produced by a pipeline step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct StepPromote {
    /// Must match a `produces.name` on this step.
    pub name: String,
    /// Valid transitions: Run→Project. (A personal/account scope returns later
    /// as Account; there is no promote-to-global paradigm.)
    pub target_scope: OutputScope,
}

/// A single lane in a fanout step — runs the same spec through a different agent/model/harness.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FanoutLane {
    /// Unique identifier within the fanout.
    pub id: String,
    /// Agent profile to use for this lane.
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
}

/// Points to the judge step that arbitrates lane outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FanoutJudge {
    /// Step ID of the judge. Must come after the fanout step in the pipeline DAG.
    pub step: String,
}

/// Fanout configuration — run K agents in parallel, then a judge step picks or merges.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FanoutSpec {
    /// Parallel lanes. Each runs the fanout step's spec through a different agent/model/harness.
    pub lanes: Vec<FanoutLane>,
    /// Judge step that receives all lane outputs and arbitrates.
    pub judge: FanoutJudge,
    /// Minimum number of successful lanes required for the fanout step to succeed. Default: 1.
    #[serde(default = "default_min_successful_lanes")]
    pub min_successful_lanes: usize,
}

fn default_min_successful_lanes() -> usize {
    1
}

/// A single step in a pipeline DAG.
///
/// Custom `Deserialize` implements the JSONC shorthand: a step with a `run`
/// key but no explicit `type` is inferred as `type: run`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    /// Harness override for this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    /// Shell command to run (populated for StepType::Run). Wire name: `run`.
    #[serde(default, rename = "run", skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Maximum number of retries when a gate fails on this step.
    /// `None` or `0` means no retries — gate failure is terminal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// Post-step diff gate. Declarative structural checks evaluated against
    /// the files changed by this step's commits.
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
    /// Harness invocation mode for this step: `"pty"` (default;
    /// subscription-billed for claude), `"print"` (one-shot, API-billed
    /// for claude post-2026-06), or `"stream"` (server-mode, codex /
    /// opencode only). When unset, the runner falls back to the
    /// run-level default (legacy `headless` flag during transition;
    /// `pty` once that field is removed in Phase 5b-5).
    ///
    /// Wire shape is a lowercase string so JSONC pipelines stay
    /// human-editable without referencing the substrate enum.
    /// `ship-platform::services::orchestrator` converts to
    /// `ship_substrate::events::agent_session::InvocationMode` at
    /// spawn time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation: Option<String>,
}

impl<'de> serde::Deserialize<'de> for PipelineStep {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
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
            harness: Option<String>,
            #[serde(rename = "run", default)]
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
            #[serde(default)]
            invocation: Option<String>,
        }
        let raw = Raw::deserialize(d)?;
        let step_type = raw.step_type.unwrap_or_else(|| {
            if raw.command.is_some() {
                StepType::Run
            } else {
                StepType::Agent
            }
        });
        // Validate invocation if present — caller-friendly error
        // surfaces here rather than letting a downstream enum-parse
        // fail with a less informative message.
        if let Some(ref inv) = raw.invocation
            && !matches!(inv.as_str(), "pty" | "print" | "stream")
        {
            return Err(serde::de::Error::custom(format!(
                "invocation must be one of: pty, print, stream (got `{inv}`)"
            )));
        }
        Ok(PipelineStep {
            id: raw.id,
            step_type,
            agent: raw.agent,
            goal: raw.goal,
            after: raw.after,
            model: raw.model,
            harness: raw.harness,
            command: raw.command,
            max_retries: raw.max_retries,
            gate: raw.gate,
            produces: raw.produces,
            consumes: raw.consumes,
            promotes: raw.promotes,
            fanout: raw.fanout,
            invocation: raw.invocation,
        })
    }
}

/// A pipeline definition — a DAG of steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
        self.steps
            .iter()
            .filter(|s| !completed.contains(&s.id))
            .filter(|s| s.after.iter().all(|dep| completed.contains(dep)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_root_steps_are_dependency_free() {
        let p = Pipeline {
            steps: vec![
                step("a", &[]),
                step("b", &["a"]),
                step("c", &[]),
            ],
            auto_merge: false,
        };
        let roots: Vec<&str> = p.root_steps().iter().map(|s| s.id.as_str()).collect();
        assert_eq!(roots, vec!["a", "c"]);
    }

    #[test]
    fn unblocked_steps_skip_completed_and_blocked() {
        let p = Pipeline {
            steps: vec![
                step("a", &[]),
                step("b", &["a"]),
                step("c", &["b"]),
            ],
            auto_merge: false,
        };
        let unblocked: Vec<&str> = p
            .unblocked_steps(&["a".to_string()])
            .iter()
            .map(|s| s.id.as_str())
            .collect();
        assert_eq!(unblocked, vec!["b"]);
    }

    #[test]
    fn run_shorthand_infers_step_type() {
        let json = r#"{ "id": "build", "run": "cargo build" }"#;
        let s: PipelineStep = serde_json::from_str(json).unwrap();
        assert_eq!(s.step_type, StepType::Run);
        assert_eq!(s.command.as_deref(), Some("cargo build"));
    }

    #[test]
    fn legacy_command_key_is_rejected() {
        // The `command` wire key is gone; only `run` is canonical.
        let json = r#"{ "id": "build", "type": "run", "command": "cargo build" }"#;
        let s: PipelineStep = serde_json::from_str(json).unwrap();
        assert!(s.command.is_none(), "command key must not deserialize: {s:?}");
    }

    #[test]
    fn legacy_name_key_is_rejected() {
        // `name` alias on the id field is gone; `id` is required.
        let json = r#"{ "name": "review", "type": "review", "agent": "reviewer" }"#;
        assert!(serde_json::from_str::<PipelineStep>(json).is_err());
    }

    #[test]
    fn invocation_field_defaults_to_none() {
        // Pipelines without an invocation field stay None — the
        // pipeline runner falls back to the run-level default
        // (transitional via headless flag; will become pty once
        // 5b-5 removes the run-level field).
        let json = r#"{ "id": "review", "type": "agent", "agent": "reviewer" }"#;
        let s: PipelineStep = serde_json::from_str(json).unwrap();
        assert!(s.invocation.is_none());
    }

    #[test]
    fn invocation_field_accepts_pty_print_stream() {
        for mode in ["pty", "print", "stream"] {
            let json = format!(
                r#"{{ "id": "x", "type": "agent", "agent": "a", "invocation": "{mode}" }}"#
            );
            let s: PipelineStep = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("must parse invocation={mode}: {e}"));
            assert_eq!(s.invocation.as_deref(), Some(mode));
        }
    }

    #[test]
    fn invocation_field_rejects_invalid_value() {
        let json = r#"{ "id": "x", "type": "agent", "agent": "a", "invocation": "bogus" }"#;
        let err = serde_json::from_str::<PipelineStep>(json)
            .err()
            .expect("must reject invalid invocation");
        let msg = err.to_string();
        assert!(
            msg.contains("invocation") && msg.contains("pty, print, stream"),
            "error must surface valid options, got: {msg}"
        );
    }

    #[test]
    fn invocation_field_roundtrips_in_serialize() {
        let step = PipelineStep {
            id: "x".into(),
            step_type: StepType::Agent,
            agent: Some("a".into()),
            goal: None,
            after: vec![],
            model: None,
            harness: None,
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
            invocation: Some("print".into()),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"invocation\":\"print\""));
        let parsed: PipelineStep = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.invocation.as_deref(), Some("print"));
    }

    fn step(id: &str, after: &[&str]) -> PipelineStep {
        PipelineStep {
            id: id.into(),
            step_type: StepType::Agent,
            agent: Some("a".into()),
            goal: None,
            after: after.iter().map(|s| s.to_string()).collect(),
            model: None,
            harness: None,
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
            invocation: None,
        }
    }
}
