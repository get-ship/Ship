//! Pipeline event payload types.
//!
//! Pipeline-run state is derived entirely from the event log. These payloads
//! cover the `pipeline.run.*`, `pipeline.step.*`, and `pipeline.gate.*`
//! lifecycle events. Two pipeline.* types carry ship-types payloads instead:
//! `pipeline.changed` (ConfigChanged) and `pipeline.step.retry`
//! (ship_types::payload::PipelineStepRetry).
//!
//! The module is split into cohesive submodules so each file stays under the
//! 400-line cap. Public API and visibility are unchanged — every item
//! re-exported here was previously defined directly in `events/pipeline.rs`.

// Pipeline DAG types moved to `ship-pipelines` per modular-runtime-01.
// Re-exported here so existing callers keep compiling unchanged.
pub use ship_pipelines::definition::{
    Invocation, PipelineDefinition, PipelineStepDef, ProviderOverride,
};
pub use ship_pipelines::gate::{GateRule, GateSpec};
pub use ship_pipelines::types::{
    FanoutJudge, FanoutLane, FanoutSpec, OutputScope, Pipeline, PipelinePhase, PipelineStep,
    StepOutput, StepPromote, StepType,
};
mod observed;
mod payloads;

// Kept `pub` (not collapsed into a flat re-export) so external callers
// can still reference `ship_substrate::events::pipeline::event_types::PIPELINE_*`.
pub mod event_types;

#[cfg(test)]
mod tests_gate_canonical;
#[cfg(test)]
mod tests_run_created;
#[cfg(test)]
mod tests_verdict;

pub use observed::{PipelineStepObservedPayload, StepTrigger, StepVerdict};
pub use payloads::*;

