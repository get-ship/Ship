//! Pipelines view-data helpers.
//!
//! These resolve the data shapes that the pipelines daemon-rendered
//! views need (focus, history, run_detail, summaries, etc.). They sit
//! between the registry dispatch (`registry/pipelines.rs`) and the
//! `ship_platform::services::pipelines` trait surface — they call into
//! the trait + projections to assemble per-render payloads.
//!
//! The pure `PipelinesService` trait + Impl + state + tests live in
//! `ship_platform::services::pipelines`. HTTP handlers are in
//! `apps/shipd/src/routes/pipelines_api.rs`.

pub mod history;
pub mod run_detail;
pub mod shape;
pub mod summaries;
pub mod view_state;
