//! Typed error for the pipeline parse/validate boundary.
//!
//! `Display` is the underlying message verbatim, so caller-visible text is
//! byte-identical to the former `Result<_, String>` surface. The variants
//! type the boundary: which phase rejected the pipeline.
//!
//! Deliberately NOT covered here: `variables::interpolate`'s `Err` payload is
//! the missing *variable name* (a semantic value callers format themselves),
//! not an error message — typing it means owning every caller's formatting,
//! which is a separate slice. `gate::evaluate` folds rule-compile failures
//! into `GateOutcome::Fail` messages and never surfaces a `Result`.

/// Why a pipeline definition was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PipelineError {
    /// The source failed to parse/normalize into the runtime shape
    /// (`parse::normalize_pipeline`).
    #[error("{0}")]
    Parse(String),
    /// A platform invariant failed at load-time validation
    /// (`validate::*`, `gate::validate`, `PipelineDefinition::validate`).
    #[error("{0}")]
    Validate(String),
}
