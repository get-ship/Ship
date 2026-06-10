//! ship-analytics — derived rollups + queries over hook.* events.
//!
//! Sits downstream of `ship-substrate`'s event log. Three concerns:
//!
//! 1. **[projection]** — `AnalyticsProjection` folds `hook.session_*`,
//!    `hook.tool_use_*`, and `hook.turn_observed`, plus
//!    `actor.created`/`actor.stopped` as a fallback session signal, into
//!    the `analytics_sessions`, `analytics_tool_uses`, and
//!    `analytics_turns` tables. Registered with the substrate's
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
//! The crate does NOT own the migrations that define its tables —
//! schema lives in `ship-substrate/migrations/` so a single migration
//! runner manages the per-scope event DBs (analytics rollups land in
//! `runtime.db`). Adding a new analytics table = new file in that dir.

pub mod projection;
pub mod queries;
pub mod service;
pub mod wire;

#[cfg(test)]
mod tests_projection;

pub use projection::AnalyticsProjection;
