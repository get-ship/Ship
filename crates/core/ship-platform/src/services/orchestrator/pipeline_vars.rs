//! Pipeline variable interpolation — moved to `ship-pipelines::variables`
//! per modular-runtime-01. This file re-exports for back-compat; new
//! callers should import `ship_pipelines::interpolate_step` directly.

pub use ship_pipelines::variables::{interpolate, interpolate_step};
