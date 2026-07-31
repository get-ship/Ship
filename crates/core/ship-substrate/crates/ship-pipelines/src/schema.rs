//! Declare-form pipeline schemas — authoring shape for `definePipeline`.
//!
//! Product-owned slice schemas (stack inversion): these types used to live in
//! `stack-model`. Distinct from the runtime DAG types in [`crate::types`]
//! (`Pipeline`, `PipelineStep`, …) — same product vocabulary home, different
//! wire shapes (declare-form vs orchestrator DAG).

use serde::{Deserialize, Serialize};

/// Guardrail on a step: which paths must (or must not) have changed for the
/// step to run. Mirrors `@ship/stack`'s `PipelineGate`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct PipelineGate {
    #[serde(default)]
    pub require_changed: Vec<String>,
    #[serde(default)]
    pub forbid_changed: Vec<String>,
}

/// One concrete job variant a step fans out into. Mirrors
/// `@ship/stack`'s `PipelineStepSpec`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PipelineStepSpec {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

/// One step in a declare-form pipeline. `after` names sibling step ids this
/// step depends on; `gate` guards whether it runs at all; `step_specs` is
/// the set of concrete job variants the step fans out into. Mirrors
/// `@ship/stack`'s `PipelineStep`.
///
/// Distinct from the runtime DAG [`crate::types::PipelineStep`].
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PipelineStep {
    pub id: String,
    #[serde(default)]
    pub after: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<PipelineGate>,
    #[serde(default)]
    pub step_specs: Vec<PipelineStepSpec>,
}

/// A pipeline definition — the authoring shape replacing
/// `pipelines/*.jsonc`. Mirrors `@ship/stack`'s `PipelineDef`
/// (census-format-migration.md Wave 3).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PipelineDef {
    pub name: String,
    pub steps: Vec<PipelineStep>,
    #[serde(default)]
    pub auto_merge: bool,
}
