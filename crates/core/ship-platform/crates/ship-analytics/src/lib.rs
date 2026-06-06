//! ship-analytics — derived rollups + queries over hook.* events.
//!
//! Sits downstream of `ship-substrate`'s event log. Three concerns:
//!
//! 1. **[projection]** — `AnalyticsProjection` folds `hook.session_*`
//!    and `hook.tool_use_*` events into the `analytics_sessions` and
//!    `analytics_tool_uses` tables. Registered with the substrate's
//!    EventBus at daemon startup via `install_extra_projections_factory`.
//!
//! 2. **[queries]** — typed SELECT helpers consumers (dashboard, CLI)
//!    call instead of writing SQL by hand.
//!
//! 3. **[mining]** — per-harness data-dir parsers (planned).
//!    Anthropic, OpenAI, Google, Cursor each store rich session
//!    transcripts in `~/.<harness>/` that already cover everything
//!    hooks try to capture. Mining gives us full historical coverage
//!    AND data for sessions launched outside Ship. Privacy-sensitive:
//!    default local-only; explicit opt-in for cloud sync.
//!
//! The crate does NOT own the migration that defines its tables —
//! schema lives in `ship-substrate/migrations/` so a single migration
//! runner manages the platform.db. Adding a new analytics table = new
//! file in that dir.

pub mod mining;
pub mod projection;
pub mod queries;
pub mod service;
pub mod wire;

#[cfg(test)]
mod tests_projection;

pub use projection::AnalyticsProjection;
