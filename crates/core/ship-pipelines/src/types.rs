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
    /// Persistent user-personal artifact.
    User,
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
    /// Valid transitions: Run→Project, Run→User, Project→User.
    pub target_scope: OutputScope,
}

/// A single lane in a fanout step — runs the same spec through a different agent/model/provider.
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
    pub provider: Option<String>,
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
    /// Parallel lanes. Each runs the fanout step's spec through a different agent/model/provider.
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
/// Custom `Deserialize` allows YAML shorthand: `run: <command>` implies
/// `type: run` and `name:` is accepted as an alias for `id:`.
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
        let yaml = r#"id: build
run: cargo build"#;
        let s: PipelineStep = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(s.step_type, StepType::Run);
        assert_eq!(s.command.as_deref(), Some("cargo build"));
    }

    #[test]
    fn name_alias_resolves_to_id() {
        let yaml = r#"name: review
type: review
agent: reviewer"#;
        let s: PipelineStep = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(s.id, "review");
        assert_eq!(s.step_type, StepType::Review);
    }

    fn step(id: &str, after: &[&str]) -> PipelineStep {
        PipelineStep {
            id: id.into(),
            step_type: StepType::Agent,
            agent: Some("a".into()),
            goal: None,
            after: after.iter().map(|s| s.to_string()).collect(),
            model: None,
            provider: None,
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
        }
    }
}
