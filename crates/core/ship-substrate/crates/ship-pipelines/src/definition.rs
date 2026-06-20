//! The unified, serializable `PipelineDefinition` — the authoring contract.
//!
//! This is the pipeline analog of the unified Agent: one topology-invariant
//! type that merges the pipeline-level metadata (`PipelineMeta`: name /
//! description / vars / timeout / max_retries / auto_merge) with the DAG of
//! steps into a single `Serialize + Deserialize + specta::Type` struct.
//!
//! Design rules (P0.2):
//!
//! * **Pure authoring contract.** Launch/runtime fields (`headless`,
//!   `keepalive`, …) are deliberately excluded — those ride the dispatch
//!   *request*, not the on-disk file.
//! * **Per-step specs.** Spec bodies attach to each step via [`PipelineStepDef::spec`]
//!   instead of the positional `step_specs` Vec the legacy `PipelineMeta`
//!   carries. The orchestrator writes each into `.artifacts/pipeline/NN-<id>.md`.
//! * **Provider is typed, not dropped.** The `provider` key appears in real
//!   pipeline files and the TS `StepDef`, but was absent from the runtime
//!   `PipelineStep` and silently dropped at parse (`deny_unknown_fields` is
//!   off). [`ProviderOverride`] gives it a typed home shared across the step
//!   and the pipeline-level default, and [`Invocation`] types the harness
//!   invocation mode.
//!
//! **Additive.** This type does not replace the orchestrator's existing parse
//! path (`normalize_pipeline` + `PipelineMeta` + the runtime `Pipeline`). It is
//! the typed authoring contract plus write-time validation; repointing the
//! orchestrator onto it is a later slice.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::gate::GateSpec;
use crate::types::{FanoutSpec, Pipeline, PipelineStep, StepOutput, StepPromote, StepType};

/// Harness invocation mode — the typed replacement for the stringly
/// `invocation` field on the runtime `PipelineStep`.
///
/// * `pty` — interactive PTY (default; subscription-billed for claude).
/// * `print` — one-shot print mode (API-billed for claude post-2026-06).
/// * `stream` — server-mode streaming (codex / opencode only).
///
/// Wire shape is the lowercase variant name so JSONC pipelines stay
/// human-editable without referencing the substrate enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum Invocation {
    Pty,
    Print,
    Stream,
}

impl Invocation {
    /// Lowercase wire token. Matches the legacy stringly form on
    /// `PipelineStep::invocation`.
    pub fn as_str(self) -> &'static str {
        match self {
            Invocation::Pty => "pty",
            Invocation::Print => "print",
            Invocation::Stream => "stream",
        }
    }
}

/// Shared execution-override bundle: which provider / model / harness /
/// invocation a step (or the pipeline default) runs under.
///
/// Flattened onto the wire so authors keep writing `provider: claude` inline
/// on a step. The same type is flattened at the pipeline level as the default
/// applied when a step leaves a field unset. Every field is optional — an
/// all-unset `ProviderOverride` serializes to nothing.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ProviderOverride {
    /// Backend provider (e.g. `claude`, `codex`, `opencode`). Previously
    /// silently dropped at parse; now preserved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Model override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Harness override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    /// Harness invocation mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation: Option<Invocation>,
}

/// A single authoring step in a [`PipelineDefinition`].
///
/// Mirrors the runtime [`PipelineStep`] DAG node but (a) carries the per-step
/// [`spec`](Self::spec) body and (b) folds provider/model/harness/invocation
/// into a flattened [`ProviderOverride`] so `provider` survives the round
/// trip. `type` is optional: an absent type is inferred (`run` when a `run`
/// command is present, else `agent`), preserving the JSONC shorthand that real
/// pipeline files rely on.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PipelineStepDef {
    /// Step identifier, unique within the pipeline.
    pub id: String,
    /// Step kind. Absent → inferred from shape.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub step_type: Option<StepType>,
    /// Agent profile to use (required for agent/review steps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Goal or instruction for the agent step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    /// Steps that must complete before this one runs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<String>,
    /// Shell command to run (`type: run`). Wire name: `run`.
    #[serde(default, rename = "run", skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Maximum retries when a gate fails on this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// Post-step declarative diff gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<GateSpec>,
    /// Artifacts this step is expected to produce.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub produces: Vec<StepOutput>,
    /// Names of prior-step outputs this step needs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumes: Vec<String>,
    /// Outputs to promote to a wider scope on success.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub promotes: Vec<StepPromote>,
    /// When present, run K parallel lanes instead of a single agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fanout: Option<FanoutSpec>,
    /// Per-step spec body. Replaces the positional `step_specs` Vec — the
    /// orchestrator writes this into `.artifacts/pipeline/NN-<id>.md` at launch.
    // DEFERRED SEAM — keep (P0.2 typed authoring contract; the legacy
    // positional `step_specs` Vec on `PipelineMeta` will be retired when
    // the orchestrator's parse path is repointed onto `PipelineDefinition`.
    // Do not remove until that cutover lands.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
    /// Execution override (provider / model / harness / invocation), flattened
    /// onto the step so the wire shape stays flat.
    #[serde(flatten)]
    pub provider: ProviderOverride,
}

impl PipelineStepDef {
    /// Resolve the effective step kind, inferring from shape when `type` is
    /// absent: a step with a `run` command is a `run` step, otherwise `agent`.
    pub fn resolved_type(&self) -> StepType {
        self.step_type.clone().unwrap_or_else(|| {
            if self.command.is_some() {
                StepType::Run
            } else {
                StepType::Agent
            }
        })
    }

    /// Project this authoring step onto the runtime [`PipelineStep`] DAG node.
    /// Drops the per-step `spec` (a launch-time artifact concern) and lowers
    /// the flattened [`ProviderOverride`] back onto `model` / `harness` /
    /// `invocation`. `provider` itself has no runtime field yet; it is carried
    /// on the authoring contract until the orchestrator is repointed.
    pub fn to_step(&self) -> PipelineStep {
        PipelineStep {
            id: self.id.clone(),
            step_type: self.resolved_type(),
            agent: self.agent.clone(),
            goal: self.goal.clone(),
            after: self.after.clone(),
            model: self.provider.model.clone(),
            harness: self.provider.harness.clone(),
            command: self.command.clone(),
            max_retries: self.max_retries,
            gate: self.gate.clone(),
            produces: self.produces.clone(),
            consumes: self.consumes.clone(),
            promotes: self.promotes.clone(),
            fanout: self.fanout.clone(),
            invocation: self.provider.invocation.map(|i| i.as_str().to_string()),
        }
    }
}

/// The unified pipeline authoring contract: pipeline-level metadata plus the
/// step DAG in one serializable type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PipelineDefinition {
    /// Human-readable pipeline name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Free-text description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Template variables interpolated into specs/goals at launch.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub vars: HashMap<String, String>,
    /// Per-step wall-clock timeout (human duration string, e.g. `"30m"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    /// Pipeline-wide retry budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    /// When true, the runtime runs `git merge` on the job branch after the
    /// final step completes.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auto_merge: bool,
    /// The step DAG.
    pub steps: Vec<PipelineStepDef>,
    /// Pipeline-level execution default (provider / model / harness /
    /// invocation), flattened. A per-step [`ProviderOverride`] takes precedence
    /// over this default.
    #[serde(flatten)]
    pub defaults: ProviderOverride,
}

impl PipelineDefinition {
    /// Project the authoring contract onto the runtime [`Pipeline`] DAG. Used
    /// by [`validate`](Self::validate) and available as the seam for repointing
    /// the orchestrator in a later slice.
    pub fn to_pipeline(&self) -> Pipeline {
        Pipeline {
            steps: self.steps.iter().map(PipelineStepDef::to_step).collect(),
            auto_merge: self.auto_merge,
        }
    }

    /// Validate every platform invariant on this definition by lowering to the
    /// runtime DAG and running the shared [`validate_pipeline`](crate::validate::validate_pipeline).
    /// Returns a clear, operator-readable error on the first violation.
    pub fn validate(&self) -> Result<(), crate::error::PipelineError> {
        crate::validate::validate_pipeline(&self.to_pipeline())
    }
}

#[cfg(test)]
mod tests;
